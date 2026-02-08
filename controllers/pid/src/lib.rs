use racer_logic::{
    constants::SENSOR_REACH,
    controller::Controller,
    environment::Action,
    observation::{Observation, get_distance},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PidController {
    pub steer: Pid,
    pub throttle: Pid,
    pub side_sensors_reach: f32,
    pub side_sensors_exp: f32,
    pub side_sensors_coef: f32,
    pub front_sensor_exp: f32,
    pub min_speed: f32,
    pub max_speed: f32,
}

impl PidController {
    pub fn load(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}

impl Default for PidController {
    fn default() -> Self {
        Self {
            steer: Pid::default(),
            throttle: Pid::default(),
            side_sensors_reach: 50.0,
            side_sensors_exp: 1.0,
            side_sensors_coef: 50.0,
            front_sensor_exp: 1.0,
            min_speed: 30.0,
            max_speed: 170.0,
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
        let distances = o.sensors.distances[6..=12]
            .iter()
            .map(|d| {
                (get_distance(d) / self.side_sensors_reach)
                    .clamp(0.0, 1.0)
                    .powf(self.side_sensors_exp)
            })
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

        let front_sensor =
            (get_distance(&o.sensors.distances[9]) / SENSOR_REACH).powf(self.front_sensor_exp);
        let target_speed = self.min_speed + front_sensor * (self.max_speed - self.min_speed);
        let error = target_speed - o.velocity;
        let throttle = self.throttle.update(error);

        Action::new(steer, throttle)
    }

    fn reset(&mut self) {
        self.steer.reset();
        self.throttle.reset();
    }
}
