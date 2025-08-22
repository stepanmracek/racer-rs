use kdam::tqdm;
use racer_logic::{
    controller::{Controller, OnnxController},
    environment::Environment,
};

fn main() {
    let mut controller = OnnxController::new("research/model.onnx");

    let gamma = 0.99;
    let mut finish_count = 0;
    for _ in tqdm!(0..10_000) {
        let mut env = Environment::new(None);
        let mut rewards = vec![];
        for _ in 0..10 * 60 {
            let action = controller.control(&env.observation);
            let output = env.step(&action, true);
            rewards.push(output.reward);
            if output.finished {
                print!("Finished!: ");
                finish_count += 1;
                break;
            }
        }
        let last_reward = rewards[rewards.len() - 1];
        let discounted_reward = rewards
            .into_iter()
            .rev()
            .reduce(|acc, r| acc * gamma + r)
            .unwrap_or_default();
        println!("{discounted_reward} {}", last_reward);
    }
    eprintln!("Finished episodes: {finish_count}");
}
