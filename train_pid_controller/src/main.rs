use racer_logic::{
    controller::Controller,
    environment::{Environment, ReachFinish},
};
use racer_pid_controller::PidController;

fn evaluate(controller: &mut PidController, env_seed: u64) -> f32 {
    controller.reset();
    let mut env = Environment::new(Some(env_seed), 0.0, Box::new(ReachFinish::default()));
    let mut reward = 0.0;
    for _ in 0..60 * 60 {
        let action = controller.control(&env.observation);
        let output = env.step(&action, true);
        reward += output.reward;
        if output.terminated {
            break;
        }
    }

    reward
}

fn cross(
    first: &PidController,
    second: &PidController,
    ratio: f32,
    mutation: f32,
) -> PidController {
    PidController::cross(first, second, ratio, mutation)
}

fn main() {
    let settings = ga::Settings::default();
    let init_size = settings.init_size();
    let ga = ga::GA::new(settings, evaluate, cross);
    let mut population: Vec<_> = (0..init_size).map(|_| PidController::random()).collect();

    for generation in 0.. {
        population = ga.step(
            population,
            generation,
            Some("train_pid_controller/controllers/"),
        );
    }
}
