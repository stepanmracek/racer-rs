use crate::{
    controller::{Controller, KeyboardController},
    follow_camera::FollowCamera,
    states::{State, finish::Finish},
    track::sensor_readings,
    utils::format_time,
    world::World,
};
use macroquad::prelude::*;

const SENSOR_REACH: f32 = 200.0;

pub struct Game {
    follow_camera: FollowCamera,
    state_started: f64,
    sensor_rays: Vec<(Vec2, Vec2)>,
    readings: Vec<Option<f32>>,
    controller: Box<dyn Controller>,
}

impl Game {
    pub fn new(follow_camera: &FollowCamera) -> Self {
        let follow_camera = follow_camera.clone();
        Self {
            follow_camera,
            state_started: get_time(),
            sensor_rays: vec![],
            readings: vec![],
            controller: Box::new(KeyboardController {}),
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

    fn draw_readings(&self) {
        for (d, (start, end)) in self.readings.iter().zip(self.sensor_rays.iter()) {
            draw_line(start.x, start.y, end.x, end.y, 0.3, GREEN.with_alpha(0.2));
            if let Some(d) = d {
                let p = (*end - *start).normalize() * *d + *start;
                draw_circle(p.x, p.y, 1.0, RED);
            }
        }
    }

    fn update_readings(&mut self, world: &World) {
        let x =
            *world.car.position() + Vec2::from_angle(*world.car.rotation()) * SENSOR_REACH * 0.5;
        let nearest_segments = world.track.nearest_segments(&x, 5);
        self.sensor_rays = world.car.sensor_rays(SENSOR_REACH);
        self.readings = sensor_readings(&nearest_segments, &self.sensor_rays);
    }
}

impl State for Game {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        self.update_readings(world);
        let wheels_on_track = world.car.wheels_on_track(&world.track);

        /*let mut vec = vec![*world.car.velocity(), *world.car.steering_angle()];
        vec.extend(wheels_on_track.iter().map(|&w| if w { 1.0 } else { 0.0 }));
        vec.extend(self.readings.iter().map(|r| r.unwrap_or(SENSOR_REACH)));*/

        let control = self.controller.control(
            *world.car.velocity(),
            *world.car.steering_angle(),
            &wheels_on_track,
            &self.readings,
        );
        world
            .car
            .update(&wheels_on_track, control.steer, control.throttle);

        /*vec.push(control.steer);
        vec.push(control.throttle);
        let s = vec
            .iter()
            .map(|i| format!("{i:.5}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("{s}");*/

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
        self.draw_readings();
        self.draw_stopwatch();
    }
}
