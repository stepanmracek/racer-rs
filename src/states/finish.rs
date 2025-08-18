use crate::{follow_camera::FollowCamera, states::State, utils::format_time, world::World};
use macroquad::prelude::*;

pub struct Finish {
    follow_camera: FollowCamera,
    result_time: f64,
}

impl Finish {
    pub fn new(follow_camera: &FollowCamera, result_time: f64) -> Self {
        let follow_camera = follow_camera.clone();
        Self {
            follow_camera,
            result_time,
        }
    }
}

impl State for Finish {
    fn step(&mut self, _world: &mut World) -> Option<Box<dyn State>> {
        None
    }

    fn draw(&mut self, world: &World) {
        world.draw(&mut self.follow_camera);

        set_default_camera();
        let time = format_time(self.result_time);
        draw_text(&format!("FINISH: {time}"), 5.0, 24.0, 32.0, WHITE);
    }
}
