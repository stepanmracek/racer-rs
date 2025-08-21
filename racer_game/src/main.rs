use macroquad::prelude::*;
use racer_logic::{
    environment::Environment,
    states::{Init, State},
};
use std::time::{SystemTime, UNIX_EPOCH};

fn window_conf() -> Conf {
    Conf {
        window_title: "racer".to_owned(),
        fullscreen: true,
        sample_count: 2,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64;
    let mut environment = Environment::new(seed);
    environment.car.load_texture().await;
    let mut state: Box<dyn State> = Box::new(Init::new(&environment));

    loop {
        if let Some(next_state) = state.step(&mut environment) {
            state = next_state;
        }

        state.draw(&environment);

        next_frame().await;
    }
}
