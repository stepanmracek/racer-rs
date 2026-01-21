use racer_logic::{
    controller::Controller,
    environment::{Action, Observation},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PidController {
    pub steer: Pid,
    pub throttle: Pid,
    pub side_sensors_reach: f32,
    pub side_sensors_coef: f32,
    pub front_sensor_coef: f32,
    pub min_speed: f32,
    pub max_speed: f32,
}

impl PidController {
    pub fn load(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}

impl Default for PidC ontroller {
    fn default() -> Self {
        Self {
            steer: Pid::default(),
            throttle: Pid::default(),
            side_sensors_reach: 50.0,
            side_sensors_coef: 50.0,
            front_sensor_coef: 1.0,
            min_speed: 75.0,
            max_speed: 150.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Pid {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,

    #[serde(skip_serializing, default)]
    prev_error: f32,
    #[serde(skip_serializing, default)]
    integral: f32,
}

impl Pid {
    pub fn new(kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            prev_error: 0.0,
            integral: 0.0,
        }
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

impl Default for Pid {
    fn default() -> Self {
        Self {
            kp: 1.0,
            ki: 0.0,
            kd: 0.0,
            prev_error: 0.0,
            integral: 0.0,
        }
    }
}

impl Controller for PidController {
    fn control(&mut self, o: &Observation) -> Action {
        let sensor_reach = 205.0f32;
        let distances = o.sensors.distances[6..=12]
            .iter()
            .map(|d| (d.unwrap_or(sensor_reach) / self.side_sensors_reach).clamp(0.0, 1.0))
            .collect::<Vec<f32>>();

        let right_space = if !o.wheels_on_track[0] || !o.wheels_on_track[2] {
            0.0
        } else {
            distances[0..=1].iter().sum::<f32>() / 2.0
        };
        let left_space = if !o.wheels_on_track[1] || !o.wheels_on_track[3] {
            0.0
        } else {
            distances[5..=6].iter().sum::<f32>() / 2.0
        };
        let error = (left_space - right_space) * self.side_sensors_coef;
        let steer = self.steer.update(error);

        let target_speed = (o.sensors.distances[9].unwrap_or(sensor_reach)
            * self.front_sensor_coef)
            .clamp(self.min_speed, self.max_speed);
        let error = target_speed - o.velocity;
        let throttle = self.throttle.update(error);

        Action::new(steer, throttle)
    }

    fn reset(&mut self) {
        self.steer.reset();
        self.throttle.reset();
    }
}
