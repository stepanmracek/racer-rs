use macroquad::prelude::*;
use racer_logic::{
    controller::{Controller, KeyboardController},
    environment::Environment,
    states::{Init, State},
};
use racer_onnx_controller::OnnxController;

fn window_conf() -> Conf {
    Conf {
        window_title: "racer".to_owned(),
        //fullscreen: true,
        sample_count: 2,
        ..Default::default()
    }
}

fn controller_factory() -> Box<dyn Controller> {
    if let Some(path) = std::env::args().nth(1) {
        Box::new(OnnxController::new(&path))
    } else {
        Box::new(KeyboardController::default())
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut environment = Environment::new(None);
    environment.car.load_texture().await;
    let mut state: Box<dyn State> = Box::new(Init::new(&environment, controller_factory));

    loop {
        if let Some(next_state) = state.step(&mut environment) {
            state = next_state;
        }

        state.draw(&environment);

        next_frame().await;
    }
}
