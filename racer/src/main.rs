use clap::Parser;
use macroquad::prelude::*;
use racer_logic::{
    controller::{Controller, KeyboardController},
    environment::EnvironmentBuilder,
    states::{Init, State},
};
use racer_ndarray_controller::NdarrayController;
use racer_onnx_controller::{ActionSelectionStrategy, OnnxController};
use racer_pid_controller::PidController;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, value_name = "ENV_SEED")]
    seed: Option<u64>,

    #[arg(long, value_name = "ONNX_MODEL_PATH")]
    onnx: Option<String>,

    #[arg(long, value_name = "NDARRAY_MODEL_PATH")]
    ndarray: Option<String>,

    #[arg(long, value_name = "PID_MODEL")]
    pid: Option<String>,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "racer".to_owned(),
        //fullscreen: true,
        sample_count: 2,
        window_resizable: true,
        ..Default::default()
    }
}

fn controller(args: &Args) -> Box<dyn Controller> {
    if let Some(path) = &args.onnx {
        Box::new(OnnxController::new(
            path,
            ActionSelectionStrategy::Stochastic,
        ))
    } else if let Some(path) = &args.ndarray {
        Box::new(NdarrayController::load(path))
    } else if let Some(path) = &args.pid {
        Box::new(PidController::load(path))
    } else {
        Box::new(KeyboardController::default())
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args = Args::parse();
    let mut environment = EnvironmentBuilder::default().with_seed(args.seed).build();
    environment.car.load_texture().await;
    let mut state: Box<dyn State> = Box::new(Init::new(&environment, controller(&args)));

    loop {
        if let Some(next_state) = state.step(&mut environment) {
            state = next_state;
        }

        state.draw(&environment);

        next_frame().await;
    }
}
