use macroquad::prelude::*;

mod car;
mod controller;
mod follow_camera;
mod physics;
mod states;
mod track;
mod utils;
mod world;

#[macroquad::main("racer")]
async fn main() {
    let mut world = world::World::new().await;
    let mut state: Box<dyn states::State> = Box::new(states::Init::new(&world));

    loop {
        if let Some(next_state) = state.step(&mut world) {
            state = next_state;
        }

        state.draw(&world);

        next_frame().await;
    }
}
