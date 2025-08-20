use macroquad::prelude::*;

mod car;
mod controller;
mod environment;
mod follow_camera;
mod physics;
mod states;
mod track;
mod utils;

#[macroquad::main("racer")]
async fn main() {
    let mut environment = environment::Environment::new().await;
    let mut state: Box<dyn states::State> = Box::new(states::Init::new(&environment));

    loop {
        if let Some(next_state) = state.step(&mut environment) {
            state = next_state;
        }

        state.draw(&environment);

        next_frame().await;
    }
}
