use crate::environment::{Action, Observation};
mod keyboard;
pub use keyboard::{KeyboardArrowsController, KeyboardWASDController};

pub trait Controller {
    fn control(&mut self, observation: &Observation) -> Action;
    fn reset(&mut self);
}
