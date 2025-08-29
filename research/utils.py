from sklearn.preprocessing import MinMaxScaler
import pandas as pd
import torch
import torch.nn as nn


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
