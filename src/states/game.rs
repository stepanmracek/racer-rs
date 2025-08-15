use crate::{
    follow_camera::FollowCamera,
    states::{State, finish::Finish},
    world::World,
};
use macroquad::prelude::*;

pub struct Game {
    follow_camera: FollowCamera,
}

impl Game {
    pub fn new(world: &World) -> Self {
        let follow_camera = FollowCamera::new(&world.car);
        Self { follow_camera }
    }

    fn draw_stopwatch(&self) {
        set_default_camera();
        let time = (get_time() * 100.0) as usize;
        let hundrets = time % 100;
        let seconds = (time / 100) % 60;
        let minutes = time / 6000;
        let stopwatch = format!("{minutes:02}:{seconds:02}:{hundrets:02}");
        draw_text(&stopwatch, 5.0, 24.0, 32.0, WHITE);
    }
}

impl State for Game {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        clear_background(DARKGREEN);

        let wheels_on_track = world.car.wheels_on_track(&world.track);
        world.car.update(&wheels_on_track);
        self.follow_camera.update(&world.car);
        world.track.draw(&world.car);
        world.car.draw();
        self.draw_stopwatch();

        if world.track.finish(world.car.bbox()) {
            Some(Box::new(Finish {}))
        } else {
            None
        }
    }
}
