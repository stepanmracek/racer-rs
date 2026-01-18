use racer_logic::{
    controller::Controller,
    environment::{Action, Observation},
};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct PidController {
    pid_steer: Pid,
    pid_throttle: Pid,
}

impl PidController {
    pub fn load(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    pub fn random() -> Self {
        Self {
            pid_steer: Pid::random(),
            pid_throttle: Pid::random(),
        }
    }

    pub fn cross(first: &PidController, second: &PidController, ratio: f32, mutation: f32) -> Self {
        Self {
            pid_steer: Pid::cross(&first.pid_steer, &second.pid_steer, ratio, mutation),
            pid_throttle: Pid::cross(&first.pid_throttle, &second.pid_throttle, ratio, mutation),
        }
    }
}

const KP_RANGE: std::ops::RangeInclusive<f32> = 0.001..=5.0;
const KI_RANGE: std::ops::RangeInclusive<f32> = 0.001..=5.0;
const KD_RANGE: std::ops::RangeInclusive<f32> = 0.001..=5.0;

#[derive(Serialize, Deserialize, Clone)]
struct Pid {
    kp: f32,
    ki: f32,
    kd: f32,

    #[serde(skip_serializing, default)]
    prev_error: f32,
    #[serde(skip_serializing, default)]
    integral: f32,
}

impl Pid {
    fn random() -> Self {
        let mut rng = rand::rng();
        Self {
            kp: rng.random_range(KP_RANGE),
            ki: rng.random_range(KI_RANGE),
            kd: rng.random_range(KD_RANGE),
            prev_error: 0.0,
            integral: 0.0,
        }
    }

    fn clamp(&mut self) {
        self.kp = self.kp.clamp(*KP_RANGE.start(), *KP_RANGE.end());
        self.ki = self.ki.clamp(*KI_RANGE.start(), *KI_RANGE.end());
        self.kd = self.kd.clamp(*KD_RANGE.start(), *KD_RANGE.end());
    }

    fn cross(first: &Pid, second: &Pid, ratio: f32, mutation: f32) -> Self {
        let mut rng = rand::rng();

        let random_select = |first: f32, second: f32| {
            if rand::random::<f32>() < ratio {
                first
            } else {
                second
            }
        };
        let mut ans = Self {
            kp: random_select(first.kp, second.kp),
            ki: random_select(first.ki, second.ki),
            kd: random_select(first.kd, second.kd),
            prev_error: 0.0,
            integral: 0.0,
        };

        let mut mutate = |value: f32| {
            let low = 1.0 - mutation / 2.0;
            let high = 1.0 + mutation / 2.0;
            value * rng.random_range(low..=high)
        };
        ans.kp = mutate(ans.kp);
        ans.ki = mutate(ans.ki);
        ans.kd = mutate(ans.kd);
        ans.clamp();

        ans
    }

    fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
    }

    fn update(&mut self, error: f32) -> f32 {
        let dt = 1.0 / 60.0;
        let p = self.kp * error;

        self.integral += error * dt;
        let i = self.ki * self.integral;

        let derivative = (error - self.prev_error) / dt;
        let d = self.kd * derivative;

        self.prev_error = error;

        p + i + d
    }
}

impl Controller for PidController {
    fn control(&mut self, o: &Observation) -> Action {
        let sensor_reach = 205.0f32;
        let distances = o.sensors.distances[6..=12]
            .iter()
            .map(|d| d.unwrap_or(sensor_reach))
            .collect::<Vec<f32>>();

        let right_space = distances[0..=2].iter().sum::<f32>();
        let left_space = distances[4..=6].iter().sum::<f32>();
        let steer_error = (left_space - right_space) / (1.0 * sensor_reach);
        let steer = self.pid_steer.update(steer_error);

        let velocity_error = distances[3] - o.velocity;
        let throttle = self.pid_throttle.update(velocity_error);

        Action::new(steer, throttle)
    }

    fn reset(&mut self) {
        self.pid_steer.reset();
        self.pid_throttle.reset();
    }
}
