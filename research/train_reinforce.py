from collections import deque
import racer_gym
import torch
import torch.nn as nn
import torch.optim as optim
import numpy as np
import pandas as pd
from tqdm import tqdm
from torch.distributions import Categorical
from sklearn.preprocessing import MinMaxScaler

class PolicyNet(nn.Module):
    def __init__(self, data_path: str, obs_dim: int, action_dim: int, hidden_dim: int):
        super().__init__()
        self.obs_dim = obs_dim
        self.scale_layer = create_scale_layer(data_path, obs_dim)
        self.net = nn.Sequential(
            nn.Linear(obs_dim, hidden_dim),
            nn.ReLU(),
            #nn.Linear(hidden_dim, hidden_dim),
            #nn.ReLU(),
            nn.Linear(hidden_dim, action_dim),
            nn.Softmax(dim=-1)
        )

    def forward(self, observation):
        scaled = self.scale_layer(observation)
        return self.net(scaled)

    def export(self, path):
        dummy_input = torch.randn(1, self.obs_dim, dtype=torch.float32)
        torch.onnx.export(
            self,
            (dummy_input,),
            path,
            input_names=["input"],          # names for inputs
            output_names=["output"],        # names for outputs
            dynamic_axes={                  # allow variable batch size
                "input": {0: "batch_size"},
                "output": {0: "batch_size"},
            },
        )

def create_scale_layer(data_path: str, obs_dim: int):
    col_names = (
        ["velocity", "steering_angle", "next_wp_angle", "next_wp_dist"] +
        [f"wheel_on_track_{i}" for i in ("front_r", "front_l", "rear_r", "rear_l")] +
        [f"sensor_readings_{i}" for i in range(13)] +
        ["target_steer", "target_throttle"]
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

def sample_action(policy, observation):
    observation = torch.tensor(observation, dtype=torch.float32)
    probs = policy(observation)
    m = Categorical(probs)
    action = m.sample()
    return action.item(), m.log_prob(action)

def train(batches=500, episodes_per_batch=100, gamma=0.995):
    obs_dim = len(racer_gym.Environment().observation())
    episode_steps = 60*60; # 1 minute @ 60fps
    policy = PolicyNet(data_path="train.csv", obs_dim=obs_dim, action_dim=9, hidden_dim=32)
    optimizer = optim.Adam(policy.parameters(), lr=1e-3)

    for batch in range(batches):
        loss = 0
        batch_rewards = []
        for ep in tqdm(range(episodes_per_batch), desc=f"batch: {batch}"):
            seed = abs(hash((batch, ep)))
            env = racer_gym.Environment(seed)
            log_probs, rewards = [], []
            observation = env.observation()
            for _step in range(episode_steps):
                policy_output, log_prob = sample_action(policy, observation)
                observation, reward, finished = env.step(*policy_output_to_action[policy_output])

                assert not finished

                log_probs.append(log_prob)
                rewards.append(reward)


            # --- Compute returns (simple REINFORCE) ---
            returns = deque()
            G = 0.0
            for r in reversed(rewards):
                G = r + gamma * G
                returns.appendleft(G)
            returns = torch.tensor(returns, dtype=torch.float32)

            # Normalize returns (helps stability)
            returns = (returns - returns.mean()) / (returns.std() + 1e-8)

            # --- Policy Gradient Update ---
            for log_prob, R in zip(log_probs, returns):
                loss += -log_prob * R  # maximize expected return

            batch_rewards.append(sum(rewards))

        optimizer.zero_grad()
        loss.backward()
        optimizer.step()

        print(f"avg reward per episode: {np.mean(batch_rewards):.2f}±{np.std(batch_rewards):.2f}")

    return policy


if __name__ == "__main__":
    policy = train(batches=20, episodes_per_batch=1, gamma=0.99)
    policy.export("policy.onnx")
