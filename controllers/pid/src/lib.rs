use racer_logic::{
    controller::Controller,
    environment::{Action, Observation},
};

pub struct PidController {
    pid_steer: Pid,
    pid_throttle: Pid,
}

impl PidController {
    pub fn new() -> Self {
        Self {
            pid_steer: Pid::new(0.8, 0.05, 0.2),
            pid_throttle: Pid::new(0.5, 0.01, 0.3),
        }
    }
}

struct Pid {
    kp: f32,
    ki: f32,
    kd: f32,
    prev_error: f32,
    integral: f32,
}

impl Pid {
    fn new(kp: f32, ki: f32, kd: f32) -> Self {
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
    }
}
