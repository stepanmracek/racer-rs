use racer_logic::{
    controller::Controller,
    environment::{Environment, ReachFinish},
};
use racer_pid_controller::{Pid, PidController};
use rand::Rng;
use serde::Serialize;

const KP_RANGE: std::ops::RangeInclusive<f32> = 0.001..=5.0;
const KI_RANGE: std::ops::RangeInclusive<f32> = 0.001..=5.0;
const KD_RANGE: std::ops::RangeInclusive<f32> = 0.001..=5.0;
const SIDE_SENSORS_RANGE: std::ops::RangeInclusive<f32> = 10.0..=100.0;
const SIDE_SENSORS_COEF: std::ops::RangeInclusive<f32> = 10.0..=100.0;
const FRONT_SENSOR_COEF: std::ops::RangeInclusive<f32> = 0.5..=2.0;
const MIN_SPEED_RANGE: std::ops::RangeInclusive<f32> = 10.0..=75.0;
const MAX_SPEED_RANGE: std::ops::RangeInclusive<f32> = 100.0..=200.0;

fn random_pid() -> Pid {
    let mut rng = rand::rng();
    Pid::new(
        rng.random_range(KP_RANGE),
        rng.random_range(KI_RANGE),
        rng.random_range(KD_RANGE),
    )
}

pub fn random_ctrl() -> PidController {
    let mut rng = rand::rng();
    PidController {
        steer: random_pid(),
        throttle: random_pid(),
        side_sensors_reach: rng.random_range(SIDE_SENSORS_RANGE),
        side_sensors_coef: rng.random_range(SIDE_SENSORS_COEF),
        front_sensor_coef: rng.random_range(FRONT_SENSOR_COEF),
        min_speed: rng.random_range(MIN_SPEED_RANGE),
        max_speed: rng.random_range(MAX_SPEED_RANGE),
    }
}

fn select_and_mutate(
    rng: &mut rand::rngs::ThreadRng,
    first: f32,
    second: f32,
    ratio: f32,
    range: std::ops::RangeInclusive<f32>,
    mutation: f32,
) -> f32 {
    let mut ans = if rng.random::<f32>() < ratio {
        first
    } else {
        second
    };

    let low = 1.0 - mutation / 2.0;
    let high = 1.0 + mutation / 2.0;
    ans *= rng.random_range(low..=high);

    ans.clamp(*range.start(), *range.end())
}

fn cross_pids(
    rng: &mut rand::rngs::ThreadRng,
    first: &Pid,
    second: &Pid,
    ratio: f32,
    mutation: f32,
) -> Pid {
    Pid::new(
        select_and_mutate(rng, first.kp, second.kp, ratio, KP_RANGE, mutation),
        select_and_mutate(rng, first.ki, second.ki, ratio, KI_RANGE, mutation),
        select_and_mutate(rng, first.kd, second.kd, ratio, KD_RANGE, mutation),
    )
}

#[derive(Clone, Serialize)]
struct Individual(PidController);

impl ga::Individual for Individual {
    fn cross(&self, other: &Self, ratio: f32, mutation: f32) -> Self {
        let mut rng = rand::rng();
        Individual(PidController {
            steer: cross_pids(&mut rng, &self.0.steer, &other.0.steer, ratio, mutation),
            throttle: cross_pids(
                &mut rng,
                &self.0.throttle,
                &other.0.throttle,
                ratio,
                mutation,
            ),
            side_sensors_reach: select_and_mutate(
                &mut rng,
                self.0.side_sensors_reach,
                other.0.side_sensors_reach,
                ratio,
                SIDE_SENSORS_RANGE,
                mutation,
            ),
            side_sensors_coef: select_and_mutate(
                &mut rng,
                self.0.side_sensors_coef,
                other.0.side_sensors_coef,
                ratio,
                SIDE_SENSORS_COEF,
                mutation,
            ),
            front_sensor_coef: select_and_mutate(
                &mut rng,
                self.0.front_sensor_coef,
                other.0.front_sensor_coef,
                ratio,
                SIDE_SENSORS_COEF,
                mutation,
            ),
            min_speed: select_and_mutate(
                &mut rng,
                self.0.min_speed,
                other.0.min_speed,
                ratio,
                MIN_SPEED_RANGE,
                mutation,
            ),
            max_speed: select_and_mutate(
                &mut rng,
                self.0.max_speed,
                other.0.max_speed,
                ratio,
                MAX_SPEED_RANGE,
                mutation,
            ),
        })
    }

    fn evaluate(&mut self, env_seed: u64) -> f32 {
        self.0.reset();
        let mut reward = 0.0;
        for t in 0..4 {
            let mut env = Environment::new(
                Some(env_seed + t * 100_000),
                0.0,
                Box::new(ReachFinish::default()),
            );
            for _ in 0..60 * 60 {
                let action = self.0.control(&env.observation);
                let output = env.step(&action, true);
                reward += output.reward;
                if output.terminated {
                    break;
                }
            }
        }

        reward
    }
}

fn main() {
    let settings = ga::Settings::default();
    let init_size = settings.init_size();
    let mut population: Vec<_> = (0..init_size).map(|_| Individual(random_ctrl())).collect();

    for generation in 0.. {
        population = ga::step(
            population,
            &settings,
            generation,
            Some("train_pid_controller/controllers/"),
        );
    }
}
