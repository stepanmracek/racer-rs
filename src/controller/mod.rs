mod keyboard;
mod onnx;

pub use keyboard::KeyboardController;
pub use onnx::OnnxController;

use crate::environment::{Action, Observation};

pub trait Controller {
    fn control(&mut self, observation: &Observation) -> Action;
}
