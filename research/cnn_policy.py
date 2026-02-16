from collections.abc import Sequence
from typing import cast

import torch
import torch.nn as nn
import torch.nn.functional as F
import utils
from torch.distributions import Categorical


class CnnPolicy(nn.Module):
    def __init__(
        self, obs_dim: int, action_dim: int, cnn_channels: int, hidden_dim: int
    ):
        super().__init__()
        self.obs_dim = obs_dim
        self.scale_layer = utils.create_scale_layer(next_waypoint=True, rays_count=18)
        # 1D temporal convolution (batch, obs_dim, timestep)
        self.conv1 = nn.Conv1d(
            in_channels=obs_dim,
            out_channels=cnn_channels,
            kernel_size=3,
            stride=1,
            padding=1,
        )
        self.fc = nn.Linear(cnn_channels, hidden_dim)
        self.action_head = nn.Linear(hidden_dim, action_dim)
        self.value_head = nn.Linear(hidden_dim, 1)

    def forward(self, x: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        """
        x.shape == (batch, obs_dim, timestep)
        """
        # Apply scaling per frame - reshape to (batch * time, obs_dim)
        b, c, t = x.shape
        x = x.permute(0, 2, 1).reshape(b * t, c)
        x = self.scale_layer(x)
        x = x.view(b, t, c).permute(0, 2, 1)

        # Temporal convolution
        y = F.relu(self.conv1(x))  # -> (batch, cnn_channels, timestep)

        y = y.mean(dim=-1)  # -> (batch, cnn_channels)
        y = F.relu(self.fc(y))  # -> (batch, hidden_dim)

        action_logits = self.action_head(y)
        value = self.value_head(y)
        return F.softmax(action_logits, dim=-1), value

    def sample_action(
        self, observation: Sequence[list[float]]
    ) -> tuple[tuple[float, float], torch.Tensor, torch.Tensor]:
        x = torch.tensor(observation, dtype=torch.float32).T.unsqueeze(0)
        probs, state_value = self(x)
        m = Categorical(probs[0])
        sample = m.sample()
        action = utils.policy_output_to_action[cast(int, sample.item())]
        return action, state_value[0], m.log_prob(sample)

    def argmax_action(self, observation: Sequence[list[float]]) -> tuple[float, float]:
        x = torch.tensor(observation, dtype=torch.float32).T.unsqueeze(0)
        probs, _ = self(x)
        probs = probs[0]
        action_index = torch.argmax(probs).item()
        return utils.policy_output_to_action[cast(int, action_index)]

    def export(self, path):
        dummy_input = torch.randn(1, self.obs_dim, 10, dtype=torch.float32)
        torch.onnx.export(
            self, (dummy_input,), path, input_names=["input"], output_names=["output"]
        )

    def save(self, path: str, optimizer: torch.optim.Optimizer):
        torch.save(
            {
                "policy": self.state_dict(),
                "optimizer": optimizer.state_dict(),
                "rng": torch.get_rng_state(),
            },
            path,
        )

    def load(self, path: str, optimizer: torch.optim.Optimizer | None = None):
        state = torch.load(path)
        self.load_state_dict(state["policy"])
        torch.set_rng_state(state["rng"])
        if optimizer:
            optimizer.load_state_dict(state["optimizer"])
