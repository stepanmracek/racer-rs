import argparse
from collections import deque
from itertools import count
from typing import cast

import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim
from torch.distributions import Categorical
from tqdm import tqdm
from utils import create_scale_layer, policy_output_to_action

import racer_gym


class Policy(nn.Module):
    def __init__(self, action_dim: int, hidden_dim: int):
        super(Policy, self).__init__()
        self.scale_layer = create_scale_layer(next_waypoint=True, rays_count=18)
        self.obs_dim = self.scale_layer._parameters["bias"].shape[0]
        self.layer1 = nn.Linear(self.obs_dim, hidden_dim)
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

    def save(self, path: str, optimizer: optim.Optimizer):
        torch.save(
            {
                "policy": self.state_dict(),
                "optimizer": optimizer.state_dict(),
                "rng": torch.get_rng_state(),
            },
            path,
        )

    def load(self, path: str, optimizer: optim.Optimizer):
        state = torch.load(path)
        self.load_state_dict(state["policy"])
        torch.set_rng_state(state["rng"])
        optimizer.load_state_dict(state["optimizer"])


def finish_batch(
    optimizer: optim.Optimizer,
    batch_rewards: list[list[float]],
    batch_log_probs: list[list[torch.Tensor]],
):
    policy_loss = torch.scalar_tensor(0.0)
    all_returns = []

    # Calculate returns for each trajectory in the batch
    for rewards in batch_rewards:
        R = 0
        returns = deque()
        for r in reversed(rewards):
            R = r + 0.99 * R
            returns.appendleft(R)
        all_returns.extend(returns)

    # Normalize returns across the entire batch
    all_returns = torch.tensor(all_returns)
    all_returns = (all_returns - all_returns.mean()) / (all_returns.std() + 1e-8)

    # Calculate policy loss for all trajectories in the batch
    return_idx = 0
    for log_probs in batch_log_probs:
        episode_returns = all_returns[return_idx : return_idx + len(log_probs)]
        for log_prob, R in zip(log_probs, episode_returns):
            policy_loss += -log_prob * R
        return_idx += len(log_probs)

    # Average the loss over the batch
    policy_loss /= len(batch_rewards)

    optimizer.zero_grad()
    policy_loss.backward()
    optimizer.step()


def train(args: argparse.Namespace, policy: Policy, optimizer: optim.Optimizer):
    if args.init_state:
        policy.load(args.init_state, optimizer)

    running_reward = 10
    episodes = tqdm(count(args.episode_start))
    trajectories = args.trajectories

    for episode in episodes:
        total_trajectories_reward = 0
        batch_rewards = []
        batch_log_probs = []

        for trajectory in range(trajectories):
            total_reward = 0
            env = racer_gym.Environment(
                seed=episode * trajectory,
                off_track_prob=args.off_track_prob,
                goal=racer_gym.Goal.ReachFinish,
            )
            observation = env.observation()
            rewards = []
            log_probs = []

            for t in range(60 * 60):
                action, log_prob = policy.sample_action(observation)
                observation, reward, finished = env.step(*action)
                rewards.append(reward)
                log_probs.append(log_prob)
                total_reward += reward

                if finished:
                    break

            if len(rewards) >= 10:
                running_reward = 0.05 * total_reward + (1 - 0.05) * running_reward
                batch_rewards.append(rewards)
                batch_log_probs.append(log_probs)
                total_trajectories_reward += total_reward

        if batch_rewards and batch_log_probs:
            finish_batch(optimizer, batch_rewards, batch_log_probs)
            print(
                f"{episode},{total_trajectories_reward / len(batch_rewards):.2f},{running_reward:.2f}"
            )
            episodes.set_postfix_str(
                f"Running reward: {running_reward:.2f}", refresh=False
            )

        if episode % args.snapshot_interval_episodes == 0:
            policy.save(f"{args.snapshot_prefix}{episode:05}.pth", optimizer)


def export(args: argparse.Namespace, policy: Policy, optimizer: optim.Optimizer):
    policy.load(args.state, optimizer)
    policy.export(args.onnx)


def main():
    parser = argparse.ArgumentParser()
    sub_parsers = parser.add_subparsers(required=True, dest="cmd")

    train_parser = sub_parsers.add_parser("train")
    train_parser.add_argument("--init-state", type=str, required=False)
    train_parser.add_argument("--episode-start", type=int, required=False, default=1)
    train_parser.add_argument(
        "--snapshot-interval-episodes", type=int, required=False, default=100
    )
    train_parser.add_argument(
        "--snapshot-prefix", type=str, required=False, default="policy-reinforce/ep"
    )
    train_parser.add_argument(
        "--off-track-prob", type=float, required=False, default=0.5
    )
    train_parser.add_argument("--trajectories", type=int, required=False, default=4)

    export_parser = sub_parsers.add_parser("export")
    export_parser.add_argument("--state", type=str, required=True)
    export_parser.add_argument("--onnx", type=str, required=True)

    args = parser.parse_args()

    torch.manual_seed(42)
    policy = Policy(action_dim=9, hidden_dim=32)
    optimizer = optim.Adam(policy.parameters(), lr=1e-3)

    if args.cmd == "train":
        train(args, policy, optimizer)
    elif args.cmd == "export":
        export(args, policy, optimizer)


if __name__ == "__main__":
    main()
