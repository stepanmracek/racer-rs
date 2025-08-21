use kdam::tqdm;
use racer_logic::{
    controller::{Controller, OnnxController},
    environment::Environment,
};

fn main() {
    let mut env = Environment::new(None);
    let mut controller = OnnxController::new("research/model.onnx");

    for _ in tqdm!(0..1_000) {
        let action = controller.control(&env.observation);
        let output = env.step(&action, true);
        if output.finished {
            println!("Finish reached!");
            break;
        }
    }
}
