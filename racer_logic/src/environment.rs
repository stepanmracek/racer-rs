use std::{
    collections::HashSet,
    f32::consts::{FRAC_PI_2, PI},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    car::Car,
    follow_camera::FollowCamera,
    track::{TRACK_WIDTH, Track, sensor_readings},
};
use macroquad::prelude::*;

pub const SENSOR_REACH: f32 = 205.0;

pub struct Environment {
    pub track: Track,
    pub car: Car,
    pub observation: Observation,
    goal: Box<dyn Goal>,
}

pub trait Goal {
    fn outcome(&mut self, track: &Track, car: &Car, observation: &Observation) -> Outcome;
}

#[derive(Default)]
pub struct ReachFinish {
    rewarded_waypoints: HashSet<(i32, i32)>,
    out_of_track_in_row: usize,
}

impl Goal for ReachFinish {
    fn outcome(&mut self, track: &Track, car: &Car, observation: &Observation) -> Outcome {
        let wheels_on_track_count = observation.wheels_on_track.iter().filter(|b| **b).count();
        if wheels_on_track_count == 4 {
            self.out_of_track_in_row = 0;
        } else {
            self.out_of_track_in_row += 1;
        }

        // penalize 0.25 points if some wheel is out of the track
        let mut reward = (4 - wheels_on_track_count) as f32 * -0.25;

        // reward for moving forward
        let forward = observation.next_waypoint.angle.abs() <= FRAC_PI_2;
        let velocity = *car.velocity();
        if wheels_on_track_count == 4 && velocity > 1.0 && forward {
            reward += velocity.ln()
        }

        if velocity < 0.0 {
            reward -= (velocity.abs() + 1.0).ln()
        }

        // reward for each new discovered waypoint (but not waypoint on the first segment)
        let wp_key = Environment::get_nearest_waypoint(track, car);
        if wheels_on_track_count == 4
            && !self.rewarded_waypoints.contains(&wp_key)
            && wp_key != (0, 100)
        {
            reward += 100.0;
            self.rewarded_waypoints.insert(wp_key);
        }

        // extra reward for reaching the finish line
        let finish_line = track.finish(car.bbox());
        if finish_line {
            reward += 10_000.0;
        }

        // end simulation if reached finish or out of track for too long (5 seconds @ 60 fps)
        let truncated = self.out_of_track_in_row > 300;
        let terminated = finish_line || truncated;

        Outcome {
            terminated,
            truncated,
            reward,
        }
    }
}

#[derive(Default)]
pub struct BackToTrack {
    prev_wp_observation: Option<NextWaypoint>,
}

impl Goal for BackToTrack {
    fn outcome(&mut self, _track: &Track, _car: &Car, observation: &Observation) -> Outcome {
        let wheels_on_track_count = observation.wheels_on_track.iter().filter(|b| **b).count();

        // penalize 0.25 points if some wheel is out of the track
        let mut reward = (4 - wheels_on_track_count) as f32 * -0.25;

        // reward if car is approaching waypoint
        if let Some(prev_wp_obs) = &self.prev_wp_observation {
            let wp_obs = &observation.next_waypoint;
            if wp_obs.alignment > prev_wp_obs.alignment {
                reward += 0.5;
            }
            if wp_obs.distance < prev_wp_obs.distance {
                reward += 0.5;
            }
            if wp_obs.angle.abs() < prev_wp_obs.angle.abs() {
                reward += 0.5;
            }
        }

        self.prev_wp_observation = Some(observation.next_waypoint.clone());
        let terminated = wheels_on_track_count == 4
            && observation.next_waypoint.distance < TRACK_WIDTH / 2.0
            && observation.next_waypoint.alignment > 0.9
            && observation.velocity > 0.0;
        if terminated {
            reward += 1_000.0;
        }
        Outcome {
            terminated,
            reward,
            truncated: false,
        }
    }
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
    pub alignment: f32,
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
    // Whether the agent reaches the terminal state
    // which can be positive or negative
    pub terminated: bool,

    // Whether the truncation condition is satisfied.
    pub truncated: bool,
    pub reward: f32,
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
    pub fn new(seed: Option<u64>, off_track_prob: f32, goal: Box<dyn Goal>) -> Self {
        let seed = seed.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64
        });
        macroquad::rand::srand(seed);

        let mut track = Track::new();
        for _ in 0..100 {
            track.add_random_shape();
        }
        track.add_finish();
        track.compute_rtree();

        let car = if off_track_prob > 0.0 && macroquad::rand::gen_range(0.0, 1.0) <= off_track_prob
        {
            let x = macroquad::rand::gen_range(-TRACK_WIDTH, TRACK_WIDTH);
            Car::new(x, 100.)
                .with_rotation(macroquad::rand::gen_range(-PI, PI))
                .with_velocity(macroquad::rand::gen_range(-20., 50.))
        } else {
            Car::new(0.0, 15.0)
        };

        let observation = Environment::observe(&car, &track);
        Self {
            car,
            track,
            observation,
            goal,
        }
    }

    fn sensor_readings(car: &Car, track: &Track) -> SensorReadings {
        let x = car.windshield_position();
        let nearest_segments = track.nearest_segments(&x, 10);
        let rays = car.sensor_rays(SENSOR_REACH);
        let distances = sensor_readings(&nearest_segments, &rays);
        SensorReadings { rays, distances }
    }

    fn observe(car: &Car, track: &Track) -> Observation {
        let car_pos = car.windshield_position();
        let waypoint = &track.nearest_segments(&car_pos, 1)[0].end;

        let to_waypoint = waypoint.pos - car_pos;
        let car_rotation = Vec2::from_angle(*car.rotation());
        let angle = car_rotation.angle_between(to_waypoint);
        let distance = to_waypoint.length();
        let alignment = car_rotation.dot(waypoint.dir);

        Observation {
            velocity: *car.velocity(),
            steering_angle: *car.steering_angle(),
            wheels_on_track: car.wheels_on_track(track),
            sensors: Environment::sensor_readings(car, track),
            next_waypoint: NextWaypoint {
                angle,
                distance,
                alignment,
            },
        }
    }

    fn get_nearest_waypoint(track: &Track, car: &Car) -> (i32, i32) {
        let segments = track.nearest_segments(car.position(), 1);
        let wp_pos = segments[0].end.pos;
        (wp_pos.x as i32, wp_pos.y as i32)
    }

    pub fn step(&mut self, action: &Action, fixed_time: bool) -> Outcome {
        self.car.update(
            &self.observation.wheels_on_track,
            action.steer,
            action.throttle,
            fixed_time,
        );
        self.observation = Environment::observe(&self.car, &self.track);

        let goal = &mut self.goal;
        goal.outcome(&self.track, &self.car, &self.observation)
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
        Self::new(Some(0), 0., Box::new(ReachFinish::default()))
    }
}
