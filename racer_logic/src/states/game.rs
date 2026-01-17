use crate::{
    car::Car,
    controller::Controller,
    environment::{Environment, Observation},
    follow_camera::FollowCamera,
    states::{State, finish::Finish},
    utils::format_time,
};
use macroquad::prelude::*;
use std::iter::zip;

pub struct Game {
    follow_camera: FollowCamera,
    state_started: f64,
    controller: Box<dyn Controller>,
    reward: f32,
}

impl Game {
    pub fn new(follow_camera: &FollowCamera, controller: Box<dyn Controller>) -> Self {
        let follow_camera = follow_camera.clone();
        let mut game = Self {
            follow_camera,
            state_started: get_time(),
            controller,
            reward: 0.0,
        };

        game.controller.reset();
        game
    }

    fn draw_stopwatch(&self) {
        set_default_camera();
        let stopwatch = format_time(self.current_time());
        draw_text(&stopwatch, 5.0, 24.0, 32.0, WHITE);
    }

    fn current_time(&self) -> f64 {
        get_time() - self.state_started
    }

    fn draw_observation(observation: &Observation, _car: &Car, reward: f32) {
        for (i, (d, (start, end))) in
            zip(&observation.sensors.distances, &observation.sensors.rays).enumerate()
        {
            draw_line(start.x, start.y, end.x, end.y, 0.3, GREEN.with_alpha(0.2));
            if let Some(d) = d {
                let p = (*end - *start).normalize() * *d + *start;

                if i == 6 {
                    draw_circle(p.x, p.y, 2.0, RED);
                } else {
                    draw_circle(p.x, p.y, 1.0, RED);
                }
            }
        }

        push_camera_state();
        set_default_camera();
        draw_multiline_text(
            &format!("speed: {:.2}\nreward: {reward:.2}", observation.velocity),
            screen_width() * 0.5,
            screen_height() * 0.5,
            24.0,
            None,
            YELLOW.with_alpha(0.8),
        );
        pop_camera_state();
    }
}

impl State for Game {
    fn step(&mut self, environment: &mut Environment) -> Option<Box<dyn State>> {
        let action = self.controller.control(&environment.observation);

        let outcome = environment.step(&action, false);
        self.reward += outcome.reward;

        if is_key_pressed(KeyCode::Space) {
            let nearest_segment = &environment
                .track
                .nearest_segments(environment.car.position(), 1)[0];
            environment.car.reset(
                &nearest_segment.start.pos,
                nearest_segment.start.dir.to_angle(),
                0.0,
            );
            self.controller.reset();
        }

        if outcome.terminated {
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
        Game::draw_observation(&environment.observation, &environment.car, self.reward);
        self.draw_stopwatch();
    }
}
