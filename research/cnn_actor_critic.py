import argparse
from collections import deque
from itertools import count

import torch
import torch.nn.functional as F
import torch.optim as optim
from cnn_policy import CnnPolicy
from tqdm import tqdm

import racer_gym


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


def train(args: argparse.Namespace, policy: CnnPolicy, optimizer: optim.Optimizer):
    if args.init_state:
        policy.load(args.init_state)  # , optimizer)

    CAR_COUNT = 3
    running_reward = 0.0
    episodes = tqdm(count(args.episode_start))
    for i_episode in episodes:
        env = racer_gym.Environment(
            seed=i_episode, car_count=CAR_COUNT, track_width=56.0
        )
        observation_history = {
            car: deque([observation] * 10, maxlen=10)
            for car, observation in enumerate(env.observations())
        }
        ep_reward = 0
        rewards = []
        log_probs = []
        values = []
        for t in range(60 * 60):
            action, state_value, log_prob = policy.sample_action(observation_history[0])
            actions = [racer_gym.Action(*action)]
            with torch.no_grad():
                for car in range(1, CAR_COUNT):
                    action = policy.argmax_action(observation_history[car])
                    actions.append(racer_gym.Action(*action))
            all_observations, reward, finished = env.step(actions)
            for car, observation in enumerate(all_observations):
                observation_history[car].append(observation)
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
            policy.save(f"policy-cnn-actor-critic/ep{i_episode:05}.pth", optimizer)


def export(args: argparse.Namespace, policy: CnnPolicy, optimizer: optim.Optimizer):
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

    export_parser = sub_parsers.add_parser("export")
    export_parser.add_argument("--state", type=str, required=True)
    export_parser.add_argument("--onnx", type=str, required=True)

    args = parser.parse_args()

    torch.manual_seed(42)
    obs_dim = len(racer_gym.Environment().observations()[0])
    policy = CnnPolicy(obs_dim=obs_dim, action_dim=9, cnn_channels=32, hidden_dim=24)
    optimizer = optim.Adam(policy.parameters(), lr=1e-3)

    if args.cmd == "train":
        train(args, policy, optimizer)
    elif args.cmd == "export":
        export(args, policy, optimizer)


if __name__ == "__main__":
    main()
