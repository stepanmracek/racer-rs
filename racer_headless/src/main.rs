use clap::Parser;
use kdam::tqdm;
use racer_logic::{
    controller::Controller,
    environment::{Environment, ReachFinish},
};
use racer_ndarray_controller::NdarrayController;
use racer_onnx_controller::{ActionSelectionStrategy, OnnxController};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, value_name = "ONNX_MODEL_PATH")]
    onnx: Option<String>,

    #[arg(long, value_name = "NDARRAY_MODEL_PATH")]
    ndarray: Option<String>,
}

fn controller() -> Box<dyn Controller> {
    let args = Args::parse();

    if let Some(path) = args.onnx {
        Box::new(OnnxController::new(
            &path,
            ActionSelectionStrategy::Stochastic,
        ))
    } else if let Some(path) = args.ndarray {
        Box::new(NdarrayController::load(&path))
    } else {
        panic!("No controller selected")
    }
}

fn main() {
    let mut controller = controller();

    let mut finish_count = 0;
    let mut fail_count = 0;
    for i in tqdm!(0..1_000) {
        let mut env = Environment::new(Some(i), 0.0, Box::new(ReachFinish::default()));
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
