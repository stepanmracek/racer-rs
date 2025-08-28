use crate::{
    controller::Controller,
    environment::{Action, Observation},
};
use macroquad::prelude::*;

#[derive(Default)]
pub struct TouchController {}

impl Controller for TouchController {
    fn control(&mut self, _observation: &Observation) -> Action {
        let mut steer = 0.0;
        let mut throttle = 0.0;
        if is_mouse_button_down(MouseButton::Left) {
            let (pos_x, pos_y) = mouse_position();
            let right = pos_x > screen_width() / 2.0;
            let brake = pos_y > screen_height() / 2.0;
            steer = if right { -1.0 } else { 1.0 };
            throttle = if brake { -1.0 } else { 1.0 }
        }

        Action { steer, throttle }
    }
}
