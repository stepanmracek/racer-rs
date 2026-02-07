use crate::{
    controller::Controller, environment::Environment, follow_camera::FollowCamera, states::State,
    utils::format_time,
};
use macroquad::prelude::*;

pub struct Finish {
    result_time: f64,
}

impl Finish {
    pub fn new(result_time: f64) -> Self {
        Self { result_time }
    }
}

impl State for Finish {
    fn step(
        &mut self,
        _environment: &mut Environment,
        _controllers: &mut [Box<dyn Controller>],
        _follow_camera: &mut FollowCamera,
    ) -> Option<Box<dyn State>> {
        None
    }

    fn draw(&mut self, environment: &Environment, follow_camera: &mut FollowCamera) {
        environment.draw(follow_camera);

        set_default_camera();
        let time = format_time(self.result_time);
        draw_text(&format!("FINISH: {time}"), 5.0, 24.0, 32.0, WHITE);
    }
}
