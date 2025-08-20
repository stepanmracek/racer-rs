use crate::{
    controller::{Controller, KeyboardController},
    environment::{Environment, SensorReadings},
    follow_camera::FollowCamera,
    states::{State, finish::Finish},
    utils::format_time,
};
use macroquad::prelude::*;

pub struct Game {
    follow_camera: FollowCamera,
    state_started: f64,
    controller: Box<dyn Controller>,
    // onnx_controller: Box<dyn Controller>,
}

impl Game {
    pub fn new(follow_camera: &FollowCamera) -> Self {
        let follow_camera = follow_camera.clone();
        Self {
            follow_camera,
            state_started: get_time(),
            controller: Box::new(KeyboardController {}),
            // onnx_controller: Box::new(OnnxController::new("research/model.onnx")),
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

    fn draw_sensors(sensors: &SensorReadings) {
        for (d, (start, end)) in sensors.distances.iter().zip(sensors.rays.iter()) {
            draw_line(start.x, start.y, end.x, end.y, 0.3, GREEN.with_alpha(0.2));
            if let Some(d) = d {
                let p = (*end - *start).normalize() * *d + *start;
                draw_circle(p.x, p.y, 1.0, RED);
            }
        }
    }
}

impl State for Game {
    fn step(&mut self, environment: &mut Environment) -> Option<Box<dyn State>> {
        //let _action = self.onnx_controller.control(&environment.observation);
        let action = self.controller.control(&environment.observation);

        let outcome = environment.step(&action, false);

        /*if is_key_pressed(KeyCode::Space) {
            let nearest_segment = &environment
                .track
                .nearest_segments(environment.car.position(), 1)[0];
            environment.car.reset(
                &nearest_segment.start.pos,
                nearest_segment.start.dir.to_angle(),
                0.0,
            );
        }*/

        if outcome.finished {
            Some(Box::new(Finish::new(
                &self.follow_camera,
                self.current_time(),
            )))
        } else {
            None
        }
    }

    fn draw(&mut self, environment: &Environment) {
        environment.draw(&mut self.follow_camera);
        Game::draw_sensors(&environment.observation.sensors);
        self.draw_stopwatch();
    }
}
