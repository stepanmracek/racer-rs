use crate::{follow_camera::FollowCamera, states::State, world::World};
use macroquad::prelude::*;

pub struct Finish {
    follow_camera: FollowCamera,
}

impl Finish {
    pub fn new(follow_camera: &FollowCamera) -> Self {
        let follow_camera = follow_camera.clone();
        Self { follow_camera }
    }
}

impl State for Finish {
    fn step(&mut self, _world: &mut World) -> Option<Box<dyn State>> {
        None
    }

    fn draw(&mut self, world: &World) {
        clear_background(DARKGREEN);
        self.follow_camera.update(&world.car);
        world.track.draw(&world.car);
        world.car.draw();

        set_default_camera();
        draw_text("FINISH!", 5.0, 24.0, 32.0, WHITE);
    }
}
