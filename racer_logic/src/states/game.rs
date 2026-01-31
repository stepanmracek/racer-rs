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
    controllers: Vec<Box<dyn Controller>>,
    reward: f32,
    show_observation: bool,
}

impl Game {
    pub fn new(follow_camera: &FollowCamera, controllers: Vec<Box<dyn Controller>>) -> Self {
        let follow_camera = follow_camera.clone();
        let mut game = Self {
            follow_camera,
            state_started: get_time(),
            controllers,
            reward: 0.0,
            show_observation: false,
        };

        game.controllers.iter_mut().for_each(|c| c.reset());
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
        for (d, (start, end)) in zip(&observation.sensors.distances, &observation.sensors.rays) {
            draw_line(start.x, start.y, end.x, end.y, 0.3, GREEN.with_alpha(0.2));
            if let Some(d) = d {
                let p = (*end - *start).normalize() * *d + *start;
                draw_circle(p.x, p.y, 1.0, RED);
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
        let actions = std::iter::zip(self.controllers.iter_mut(), environment.observations.iter())
            .map(|(controller, observation)| controller.control(observation))
            .collect::<Vec<_>>();

        let outcome = environment.step(&actions, false);
        self.reward += outcome.reward;

        if is_key_pressed(KeyCode::Space) {
            let nearest_segment = &environment
                .track
                .nearest_segments(environment.cars[0].position(), 1)[0];
            environment.cars[0].reset(
                nearest_segment.start.pos,
                nearest_segment.start.dir.to_angle(),
                0.0,
            );
            self.controllers[0].reset();
        }
        if is_key_pressed(KeyCode::O) {
            self.show_observation = !self.show_observation;
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
        if self.show_observation {
            Game::draw_observation(
                &environment.observations[0],
                &environment.cars[0],
                self.reward,
            );
        }
        self.draw_stopwatch();
    }
}
