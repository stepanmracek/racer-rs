use crate::{
    controller::Controller,
    environment::Environment,
    follow_camera::FollowCamera,
    states::{State, game::Game},
};
use macroquad::prelude::*;

pub struct Init {
    follow_camera: FollowCamera,
    controller_factories: Vec<fn() -> Box<dyn Controller>>,
}

impl Init {
    pub fn new(
        environment: &Environment,
        controller_factories: Vec<fn() -> Box<dyn Controller>>,
    ) -> Self {
        let follow_camera = FollowCamera::new(&environment.cars[0]);
        Self {
            follow_camera,
            controller_factories,
        }
    }
}

impl State for Init {
    fn step(&mut self, _environment: &mut Environment) -> Option<Box<dyn State>> {
        if is_key_pressed(KeyCode::Space) {
            Some(Box::new(Game::new(
                &self.follow_camera,
                &self.controller_factories,
            )))
        } else {
            None
        }
    }

    fn draw(&mut self, environment: &Environment) {
        environment.draw(&mut self.follow_camera);

        set_default_camera();
        draw_text("Press space to start", 5.0, 24.0, 32.0, WHITE);
    }
}
