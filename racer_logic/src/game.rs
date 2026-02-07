use crate::{
    controller::Controller,
    environment::Environment,
    follow_camera::FollowCamera,
    states::{Init, State},
};

pub struct Game {
    pub environment: Environment,
    pub controllers: Vec<Box<dyn Controller>>,
    pub follow_camera: FollowCamera,
    state: Box<dyn State>,
}

impl Game {
    pub fn new(environment: Environment, controllers: Vec<Box<dyn Controller>>) -> Self {
        let follow_camera = FollowCamera::new(&environment.cars[0]);
        Self {
            environment,
            controllers,
            follow_camera,
            state: Box::new(Init::default()),
        }
    }

    pub fn step(&mut self) {
        if let Some(next_state) = self.state.step(
            &mut self.environment,
            &mut self.controllers,
            &mut self.follow_camera,
        ) {
            self.state = next_state;
        }

        self.state.draw(&self.environment, &mut self.follow_camera);
    }
}
