use ndarray::Array2;
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Normal;
use racer_logic::{
    controller::Controller,
    environment::{Environment, ReachFinish},
};
use racer_ndarray_controller::{Layer, NdarrayController};

//mod ga;

fn evaluate(controller: &mut NdarrayController, env_seed: u64) -> f32 {
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
    first: &NdarrayController,
    second: &NdarrayController,
    ratio: f32,
    noise_std: f32,
) -> NdarrayController {
    let layers: Vec<Layer> = std::iter::zip(first.layers.iter(), second.layers.iter())
        .map(|(first, second)| {
            let w = ratio * &first.w + (1.0 - ratio) * &second.w;
            let b = ratio * &first.b + (1.0 - ratio) * &second.b;
            Layer {
                w: &w + Array2::random(w.raw_dim(), Normal::new(0.0, noise_std).unwrap()),
                b: &b + Array2::random(b.raw_dim(), Normal::new(0.0, noise_std).unwrap()),
            }
        })
        .collect();

    NdarrayController::new(layers)
}

fn main() {
    let settings = ga::Settings::default();
    let init_size = settings.init_size();
    let ga = ga::GA::new(settings, evaluate, cross);
    let mut population: Vec<_> = (0..init_size)
        .map(|_| NdarrayController::random())
        .collect();

    for generation in 0.. {
        population = ga.step(
            population, generation, None, // Some("research/ndarray_controllers/"),
        );
    }
}
