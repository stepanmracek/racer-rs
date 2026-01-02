use crate::environment::{Action, Observation};
mod keyboard;
mod none;
pub use keyboard::KeyboardController;
pub use none::NonController;

pub trait Controller {
    fn control(&mut self, observation: &Observation) -> Action;
}
