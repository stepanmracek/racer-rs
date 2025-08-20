use racer_logic::{
    controller::{Controller, OnnxController},
    environment::Environment,
};

fn main() {
    let mut env = Environment::new();
    let mut controller = OnnxController::new("research/model.onnx");

    loop {
        let action = controller.control(&env.observation);
        println!("{:?} -> {:?}", env.observation, action);
        let output = env.step(&action);
        if output.finished {
            break;
        }
    }
}
