import argparse
from itertools import islice

import actor_critic
import mscdgpack
import torch
import torch.nn.functional as F
import utils
from cnn_policy import CnnPolicy
from torch.utils.data import DataLoader, IterableDataset
from tqdm import tqdm

import racer_gym


def train_data(input_state_path: str, teacher_runs: int, output: str):
    env = racer_gym.Environment()
    observation = env.observation()
    src_policy = actor_critic.Policy(
        obs_dim=len(observation), action_dim=9, hidden_dim=32
    )
    input_state = torch.load(input_state_path)
    src_policy.load_state_dict(input_state["policy"])

    packer = msgpack.Packer()
    with torch.no_grad(), open(output, "wb") as f:
        for seed in tqdm(range(teacher_runs), desc="Creating training data"):
            env.reset(seed=seed)
            observations = []
            teacher_probs = []
            teacher_values = []

            observation = env.observation()
            terminated = False
            count = 0
            while not terminated:
                probs, value = src_policy(torch.tensor(observation))
                observations.append([float(f) for f in observation])
                teacher_probs.append([float(f) for f in probs])
                teacher_values.append([float(f) for f in value])
                action = utils.policy_output_to_action[torch.argmax(probs).item()]
                observation, reward, terminated = env.step(*action)
                count += 1
                if count >= 60_00:
                    break

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
                    for i in range(len(observations) - self.timesteps + 1):
                        obs.append(observations[i : i + self.timesteps].T)

                    yield (
                        torch.stack(obs),
                        teacher_probs[self.timesteps - 1 :],
                        teacher_values[self.timesteps - 1 :],
                    )


def train(input_file: str, student_epochs: int, output_state_path: str):
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

        student_probs, student_values = dst_policy(observations)
        student_log_probs = torch.log(student_probs)
        probs_loss = F.kl_div(student_log_probs, teacher_probs, reduction="batchmean")
        value_loss = F.mse_loss(student_values, teacher_values)
        loss = probs_loss + 0.1 * value_loss

        loss.backward()
        optimizer.step()

        print(
            f"{step}: probs_loss={probs_loss.item():.4f}, value_loss={value_loss.item():.4f}"
        )

    dst_policy.save(output_state_path, optimizer)


def export(input_state_path: str, output_onnx_path: str):
    env = racer_gym.Environment()
    observation = env.observation()
    policy = CnnPolicy(
        obs_dim=len(observation), action_dim=9, cnn_channels=32, hidden_dim=24
    )
    optimizer = torch.optim.Adam(policy.parameters(), lr=1e-3)
    policy.load(input_state_path, optimizer)
    policy.export(output_onnx_path)


def main():
    parser = argparse.ArgumentParser()
    sub_parsers = parser.add_subparsers(required=True, dest="cmd")

    data_parser = sub_parsers.add_parser("train-data")
    data_parser.add_argument("--input-state", type=str, required=True)
    data_parser.add_argument("--teacher-runs", type=int, required=True)
    data_parser.add_argument("--output", type=str, required=True)

    train_parser = sub_parsers.add_parser("train")
    train_parser.add_argument("--student-epochs", type=int, required=True)
    train_parser.add_argument("--teacher-data", type=str, required=True)
    train_parser.add_argument("--output-state", type=str, required=True)

    export_parser = sub_parsers.add_parser("export")
    export_parser.add_argument("--input-state", type=str, required=True)
    export_parser.add_argument("--output-onnx", type=str, required=True)

    args = parser.parse_args()

    match args.cmd:
        case "train-data":
            train_data(
                input_state_path=args.input_state,
                teacher_runs=args.teacher_runs,
                output=args.output,
            )
        case "train":
            train(
                input_file=args.teacher_data,
                student_epochs=args.student_epochs,
                output_state_path=args.output_state,
            )
        case "export":
            export(
                input_state_path=args.input_state,
                output_onnx_path=args.output_onnx,
            )


if __name__ == "__main__":
    main()
