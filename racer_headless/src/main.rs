use clap::Parser;
use kdam::tqdm;
use racer_logic::{controller::Controller, environment::EnvironmentBuilder};
use racer_ndarray_controller::NdarrayController;
use racer_onnx_controller::{ActionSelectionStrategy, OnnxController};
use racer_pid_controller::PidController;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, value_name = "ONNX_MODEL_PATH")]
    onnx: Option<String>,

    #[arg(long, value_name = "NDARRAY_MODEL_PATH")]
    ndarray: Option<String>,

    #[arg(long, value_name = "PID_MODEL")]
    pid: Option<String>,
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
    } else if let Some(path) = args.pid {
        Box::new(PidController::load(&path))
    } else {
        panic!("No controller selected")
    }
}

fn main() {
    let mut controller = controller();

    let mut finish_count = 0;
    let mut fail_count = 0;
    for i in tqdm!(0..1_000) {
        let mut env = EnvironmentBuilder::default()
            .with_seed(Some(i))
            .build(1)
            .unwrap();
        controller.reset();
        for _ in 0..100 * 60 {
            let action = controller.control(&env.observations[0]);
            let output = &env.step(&[action], true)[0];
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
    eprintln!("Finish: {finish_count}");
    eprintln!("Fails: {fail_count}");
    eprintln!("No finish: {}", 1000 - (finish_count + fail_count));
}
