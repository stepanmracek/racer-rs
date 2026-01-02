use kdam::tqdm;
use racer_logic::{controller::Controller, environment::Environment};
use racer_onnx_controller::{ActionSelectionStrategy, OnnxController};

fn main() {
    let onnx_path = std::env::args().nth(1).unwrap();
    let mut controller = OnnxController::new(&onnx_path, ActionSelectionStrategy::Greedy);

    let gamma = 0.99;
    let mut finish_count = 0;
    for _ in tqdm!(0..10_000) {
        let mut env = Environment::default();
        let mut rewards = vec![];
        for _ in 0..10 * 60 {
            let action = controller.control(&env.observations[0]);
            let output = env.step(&[action], true);
            rewards.push(output[0].reward);
            if output[0].finished {
                print!("Finished!: ");
                finish_count += 1;
                break;
            }
        }
        let mut discounted_reward: Vec<f32> = rewards
            .iter()
            .rev()
            .scan(0.0, |acc, r| {
                *acc = *acc * gamma + r;
                Some(*acc)
            })
            .collect();
        discounted_reward.reverse();
        assert_eq!(discounted_reward.len(), rewards.len());
        println!("{discounted_reward:?}");
    }
    eprintln!("Finished episodes: {finish_count}");
}
