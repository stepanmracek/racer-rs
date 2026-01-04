use kdam::tqdm;
use ndarray::Array2;
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Normal;
use racer_logic::{
    controller::Controller,
    environment::{Environment, ReachFinish},
};
use racer_ndarray_controller::{Layer, NdarrayController};
use rand::Rng;

struct GASettings {
    elite: usize,
    parents: usize,
    selected_pairs: usize,
    childred_per_pair: usize,
    mutation: f32,
}

impl Default for GASettings {
    fn default() -> Self {
        Self {
            elite: 50,
            parents: 200,
            selected_pairs: 195,
            childred_per_pair: 10,
            mutation: 0.1,
        }
    }
}

fn evaluate(controller: &mut NdarrayController, env_seed: u64) -> f32 {
    let mut env = Environment::new(Some(env_seed), 0.0, Box::new(ReachFinish::default()));
    let mut reward = 0.0;
    for _ in 0..60 * 60 {
        let action = controller.control(&env.observation);
        let output = env.step(&action, true);
        reward += output.reward;
        if output.finished {
            break;
        }
    }

    return reward;
}

fn evaluate_population(population: &mut [NdarrayController], env_seed: u64) -> Vec<f32> {
    tqdm!(population.iter_mut())
        .map(|ctrl| evaluate(ctrl, env_seed))
        .collect()
}

fn cross(
    first: &NdarrayController,
    second: &NdarrayController,
    ratio: f32,
    noise_std: f32,
) -> NdarrayController {
    let layers: Vec<Layer> = std::iter::zip(first.layers.iter(), second.layers.iter())
        .map(|(first, second)| {
            /*let w_std = (first.w.std(0.0) + second.w.std(0.0)) / 2.0;
            let b_std = (first.b.std(0.0) + second.b.std(0.0)) / 2.0;*/
            let w = ratio * &first.w + (1.0 - ratio) * &second.w;
            let b = ratio * &first.b + (1.0 - ratio) * &second.b;
            Layer {
                /*w: &w + noise_std * Array2::random(w.raw_dim(), Normal::new(0.0, w_std).unwrap()),
                b: &b + noise_std * Array2::random(b.raw_dim(), Normal::new(0.0, b_std).unwrap()),*/
                w: &w + Array2::random(w.raw_dim(), Normal::new(0.0, noise_std).unwrap()),
                b: &b + Array2::random(b.raw_dim(), Normal::new(0.0, noise_std).unwrap()),
            }
        })
        .collect();

    NdarrayController::new(layers)
}

fn random_pair(len: usize, rng: &mut impl Rng) -> (usize, usize) {
    let i = rng.random_range(0..len);
    let offset = rng.random_range(1..len);
    let j = (i + offset) % len;
    (i, j)
}

fn ga_step(
    mut population: Vec<NdarrayController>,
    generation: u64,
    settings: &GASettings,
) -> Vec<NdarrayController> {
    let rewards = evaluate_population(&mut population, generation);
    let avg_reward = rewards.iter().sum::<f32>() / rewards.len() as f32;

    let mut pop_with_rewards: Vec<_> =
        std::iter::zip(population.into_iter(), rewards.into_iter()).collect();
    pop_with_rewards.sort_by(|(_, a), (_, b)| b.total_cmp(a));

    println!(
        "{}: Best: {}, average: {}",
        generation, pop_with_rewards[0].1, avg_reward
    );

    pop_with_rewards[0]
        .0
        .save(&format!("controllers/{generation:05}.json"));

    let mut new_generation: Vec<_> = pop_with_rewards
        .iter()
        .take(settings.elite)
        .map(|(ctrl, _)| ctrl.clone())
        .collect();

    let parents_pool: Vec<_> = pop_with_rewards
        .iter()
        .take(settings.parents)
        .map(|(ctrl, _)| ctrl)
        .collect();

    let mut rng = rand::rng();
    for _ in 0..settings.selected_pairs {
        let (first, second) = random_pair(settings.parents, &mut rng);

        for ratio in ndarray::linspace(0.1, 0.9, settings.childred_per_pair) {
            let mut_coef = 1.0; // rng.random::<f32>();
            new_generation.push(cross(
                parents_pool[first],
                parents_pool[second],
                ratio,
                mut_coef * settings.mutation,
            ));
        }
    }

    new_generation
}

fn main() {
    let settings = GASettings::default();
    let init_size = settings.elite + settings.selected_pairs * settings.childred_per_pair;
    let mut population: Vec<_> = (0..init_size)
        .into_iter()
        .map(|_| NdarrayController::random())
        .collect();

    for generation in 0.. {
        population = ga_step(population, generation, &settings);
    }
}
