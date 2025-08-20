mod keyboard;
mod onnx;

pub use keyboard::KeyboardController;
pub use onnx::OnnxController;

#[derive(Debug)]
pub struct Control {
    pub steer: f32,
    pub throttle: f32,
}

pub trait Controller {
    fn control(
        &mut self,
        velocity: f32,
        steering_angle: f32,
        wheels_on_track: &[bool; 4],
        sensor_readings: &[Option<f32>],
    ) -> Control;
}
