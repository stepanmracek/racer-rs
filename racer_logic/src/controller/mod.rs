use crate::environment::{Action, Observation};
mod keyboard;
mod touch;
pub use keyboard::KeyboardController;
pub use touch::TouchController;

pub trait Controller {
    fn control(&mut self, observation: &Observation) -> Action;
}
