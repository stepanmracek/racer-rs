import math
import os
import time
from typing import Any

import gymnasium as gym
import numpy as np
import numpy.typing as npt
from gymnasium import spaces
from stable_baselines3 import PPO
from stable_baselines3.common.callbacks import (
    EvalCallback,
    StopTrainingOnNoModelImprovement,
)
from stable_baselines3.common.env_util import make_vec_env
from stable_baselines3.common.vec_env import SubprocVecEnv

import racer_gym


class RacerEnv(gym.Env):
    def __init__(self) -> None:
        super().__init__()
        self.action_space = spaces.Box(low=-1.0, high=1.0, shape=(2,))

        pi_6 = math.pi / 6.0
        self.observation_space = spaces.Box(low=0.0, high=1.0, shape=(26,))

        self.obs_low = np.array(
            [-167.0, -pi_6, -math.pi, -205.0] + 4 * [0.0] + 18 * [0.0], dtype=np.float32
        )
        self.obs_high = np.array(
            [167.0, pi_6, math.pi, 205.0] + 4 * [1.0] + 18 * [205.0],
            dtype=np.float32,
        )

        self.racer = None

    def norm_observation(self, raw_observation):
        return (raw_observation - self.obs_low) / (self.obs_high - self.obs_low)

    def reset(
        self, seed=None, options=None
    ) -> tuple[npt.NDArray[np.float32], dict[str, Any]]:
        if seed is None:
            seed = time.perf_counter_ns()
        super().reset(seed=seed)
        self.racer = racer_gym.Environment(seed=seed)
        obs = self.norm_observation(self.racer.observation())
        return np.array(obs), {}

    def step(
        self, action
    ) -> tuple[npt.NDArray[np.float32], float, bool, bool, dict[str, Any]]:
        obs, reward, done = self.racer.step(action[0], action[1])
        return self.norm_observation(np.array(obs)), reward, done, False, {}


def main():
    log_dir = "./ppo_tensorboard/"
    os.makedirs(log_dir, exist_ok=True)
    env = make_vec_env(
        RacerEnv,
        n_envs=4,
        vec_env_cls=SubprocVecEnv,
        monitor_dir=log_dir,
    )

    model = PPO(
        "MlpPolicy",
        env,
        verbose=1,
        n_steps=64 * 60,  # 1 minute (@ 60fps)
        tensorboard_log=log_dir,
    )

    stop_callback = StopTrainingOnNoModelImprovement(
        max_no_improvement_evals=10,
        min_evals=10,
        verbose=1,
    )
    eval_callback = EvalCallback(
        model.env,
        eval_freq=100_000,
        callback_after_eval=stop_callback,
        verbose=1,
    )
    model.learn(
        total_timesteps=60 * 60 * 60 * 24 * 2,  # 2 days (@ 60fps)
        progress_bar=True,
        callback=eval_callback,
    )

    model.save("sb3_ppo")
    print(model.policy)


if __name__ == "__main__":
    main()
