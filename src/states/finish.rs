use crate::{states::State, world::World};
use macroquad::prelude::*;

pub struct Finish {}

impl State for Finish {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        set_default_camera();
        draw_text("FINISH!", 5.0, 24.0, 32.0, WHITE);
        None
    }
}
