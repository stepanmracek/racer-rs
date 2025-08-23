from collections import deque
import racer_gym
import torch
import torch.nn as nn
import torch.optim as optim
import numpy as np
from tqdm import tqdm
from torch.distributions import Normal

class PolicyNet(nn.Module):
    def __init__(self, obs_dim, action_dim, hidden_dim):
        super().__init__()
        self.obs_dim = obs_dim
        self.net = nn.Sequential(
            nn.Linear(obs_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
        )
        self.mean_head = nn.Linear(hidden_dim, action_dim)
        self.log_std = nn.Parameter(torch.zeros(action_dim))  # learned or fixed

    def forward(self, state):
        x = self.net(state)
        mean = torch.tanh(self.mean_head(x))  # keep mean in [-1,1]
        return mean, self.log_std.exp()  # std > 0

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


def sample_action(policy, state):
    state = torch.tensor(state, dtype=torch.float32)
    mean, std = policy(state)
    dist = Normal(mean, std)
    action = dist.rsample()                # reparameterized sample
    log_prob = dist.log_prob(action).sum() # sum over action dims
    action_clipped = torch.clamp(action, -1, 1)
    return action_clipped.detach().numpy(), log_prob

def train(batches=500, episodes_per_batch=100, gamma=0.995):
    obs_dim = len(racer_gym.Environment().observation())
    episode_steps = 60*60; # 1 minute @ 60fps
    policy = PolicyNet(obs_dim=obs_dim, action_dim=2, hidden_dim=32)
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
                action, log_prob = sample_action(policy, observation)
                steer, throttle = action
                observation, reward, finished = env.step(steer, throttle)

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


policy = train(batches=10, episodes_per_batch=20, gamma=0.99)
policy.export("policy.onnx")
