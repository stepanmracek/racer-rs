from collections import deque
from itertools import count
from sklearn.preprocessing import MinMaxScaler
from torch.distributions import Categorical
from typing import cast
import pandas as pd
import racer_gym
import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim


class Policy(nn.Module):
    def __init__(self, data_path: str, obs_dim: int, action_dim: int, hidden_dim: int):
        super(Policy, self).__init__()
        self.obs_dim = obs_dim
        self.scale_layer = create_scale_layer(data_path, obs_dim)
        self.layer1 = nn.Linear(obs_dim, hidden_dim)
        self.layer2 = nn.Linear(hidden_dim, action_dim)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        scaled = self.scale_layer(x)
        y = F.relu(self.layer1(scaled))
        return F.softmax(self.layer2(y), dim=-1)

    def sample_action(self, observation) -> tuple[tuple[float, float], torch.Tensor]:
        x = torch.tensor(observation, dtype=torch.float32)
        probs = self(x)
        m = Categorical(probs)
        sample = m.sample()
        action = policy_output_to_action[cast(int, sample.item())]
        return action, m.log_prob(sample)

    def export(self, path):
        dummy_input = torch.randn(1, self.obs_dim, dtype=torch.float32)
        torch.onnx.export(
            self,
            (dummy_input,),
            path,
            input_names=["input"],
            output_names=["output"],
        )


def create_scale_layer(data_path: str, obs_dim: int) -> nn.Linear:
    col_names = (
        ["velocity", "steering_angle", "next_wp_angle", "next_wp_dist"]
        + [f"wheel_on_track_{i}" for i in ("front_r", "front_l", "rear_r", "rear_l")]
        + [f"sensor_readings_{i}" for i in range(13)]
        + ["target_steer", "target_throttle"]
    )
    data = pd.read_csv(data_path, names=col_names)
    scaler = MinMaxScaler(feature_range=(-1, 1), copy=True, clip=False)
    scaler.fit(data)
    min_, max_ = scaler.feature_range
    scale = (max_ - min_) / (scaler.data_max_ - scaler.data_min_)
    bias = -scaler.data_min_ * scale + min_

    scale_layer = nn.Linear(obs_dim, obs_dim)
    with torch.no_grad():
        scale = torch.tensor(scale[:obs_dim], dtype=torch.float32)
        bias = torch.tensor(bias[:obs_dim], dtype=torch.float32)
        scale_layer.weight.copy_(torch.diag(scale))
        scale_layer.bias.copy_(bias)
    scale_layer.requires_grad_(False)

    return scale_layer


policy_output_to_action = {
    0: (1.0, 1.0),
    1: (0.0, 1.0),
    2: (-1.0, 1.0),
    3: (1.0, 0.0),
    4: (0.0, 0.0),
    5: (-1.0, 0.0),
    6: (1.0, -1.0),
    7: (0.0, -1.0),
    8: (-1.0, -1.0),
}


def finish_episode(optimizer: optim.Optimizer, rewards: list[float], log_probs: list[torch.Tensor]):
    R = 0
    policy_loss = torch.scalar_tensor(0.0)
    returns = deque()
    for r in reversed(rewards):
        R = r + 0.99 * R
        returns.appendleft(R)
    returns = torch.tensor(returns)
    returns = (returns - returns.mean()) / (returns.std() + 1e-8)
    for log_prob, R in zip(log_probs, returns):
        policy_loss += -log_prob * R
    optimizer.zero_grad()
    policy_loss.backward()
    optimizer.step()


def main():
    torch.manual_seed(42)
    obs_dim = len(racer_gym.Environment().observation())
    policy = Policy("train.csv", obs_dim=obs_dim, action_dim=9, hidden_dim=32)
    optimizer = optim.Adam(policy.parameters(), lr=1e-3)

    running_reward = 10
    for i_episode in count(1):
        env = racer_gym.Environment(seed=i_episode)
        observation = env.observation()
        ep_reward = 0
        rewards = []
        log_probs = []
        for t in range(60 * 60):
            action, log_prob = policy.sample_action(observation)
            observation, reward, finished = env.step(*action)
            rewards.append(reward)
            log_probs.append(log_prob)
            ep_reward += reward

            if finished:
                break

        running_reward = 0.05 * ep_reward + (1 - 0.05) * running_reward
        finish_episode(optimizer, rewards, log_probs)
        print(f"{i_episode},{ep_reward:.2f},{running_reward:.2f}")

        if i_episode % 100 == 0:
            policy.export(f"policy-reinforce/policy-reinforce-ep{i_episode:05}.onnx")


if __name__ == "__main__":
    main()
