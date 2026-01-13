use kdam::tqdm;
use racer_logic::{
    controller::Controller,
    environment::{Environment, ReachFinish},
};
use racer_onnx_controller::{ActionSelectionStrategy, OnnxController};

fn main() {
    let onnx_path = std::env::args().nth(1).unwrap();
    let mut controller = OnnxController::new(&onnx_path, ActionSelectionStrategy::Greedy);

    let mut finish_count = 0;
    let mut fail_count = 0;
    for _ in tqdm!(0..1_000) {
        let mut env = Environment::new(None, 0.0, Box::new(ReachFinish::default()));
        for _ in 0..120 * 60 {
            let action = controller.control(&env.observation);
            let output = env.step(&action, true);
            if output.terminated {
                if !output.truncated {
                    finish_count += 1;
                } else {
                    fail_count += 1;
                }
                break;
            }
        }
    }
    eprintln!("Wins: {finish_count}");
    eprintln!("Losses: {fail_count}");
}
