from collections import deque
from itertools import count
from torch.distributions import Categorical
from tqdm import tqdm
from typing import cast
import argparse
import racer_gym
import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim

from utils import create_scale_layer, policy_output_to_action


class Policy(nn.Module):
    def __init__(self, data_path: str, obs_dim: int, action_dim: int, hidden_dim: int):
        super(Policy, self).__init__()
        self.obs_dim = obs_dim
        self.scale_layer = create_scale_layer(next_waypoint=True, rays_count=18)
        self.layer1 = nn.Linear(obs_dim, hidden_dim)
        self.action_head = nn.Linear(hidden_dim, action_dim)
        self.value_head = nn.Linear(hidden_dim, 1)

    def forward(self, x: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        scaled = self.scale_layer(x)
        y = F.relu(self.layer1(scaled))
        return F.softmax(self.action_head(y), dim=-1), self.value_head(y)

    def sample_action(self, observation) -> tuple[tuple[float, float], torch.Tensor, torch.Tensor]:
        x = torch.tensor(observation, dtype=torch.float32)
        probs, state_value = self(x)
        m = Categorical(probs)
        sample = m.sample()
        action = policy_output_to_action[cast(int, sample.item())]
        return action, state_value, m.log_prob(sample)

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


def finish_episode(
    optimizer: optim.Optimizer,
    rewards: list[float],
    log_probs: list[torch.Tensor],
    values: list[torch.Tensor],
):
    R = 0
    policy_loss = torch.scalar_tensor(0.0)
    value_loss = torch.scalar_tensor(0.0)
    returns = deque()
    for r in reversed(rewards):
        R = r + 0.99 * R
        returns.appendleft(R)
    returns = torch.tensor(returns)
    returns = (returns - returns.mean()) / (returns.std() + 1e-8)
    for log_prob, R, value in zip(log_probs, returns, values):
        advantage = R - value.item()
        policy_loss += -log_prob * advantage
        value_loss += F.smooth_l1_loss(value, torch.tensor([R]))

    optimizer.zero_grad()
    loss = policy_loss + value_loss
    loss.backward()
    optimizer.step()


def train(args: argparse.Namespace, policy: Policy, optimizer: optim.Optimizer):
    if args.init_state:
        policy.load(args.init_state, optimizer)

    running_reward = 10
    episodes = tqdm(count(args.episode_start))
    for i_episode in episodes:
        env = racer_gym.Environment(seed=i_episode)
        observation = env.observation()
        ep_reward = 0
        rewards = []
        log_probs = []
        values = []
        for t in range(60 * 60):
            action, state_value, log_prob = policy.sample_action(observation)
            observation, reward, finished = env.step(*action)
            rewards.append(reward)
            log_probs.append(log_prob)
            values.append(state_value)
            ep_reward += reward

            if finished:
                break

        running_reward = 0.05 * ep_reward + (1 - 0.05) * running_reward
        finish_episode(optimizer, rewards, log_probs, values)
        print(f"{i_episode},{ep_reward:.2f},{running_reward:.2f}")
        episodes.set_postfix_str(f"Running reward: {running_reward:.2f}", refresh=False)

        if i_episode % args.snapshot_interval_episodes == 0:
            policy.save(f"policy-actor-critic/ep{i_episode:05}.pth", optimizer)


def export(args: argparse.Namespace, policy: Policy, optimizer: optim.Optimizer):
    policy.load(args.state, optimizer)
    policy.export(args.onnx)


def main():
    parser = argparse.ArgumentParser()
    sub_parsers = parser.add_subparsers(required=True, dest="cmd")

    train_parser = sub_parsers.add_parser("train")
    train_parser.add_argument("--init-state", type=str, required=False)
    train_parser.add_argument("--episode-start", type=int, required=False, default=1)
    train_parser.add_argument("--snapshot-interval-episodes", type=int, required=False, default=100)

    export_parser = sub_parsers.add_parser("export")
    export_parser.add_argument("--state", type=str, required=True)
    export_parser.add_argument("--onnx", type=str, required=True)

    args = parser.parse_args()

    torch.manual_seed(42)
    obs_dim = len(racer_gym.Environment().observation())
    policy = Policy("train.csv", obs_dim=obs_dim, action_dim=9, hidden_dim=32)
    optimizer = optim.Adam(policy.parameters(), lr=1e-3)

    if args.cmd == "train":
        train(args, policy, optimizer)
    elif args.cmd == "export":
        export(args, policy, optimizer)


if __name__ == "__main__":
    main()
