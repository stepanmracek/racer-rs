use crate::{controller::Controller, environment::Action, observation::Observation};
use macroquad::prelude::*;

#[derive(Default)]
pub struct KeyboardArrowsController {}

impl Controller for KeyboardArrowsController {
    fn control(&mut self, _observation: &Observation) -> Action {
        let steer =
            ((is_key_down(KeyCode::Left) as i32) - (is_key_down(KeyCode::Right) as i32)) as f32;
        let throttle =
            ((is_key_down(KeyCode::Up) as i32) - (is_key_down(KeyCode::Down) as i32)) as f32;

        Action::new(steer, throttle)
    }

    fn reset(&mut self) {}
}

#[derive(Default)]
pub struct KeyboardWASDController {}

impl Controller for KeyboardWASDController {
    fn control(&mut self, _observation: &Observation) -> Action {
        let steer = ((is_key_down(KeyCode::A) as i32) - (is_key_down(KeyCode::D) as i32)) as f32;
        let throttle = ((is_key_down(KeyCode::W) as i32) - (is_key_down(KeyCode::S) as i32)) as f32;

        Action::new(steer, throttle)
    }

    fn reset(&mut self) {}
}
