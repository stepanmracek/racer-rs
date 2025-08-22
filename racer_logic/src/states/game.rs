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
}

impl Game {
    pub fn new(
        follow_camera: &FollowCamera,
        controller_factory: fn() -> Box<dyn Controller>,
    ) -> Self {
        let follow_camera = follow_camera.clone();
        Self {
            follow_camera,
            state_started: get_time(),
            controller: controller_factory(),
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

    fn draw_observation(observation: &Observation, car: &Car) {
        for (d, (start, end)) in zip(&observation.sensors.distances, &observation.sensors.rays) {
            draw_line(start.x, start.y, end.x, end.y, 0.3, GREEN.with_alpha(0.2));
            if let Some(d) = d {
                let p = (*end - *start).normalize() * *d + *start;
                draw_circle(p.x, p.y, 1.0, RED);
            }
        }

        let to_waypoint = Vec2::from_angle(*car.rotation()).rotate(
            Vec2::from_angle(observation.next_waypoint.angle) * observation.next_waypoint.distance,
        );
        let car_pos = car.windshield_position();
        draw_line(
            car_pos.x,
            car_pos.y,
            car_pos.x + to_waypoint.x,
            car_pos.y + to_waypoint.y,
            0.5,
            GREEN.with_alpha(0.5),
        );
        push_camera_state();
        set_default_camera();
        draw_multiline_text(
            &format!(
                "next_waypoint: {:.2}\nspeed: {:.2}",
                observation.next_waypoint.angle, observation.velocity
            ),
            screen_width() * 0.5,
            screen_height() * 0.5,
            16.0,
            None,
            YELLOW.with_alpha(0.3),
        );
        pop_camera_state();
    }
}

impl State for Game {
    fn step(&mut self, environment: &mut Environment) -> Option<Box<dyn State>> {
        //let mut vec: Vec<f32> = environment.observation.clone().into();
        let action = self.controller.control(&environment.observation);
        //vec.extend([action.steer, action.throttle]);
        //println!("{vec:?}");

        let outcome = environment.step(&action, false);

        if is_key_pressed(KeyCode::Space) {
            let nearest_segment = &environment
                .track
                .nearest_segments(environment.car.position(), 1)[0];
            environment.car.reset(
                &nearest_segment.start.pos,
                nearest_segment.start.dir.to_angle(),
                0.0,
            );
        }

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
        Game::draw_observation(&environment.observation, &environment.car);
        self.draw_stopwatch();
    }
}
