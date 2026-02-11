import argparse
from itertools import islice

import actor_critic
import msgpack
import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F
import utils
from torch.utils.data import DataLoader, IterableDataset
from tqdm import tqdm

import racer_gym


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
        return action_logits, value
        # no softmax, because loss function accepts log_softmax
        # F.softmax(action_logits, dim=-1)


def train_data(state: str, teacher_runs: int, output: str):
    env = racer_gym.Environment()
    observation = env.observation()
    src_policy = actor_critic.Policy(
        obs_dim=len(observation), action_dim=9, hidden_dim=32
    )
    state = torch.load(state)
    src_policy.load_state_dict(state["policy"])

    packer = msgpack.Packer()
    with torch.no_grad(), open(output, "wb") as f:
        for seed in tqdm(range(teacher_runs), desc="Creating training data"):
            env.reset(seed=seed)
            observations = []
            teacher_probs = []
            teacher_values = []

            observation = env.observation()
            terminated = False
            while not terminated:
                probs, value = src_policy(torch.tensor(observation))
                observations.append([float(f) for f in observation])
                teacher_probs.append([float(f) for f in probs])
                teacher_values.append([float(f) for f in value])
                action = utils.policy_output_to_action[torch.argmax(probs).item()]
                observation, reward, terminated = env.step(*action)

            f.write(
                packer.pack(
                    {
                        "observations": observations,
                        "teacher_probs": teacher_probs,
                        "teacher_values": teacher_values,
                    }
                )
            )


class TeacherDataset(IterableDataset):
    def __init__(self, input_file: str, timesteps: int):
        self.input_file = input_file
        self.timesteps = timesteps

    def __iter__(self):
        while True:
            with open(self.input_file, "rb") as f:
                for trajectory in msgpack.Unpacker(f):
                    observations = torch.tensor(trajectory["observations"])
                    teacher_probs = torch.tensor(trajectory["teacher_probs"])
                    teacher_values = torch.tensor(trajectory["teacher_values"])

                    obs = []
                    for i in range(len(observations) - self.timesteps):
                        obs.append(observations[i : i + self.timesteps].T)

                    yield (
                        torch.stack(obs),
                        teacher_probs[self.timesteps :],
                        teacher_values[self.timesteps :],
                    )


def train(input_file: str, student_epochs: int):
    env = racer_gym.Environment()
    observation = env.observation()
    dst_policy = CnnPolicy(
        obs_dim=len(observation), action_dim=9, cnn_channels=32, hidden_dim=24
    )

    dataset = TeacherDataset(input_file, timesteps=10)
    loader = DataLoader(dataset, batch_size=None)

    optimizer = torch.optim.Adam(dst_policy.parameters(), lr=1e-3)
    for step, (observations, teacher_probs, teacher_values) in enumerate(
        islice(loader, student_epochs)
    ):
        optimizer.zero_grad()

        student_logits, student_values = dst_policy(observations)
        student_log_probs = F.log_softmax(student_logits, dim=-1)
        probs_loss = F.kl_div(student_log_probs, teacher_probs, reduction="batchmean")
        value_loss = F.mse_loss(student_values, teacher_values)
        loss = probs_loss + 0.1 * value_loss

        loss.backward()
        optimizer.step()

        print(
            f"{step}: probs_loss={probs_loss.item():.4f}, value_loss={value_loss.item():.4f}"
        )

        # train_progress.set_description_str(f"loss: {loss.item():.4f}", refresh=False)


def main():
    parser = argparse.ArgumentParser()
    sub_parsers = parser.add_subparsers(required=True, dest="cmd")

    data_parser = sub_parsers.add_parser("train-data")
    data_parser.add_argument("--state", type=str, required=True)
    data_parser.add_argument("--teacher-runs", type=int, required=True)
    data_parser.add_argument("--output", type=str, required=True)

    train_parser = sub_parsers.add_parser("train")
    train_parser.add_argument("--student-epochs", type=int, required=True)
    train_parser.add_argument("--teacher-data", type=str, required=True)
    args = parser.parse_args()

    match args.cmd:
        case "train-data":
            train_data(
                state=args.state,
                teacher_runs=args.teacher_runs,
                output=args.output,
            )
        case "train":
            train(
                input_file=args.teacher_data,
                student_epochs=args.student_epochs,
            )


if __name__ == "__main__":
    main()
