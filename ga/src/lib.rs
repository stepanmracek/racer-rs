use kdam::tqdm;
use rand::Rng;
use serde::Serialize;

pub struct Settings {
    pub elite: usize,
    pub parents: usize,
    pub selected_pairs: usize,
    pub childred_per_pair: usize,
    pub mutation: f32,
}

impl Default for Settings {
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

impl Settings {
    pub fn init_size(&self) -> usize {
        self.elite + self.selected_pairs * self.childred_per_pair
    }
}

fn random_pair(len: usize, rng: &mut impl Rng) -> (usize, usize) {
    let i = rng.random_range(0..len);
    let offset = rng.random_range(1..len);
    let j = (i + offset) % len;
    (i, j)
}

fn linspace(start: f32, end: f32, num: usize) -> Vec<f32> {
    if num == 0 {
        return vec![];
    }
    if num == 1 {
        return vec![start];
    }

    let step = (end - start) / (num - 1) as f32;
    (0..num).map(|i| start + (i as f32 * step)).collect()
}

pub trait Individual {
    fn evaluate(&mut self, env_seed: u64) -> f32;
    fn cross(&self, other: &Self, ratio: f32, mutation: f32) -> Self;
}

fn evaluate_population<I>(population: &mut [I], generation: u64) -> Vec<f32>
where
    I: Individual,
{
    tqdm!(
        population.iter_mut(),
        desc = format!("Generation: {generation}")
    )
    .map(|i| i.evaluate(generation))
    .collect()
}

pub fn step<I>(
    mut population: Vec<I>,
    settings: &Settings,
    generation: u64,
    save_best: Option<&str>,
) -> Vec<I>
where
    I: Individual + Clone + Serialize,
{
    let rewards = evaluate_population(&mut population, generation);
    let avg_reward = rewards.iter().sum::<f32>() / rewards.len() as f32;

    let mut pop_with_rewards: Vec<_> = std::iter::zip(population, rewards).collect();
    pop_with_rewards.sort_by(|(_, a), (_, b)| b.total_cmp(a));

    println!("{},{},{}", generation, pop_with_rewards[0].1, avg_reward);

    if let Some(path) = save_best {
        let path = format!("{path}/{generation:05}.json");
        let json = serde_json::to_string_pretty(&pop_with_rewards[0].0).unwrap();
        std::fs::write(path, json).unwrap();
    }

    let mut new_generation: Vec<_> = pop_with_rewards
        .iter()
        .take(settings.elite)
        .map(|(i, _)| i.clone())
        .collect();

    let parents_pool: Vec<_> = pop_with_rewards
        .iter()
        .take(settings.parents)
        .map(|(ctrl, _)| ctrl)
        .collect();

    let mut rng = rand::rng();
    for _ in 0..settings.selected_pairs {
        let (first, second) = random_pair(settings.parents, &mut rng);

        for ratio in linspace(0.1, 0.9, settings.childred_per_pair) {
            let mut_coef = 1.0; // rng.random::<f32>();
            new_generation.push(parents_pool[first].cross(
                parents_pool[second],
                ratio,
                mut_coef * settings.mutation,
            ));
        }
    }

    new_generation
}
