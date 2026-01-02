use crate::{
    controller::Controller,
    environment::{Action, Observation},
};

#[derive(Default)]
pub struct NonController {}

impl Controller for NonController {
    fn control(&mut self, _observation: &Observation) -> Action {
        Action {
            steer: 0.0,
            throttle: 0.0,
        }
    }
}
