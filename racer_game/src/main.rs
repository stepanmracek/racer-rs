use macroquad::prelude::*;
use racer_logic::{
    environment::Environment,
    states::{Init, State},
};

#[macroquad::main("racer")]
async fn main() {
    let mut environment = Environment::new().await;
    let mut state: Box<dyn State> = Box::new(Init::new(&environment));

    loop {
        if let Some(next_state) = state.step(&mut environment) {
            state = next_state;
        }

        state.draw(&environment);

        next_frame().await;
    }
}
