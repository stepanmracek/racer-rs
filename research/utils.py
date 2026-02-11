import math

import numpy as np
import pandas as pd
import torch
import torch.nn as nn
from sklearn.preprocessing import MinMaxScaler


def create_scale_layer_from_ranges(
    data_min: np.ndarray,
    data_max: np.ndarray,
    target_range: tuple[float, float] = (-1.0, 1.0),
) -> nn.Linear:
    min_, max_ = target_range
    scale = (max_ - min_) / (data_max - data_min)
    bias = -data_min * scale + min_

    dim = len(data_min)
    scale_layer = nn.Linear(dim, dim)
    with torch.no_grad():
        scale = torch.tensor(scale, dtype=torch.float32)
        bias = torch.tensor(bias, dtype=torch.float32)
        scale_layer.weight.copy_(torch.diag(scale))
        scale_layer.bias.copy_(bias)
    scale_layer.requires_grad_(False)

    return scale_layer


def create_scale_layer_from_csv(data_path: str, obs_dim: int) -> nn.Linear:
    col_names = (
        ["velocity", "steering_angle", "next_wp_angle", "next_wp_dist"]
        + [f"wheel_on_track_{i}" for i in ("front_r", "front_l", "rear_r", "rear_l")]
        + [f"sensor_readings_{i}" for i in range(13)]
        + ["target_steer", "target_throttle"]
    )
    data = pd.read_csv(data_path, names=col_names)
    data = data.to_numpy()[:, :obs_dim]
    scaler = MinMaxScaler(feature_range=(-1, 1), copy=True, clip=False)
    scaler.fit(data)
    min_, max_ = scaler.feature_range
    return create_scale_layer_from_ranges(
        scaler.data_min_, scaler.data_max_, scaler.feature_range
    )


def create_scale_layer(next_waypoint: bool, rays_count: int) -> nn.Linear:
    # velocity and steering angle
    ranges = [(-50.0, 150.0), (-math.pi / 6, math.pi / 6)]

    if next_waypoint:
        ranges.extend([(-math.pi, math.pi), (0, 205)])

    # wheels on track
    ranges.extend([(0, 1)] * 4)

    # rays
    ranges.extend([(0, 205)] * rays_count)

    data_min, data_max = zip(*ranges)
    return create_scale_layer_from_ranges(
        np.array(data_min), np.array(data_max), (-1.0, 1.0)
    )


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

action_to_policy_output = {v: k for k, v in policy_output_to_action.items()}
