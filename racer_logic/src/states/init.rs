use crate::{
    controller::Controller,
    environment::Environment,
    follow_camera::FollowCamera,
    states::{State, race::Race},
};
use macroquad::prelude::*;

#[derive(Default)]
pub struct Init {}

impl State for Init {
    fn step(
        &mut self,
        _environment: &mut Environment,
        controllers: &mut [Box<dyn Controller>],
        _follow_camera: &mut FollowCamera,
    ) -> Option<Box<dyn State>> {
        if is_key_pressed(KeyCode::Space) {
            controllers.iter_mut().for_each(|c| c.reset());
            Some(Box::new(Race::new()))
        } else {
            None
        }
    }

    fn draw(&mut self, environment: &Environment, follow_camera: &mut FollowCamera) {
        environment.draw(follow_camera);

        set_default_camera();
        draw_text("Press space to start", 5.0, 24.0, 32.0, WHITE);
    }
}
