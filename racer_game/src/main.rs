use macroquad::prelude::*;
use racer_logic::{
    controller::OnnxController,
    environment::Environment,
    states::{Init, State},
};

fn window_conf() -> Conf {
    Conf {
        window_title: "racer".to_owned(),
        //fullscreen: true,
        sample_count: 2,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut environment = Environment::new(None);
    environment.car.load_texture().await;
    let mut state: Box<dyn State> = Box::new(Init::new(&environment, || {
        Box::new(OnnxController::new("research/model.onnx"))
    }));

    loop {
        if let Some(next_state) = state.step(&mut environment) {
            state = next_state;
        }

        state.draw(&environment);

        next_frame().await;
    }
}
