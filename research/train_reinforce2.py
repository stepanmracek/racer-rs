from itertools import count
from collections import deque
import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim
from torch.distributions import Categorical
from train_reinforce import create_scale_layer, policy_output_to_action
import racer_gym

class Policy(nn.Module):
    def __init__(self, data_path: str, obs_dim: int, action_dim: int, hidden_dim: int):
        super(Policy, self).__init__()
        self.obs_dim = obs_dim
        self.scale_layer = create_scale_layer(data_path, obs_dim)
        self.layer1 = nn.Linear(obs_dim, hidden_dim)
        self.layer2 = nn.Linear(hidden_dim, action_dim)

        self.saved_log_probs = []
        self.rewards = []

    def forward(self, x):
        scaled = self.scale_layer(x)
        y = F.relu(self.layer1(scaled))
        return F.softmax(self.layer2(y), dim=-1)

    def export(self, path):
        dummy_input = torch.randn(1, self.obs_dim, dtype=torch.float32)
        torch.onnx.export(
            self,
            (dummy_input,),
            path,
            input_names=["input"],
            output_names=["output"],
        )


def select_action(policy, observation):
    state = torch.tensor(observation, dtype=torch.float32)
    probs = policy(state)
    m = Categorical(probs)
    action = m.sample()
    policy.saved_log_probs.append(m.log_prob(action))
    return action.item()


def finish_episode(policy, optimizer):
    R = 0
    policy_loss = 0
    returns = deque()
    for r in policy.rewards[::-1]:
        R = r + 0.99 * R
        returns.appendleft(R)
    returns = torch.tensor(returns)
    returns = (returns - returns.mean()) / (returns.std() + 1e-8)
    for log_prob, R in zip(policy.saved_log_probs, returns):
        policy_loss += (-log_prob * R)
    optimizer.zero_grad()
    #policy_loss = torch.cat(policy_loss).sum()
    policy_loss.backward()
    optimizer.step()
    del policy.rewards[:]
    del policy.saved_log_probs[:]


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
        for t in range(60*60):
            policy_output = select_action(policy, observation)
            observation, reward, finished = env.step(*policy_output_to_action[policy_output])
            policy.rewards.append(reward)
            ep_reward += reward

            if finished:
                break

        running_reward = 0.05 * ep_reward + (1 - 0.05) * running_reward
        finish_episode(policy, optimizer)
        print(f"{i_episode},{ep_reward:.2f},{running_reward:.2f}")

        if i_episode % 100 == 0:
            policy.export(f"policy-reinforce2-ep{i_episode:05}.onnx")


if __name__ == '__main__':
    main()
