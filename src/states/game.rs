use crate::{
    follow_camera::FollowCamera,
    states::{State, finish::Finish},
    utils::format_time,
    world::World,
};
use macroquad::prelude::*;

pub struct Game {
    follow_camera: FollowCamera,
    state_started: f64,
}

impl Game {
    pub fn new(follow_camera: &FollowCamera) -> Self {
        let follow_camera = follow_camera.clone();
        Self {
            follow_camera,
            state_started: get_time(),
        }
    }

    fn draw_stopwatch(&self) {
        set_default_camera();
        let stopwatch = format_time(self.current_time());
        draw_text(&stopwatch, 5.0, 24.0, 32.0, WHITE);
    }

    fn current_time(&self) -> f64 {
        get_time() - self.state_started
    }
}

impl State for Game {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        let wheels_on_track = world.car.wheels_on_track(&world.track);
        world.car.update(&wheels_on_track);

        if world.track.finish(world.car.bbox()) {
            Some(Box::new(Finish::new(
                &self.follow_camera,
                self.current_time(),
            )))
        } else {
            None
        }
    }

    fn draw(&mut self, world: &World) {
        world.draw(&mut self.follow_camera);

        self.draw_stopwatch();
    }
}
