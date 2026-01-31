use clap::Parser;
use macroquad::prelude::*;
use racer_logic::{
    controller::{Controller, KeyboardArrowsController, KeyboardWASDController},
    environment::EnvironmentBuilder,
    states::{Init, State},
};
use racer_ndarray_controller::NdarrayController;
use racer_onnx_controller::{ActionSelectionStrategy, OnnxController};
use racer_pid_controller::PidController;
use std::str::FromStr;

#[derive(Debug, Clone)]
enum ControllerArg {
    Onnx(String),
    Pid(String),
    Ndarray(String),
    KeyboardArrows,
    KeyboardWasd,
}

impl FromStr for ControllerArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((ctype, path)) = s.split_once(':') {
            match ctype {
                "onnx" => Ok(ControllerArg::Onnx(path.to_string())),
                "pid" => Ok(ControllerArg::Pid(path.to_string())),
                "ndarray" => Ok(ControllerArg::Ndarray(path.to_string())),
                other => Err(format!("unknown controller type '{}'", other)),
            }
        } else {
            match s {
                "arrows" => Ok(ControllerArg::KeyboardArrows),
                "wasd" => Ok(ControllerArg::KeyboardWasd),
                other => Err(format!(
                    "unknown controller type '{}' or missing path, use format <type>:<path>",
                    other
                )),
            }
        }
    }
}

impl ControllerArg {
    fn build(&self) -> Box<dyn Controller> {
        let controller: Box<dyn Controller> = match self {
            ControllerArg::Onnx(path) => Box::new(OnnxController::new(
                path,
                ActionSelectionStrategy::Stochastic,
            )),
            ControllerArg::Pid(path) => Box::new(PidController::load(path)),
            ControllerArg::Ndarray(path) => Box::new(NdarrayController::load(path)),
            ControllerArg::KeyboardArrows => Box::new(KeyboardArrowsController::default()),
            ControllerArg::KeyboardWasd => Box::new(KeyboardWASDController::default()),
        };
        controller
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, value_name = "ENV_SEED")]
    seed: Option<u64>,

    #[arg(long, value_name = "CONTROLLER")]
    controller: Vec<ControllerArg>,
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

#[macroquad::main(window_conf)]
async fn main() {
    let args = Args::parse();
    let controllers = args
        .controller
        .into_iter()
        .map(|arg| arg.build())
        .collect::<Vec<_>>();

    let mut environment = EnvironmentBuilder::default()
        .with_seed(args.seed)
        .build(controllers.len())
        .unwrap();

    for (i, car) in environment.cars.iter_mut().enumerate() {
        car.load_texture(i + 1).await;
    }
    let mut state: Box<dyn State> = Box::new(Init::new(&environment, controllers));

    loop {
        if let Some(next_state) = state.step(&mut environment) {
            state = next_state;
        }

        state.draw(&environment);

        next_frame().await;
    }
}
