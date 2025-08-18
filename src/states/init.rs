use crate::{
    follow_camera::FollowCamera,
    states::{State, game::Game},
    world::World,
};
use macroquad::prelude::*;

pub struct Init {
    follow_camera: FollowCamera,
}

impl Init {
    pub fn new(world: &World) -> Self {
        let follow_camera = FollowCamera::new(&world.car);
        Self { follow_camera }
    }
}

impl State for Init {
    fn step(&mut self, _world: &mut World) -> Option<Box<dyn State>> {
        if is_key_pressed(KeyCode::Space) {
            Some(Box::new(Game::new(&self.follow_camera)))
        } else {
            None
        }
    }

    fn draw(&mut self, world: &World) {
        world.draw(&mut self.follow_camera);

        set_default_camera();
        draw_text("Press space to start", 5.0, 24.0, 32.0, WHITE);
    }
}
