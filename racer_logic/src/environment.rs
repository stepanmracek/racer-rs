use crate::{
    car::Car,
    follow_camera::FollowCamera,
    track::{Track, sensor_readings},
};
use macroquad::prelude::*;

pub const SENSOR_REACH: f32 = 205.0;

pub struct Environment {
    pub track: Track,
    pub car: Car,
    pub observation: Observation,
}

#[derive(Debug, Clone)]
pub struct SensorReadings {
    pub rays: Vec<(Vec2, Vec2)>,
    pub distances: Vec<Option<f32>>,
}

#[derive(Debug, Clone)]
pub struct NextWaypoint {
    pub angle: f32,
    pub distance: f32,
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub velocity: f32,
    pub steering_angle: f32,
    pub wheels_on_track: [bool; 4],
    pub sensors: SensorReadings,
    pub next_waypoint: NextWaypoint,
}

#[derive(Debug)]
pub struct Action {
    pub steer: f32,
    pub throttle: f32,
}

#[derive(Debug)]
pub struct Outcome {
    pub finished: bool,
}

impl From<Observation> for Vec<f32> {
    fn from(o: Observation) -> Vec<f32> {
        let mut ans = vec![
            o.velocity,
            o.steering_angle,
            o.next_waypoint.angle,
            o.next_waypoint.distance,
        ];
        ans.extend(o.wheels_on_track.iter().map(|&w| if w { 1.0 } else { 0.0 }));
        ans.extend(
            o.sensors
                .distances
                .iter()
                .map(|r| r.unwrap_or(SENSOR_REACH)),
        );
        ans
    }
}

impl Environment {
    pub fn new(seed: u64) -> Self {
        macroquad::rand::srand(seed);
        let car = Car::new(0.0, 15.0);
        let mut track = Track::new();
        for _ in 0..2 {
            track.add_random_shape();
        }
        track.add_finish();
        track.compute_rtree();
        let observation = Environment::observe(&car, &track);
        Self {
            car,
            track,
            observation,
        }
    }

    fn sensor_readings(car: &Car, track: &Track) -> SensorReadings {
        let x = car.position_with_offset(SENSOR_REACH * 0.5);
        let nearest_segments = track.nearest_segments(&x, 5);
        let rays = car.sensor_rays(SENSOR_REACH);
        let distances = sensor_readings(&nearest_segments, &rays);
        SensorReadings { rays, distances }
    }

    fn observe(car: &Car, track: &Track) -> Observation {
        let car_pos = car.windshield_position();
        let search_pos = car.position_with_offset(50.0);
        let waypoint_pos = track.nearest_segments(&search_pos, 1)[0].end.pos;
        let to_waypoint = waypoint_pos - car_pos;
        let angle = Vec2::from_angle(*car.rotation()).angle_between(to_waypoint);
        let distance = to_waypoint.length();

        Observation {
            velocity: *car.velocity(),
            steering_angle: *car.steering_angle(),
            wheels_on_track: car.wheels_on_track(track),
            sensors: Environment::sensor_readings(car, track),
            next_waypoint: NextWaypoint { angle, distance },
        }
    }

    pub fn step(&mut self, action: &Action, fixed_time: bool) -> Outcome {
        self.car.update(
            &self.observation.wheels_on_track,
            action.steer,
            action.throttle,
            fixed_time,
        );
        self.observation = Environment::observe(&self.car, &self.track);

        Outcome {
            finished: self.track.finish(self.car.bbox()),
        }
    }

    pub fn draw(&self, follow_camera: &mut FollowCamera) {
        clear_background(DARKGREEN);
        follow_camera.update(&self.car);
        self.track.draw(&self.car);
        self.car.draw();
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(0)
    }
}
