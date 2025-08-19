use crate::controller::{Control, Controller};
use macroquad::prelude::*;

pub struct KeyboardController {}

impl Controller for KeyboardController {
    fn control(
        &mut self,
        _velocity: f32,
        _steering_angle: f32,
        _wheels_on_track: &[bool; 4],
        _sensor_readings: &[Option<f32>],
    ) -> Control {
        let steer =
            ((is_key_down(KeyCode::Left) as i32) - (is_key_down(KeyCode::Right) as i32)) as f32;
        let throttle =
            ((is_key_down(KeyCode::Up) as i32) - (is_key_down(KeyCode::Down) as i32)) as f32;

        Control { steer, throttle }
    }
}
