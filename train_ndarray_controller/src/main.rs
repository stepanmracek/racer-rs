use ndarray::Array2;
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Normal;
use racer_logic::{controller::Controller, environment::EnvironmentBuilder};
use racer_ndarray_controller::{Layer, NdarrayController};
use serde::Serialize;

#[derive(Clone, Serialize)]
struct Individual(NdarrayController);

impl ga::Individual for Individual {
    fn cross(&self, other: &Self, ratio: f32, mutation: f32) -> Self {
        let layers: Vec<Layer> = std::iter::zip(self.0.layers.iter(), other.0.layers.iter())
            .map(|(first, second)| {
                let w = ratio * &first.w + (1.0 - ratio) * &second.w;
                let b = ratio * &first.b + (1.0 - ratio) * &second.b;
                Layer {
                    w: &w + Array2::random(w.raw_dim(), Normal::new(0.0, mutation).unwrap()),
                    b: &b + Array2::random(b.raw_dim(), Normal::new(0.0, mutation).unwrap()),
                }
            })
            .collect();

        Individual(NdarrayController::new(layers))
    }

    fn evaluate(&mut self, env_seed: u64) -> f32 {
        let mut env = EnvironmentBuilder::default()
            .with_seed(Some(env_seed))
            .build();
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
    let mut population: Vec<_> = (0..init_size)
        .map(|_| Individual(NdarrayController::random()))
        .collect();

    for generation in 0.. {
        population = ga::step(
            population, &settings, generation, None, // Some("research/ndarray_controllers/"),
        );
    }
}
