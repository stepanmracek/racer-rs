use crate::{
    states::{State, game::Game},
    world::World,
};
use macroquad::prelude::*;

pub struct Init {}

impl State for Init {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        set_default_camera();
        draw_text("Press space to start", 5.0, 24.0, 32.0, WHITE);
        if is_key_pressed(KeyCode::Space) {
            Some(Box::new(Game::new(world)))
        } else {
            None
        }
    }
}
