use crate::environment::{Action, Observation};
mod keyboard;
pub use keyboard::KeyboardController;

pub trait Controller {
    fn control(&mut self, observation: &Observation) -> Action;
}
