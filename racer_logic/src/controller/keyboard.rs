use crate::{
    controller::Controller,
    environment::{Action, Observation},
};
use macroquad::prelude::*;

pub struct KeyboardController {}

impl Controller for KeyboardController {
    fn control(&mut self, _observation: &Observation) -> Action {
        let steer =
            ((is_key_down(KeyCode::Left) as i32) - (is_key_down(KeyCode::Right) as i32)) as f32;
        let throttle =
            ((is_key_down(KeyCode::Up) as i32) - (is_key_down(KeyCode::Down) as i32)) as f32;

        Action { steer, throttle }
    }
}
