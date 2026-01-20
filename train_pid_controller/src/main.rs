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

fn random_pid() -> Pid {
    let mut rng = rand::rng();
    Pid::new(
        rng.random_range(KP_RANGE),
        rng.random_range(KI_RANGE),
        rng.random_range(KD_RANGE),
    )
}

pub fn random_ctrl() -> PidController {
    PidController::new(random_pid(), random_pid())
}

fn clamp(pid: &mut Pid) {
    pid.kp = pid.kp.clamp(*KP_RANGE.start(), *KP_RANGE.end());
    pid.ki = pid.ki.clamp(*KI_RANGE.start(), *KI_RANGE.end());
    pid.kd = pid.kd.clamp(*KD_RANGE.start(), *KD_RANGE.end());
}

fn cross(first: &Pid, second: &Pid, ratio: f32, mutation: f32) -> Pid {
    let mut rng = rand::rng();

    let random_select = |first: f32, second: f32| {
        if rand::random::<f32>() < ratio {
            first
        } else {
            second
        }
    };
    let mut ans = Pid::new(
        random_select(first.kp, second.kp),
        random_select(first.ki, second.ki),
        random_select(first.kd, second.kd),
    );

    let mut mutate = |value: f32| {
        let low = 1.0 - mutation / 2.0;
        let high = 1.0 + mutation / 2.0;
        value * rng.random_range(low..=high)
    };
    ans.kp = mutate(ans.kp);
    ans.ki = mutate(ans.ki);
    ans.kd = mutate(ans.kd);
    clamp(&mut ans);

    ans
}

#[derive(Clone, Serialize)]
struct Individual(PidController);

impl ga::Individual for Individual {
    fn cross(&self, other: &Self, ratio: f32, mutation: f32) -> Self {
        Individual(PidController::new(
            cross(&self.0.steer, &other.0.steer, ratio, mutation),
            cross(&self.0.throttle, &other.0.throttle, ratio, mutation),
        ))
    }

    fn evaluate(&mut self, env_seed: u64) -> f32 {
        self.0.reset();
        let mut env = Environment::new(Some(env_seed), 0.0, Box::new(ReachFinish::default()));
        let mut reward = 0.0;
        for _ in 0..60 * 60 {
            let action = self.0.control(&env.observation);
            let output = env.step(&action, true);
            reward += output.reward;
            if output.terminated {
                break;
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
