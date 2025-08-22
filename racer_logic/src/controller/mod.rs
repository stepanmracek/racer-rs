mod keyboard;
mod onnx;

pub use keyboard::KeyboardController;
pub use onnx::OnnxController;

use crate::environment::{Action, Observation};

pub trait Controller {
    fn control(&mut self, observation: &Observation) -> Action;
}

pub fn controller_factory() -> Box<dyn Controller> {
    if let Some(path) = std::env::args().nth(1) {
        Box::new(OnnxController::new(&path))
    } else {
        Box::new(KeyboardController {})
    }
}
