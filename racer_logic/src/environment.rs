use crate::{
    car::{Car, Intention},
    follow_camera::FollowCamera,
    goal::{Goal, ReachFinish},
    physics::segment_vs_rotrect,
    track::{Track, distances_to_segments},
};
use macroquad::prelude::*;
use std::{
    f32::consts::PI,
    time::{SystemTime, UNIX_EPOCH},
    vec,
};

pub const SENSOR_REACH: f32 = 205.0;

pub struct Environment {
    pub track: Track,
    pub cars: Vec<Car>,
    pub observations: Vec<Observation>,
    goal: Box<dyn Goal>,
}

pub struct EnvironmentBuilder {
    seed: Option<u64>,
    off_track_prob: f32,
    goal: Box<dyn Goal>,
    track_width: f32,
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
    steer: f32,
    throttle: f32,
}

impl Action {
    pub fn new(steer: f32, throttle: f32) -> Self {
        Self {
            steer: steer.clamp(-1.0, 1.0),
            throttle: throttle.clamp(-1.0, 1.0),
        }
    }
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

impl Default for EnvironmentBuilder {
    fn default() -> Self {
        Self {
            seed: None,
            off_track_prob: 0.0,
            goal: Box::new(ReachFinish::default()),
            track_width: 42.0,
        }
    }
}

impl EnvironmentBuilder {
    pub fn with_seed(mut self, seed: Option<u64>) -> Self {
        self.seed = seed;
        self
    }
    pub fn with_off_track_prob(mut self, off_track_prob: f32) -> Self {
        self.off_track_prob = off_track_prob;
        self
    }
    pub fn with_goal(mut self, goal: Box<dyn Goal>) -> Self {
        self.goal = goal;
        self
    }
    pub fn with_track_width(mut self, track_width: f32) -> Self {
        self.track_width = track_width;
        self
    }
    pub fn build(self, cars_count: usize) -> Result<Environment, String> {
        if cars_count == 0 {
            return Err("At least one car is required!".into());
        }
        Ok(Environment::new(
            self.seed,
            self.off_track_prob,
            self.goal,
            cars_count,
            self.track_width,
        ))
    }
}

impl Environment {
    pub fn new(
        seed: Option<u64>,
        off_track_prob: f32,
        goal: Box<dyn Goal>,
        cars_count: usize,
        track_width: f32,
    ) -> Self {
        let seed = seed.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64
        });
        macroquad::rand::srand(seed);

        let mut track = Track::new(track_width);
        for _ in 0..100 {
            track.add_random_shape();
        }
        track.add_finish();
        track.compute_rtree();

        let cars = if cars_count == 1 && off_track_prob > 0.0 {
            let car = if macroquad::rand::gen_range(0.0, 1.0) <= off_track_prob {
                let x = macroquad::rand::gen_range(-track_width, track_width);
                Car::new(x, 100.)
                    .with_rotation(macroquad::rand::gen_range(-PI, PI))
                    .with_velocity(macroquad::rand::gen_range(-20., 50.))
            } else {
                Car::new(0.0, 15.0)
            };
            vec![car]
        } else {
            std::iter::once(0)
                .chain((1..).flat_map(|n| [n * 16, -n * 16]))
                .take(cars_count)
                .map(|x| {
                    #[allow(clippy::manual_is_multiple_of)]
                    let dx = if cars_count % 2 == 0 { -8.0 } else { 0.0 };
                    Car::new(x as f32 + dx, 15.0)
                })
                .collect::<Vec<_>>()
        };

        let observations = Environment::observe_all(&cars, &track);

        Self {
            cars,
            track,
            observations,
            goal,
        }
    }

    fn ray_vs_cars(car_index: usize, ray: &(Vec2, Vec2), cars: &[Car]) -> Option<f32> {
        cars.iter()
            .enumerate()
            .filter_map(|(other_index, other_car)| {
                if other_index == car_index {
                    None
                } else {
                    segment_vs_rotrect(ray, other_car.bbox())
                }
            })
            .min_by(|a, b| {
                let dist_a = (*a - ray.0).length_squared();
                let dist_b = (*b - ray.0).length_squared();
                dist_a.total_cmp(&dist_b)
            })
            .map(|intersection| (intersection - ray.0).length())
    }

    fn rays_vs_cars(car_index: usize, rays: &[(Vec2, Vec2)], cars: &[Car]) -> Vec<Option<f32>> {
        rays.iter()
            .map(|ray| Environment::ray_vs_cars(car_index, ray, cars))
            .collect()
    }

    fn merge(dist1: &[Option<f32>], dist2: &[Option<f32>]) -> Vec<Option<f32>> {
        dist1
            .iter()
            .zip(dist2.iter())
            .map(|(&d1, &d2)| match (d1, d2) {
                (Some(val1), Some(val2)) => Some(val1.min(val2)),
                (Some(val1), None) => Some(val1),
                (None, Some(val2)) => Some(val2),
                (None, None) => None,
            })
            .collect()
    }

    fn sensor_readings(car_index: usize, cars: &[Car], track: &Track) -> SensorReadings {
        let car = &cars[car_index];
        let x = car.windshield_position();
        let rays = car.sensor_rays(SENSOR_REACH);

        let car_distances = Environment::rays_vs_cars(car_index, &rays, cars);

        let nearest_segments = track.nearest_segments(x, 10);
        let segment_distances = distances_to_segments(&nearest_segments, &rays, track.width());
        SensorReadings {
            rays,
            distances: Environment::merge(&segment_distances, &car_distances),
        }
    }

    fn observe(car_index: usize, cars: &[Car], track: &Track) -> Observation {
        let car = &cars[car_index];
        let car_pos = car.windshield_position();
        let waypoint = &track.nearest_segments(car_pos, 1)[0].end;

        let to_waypoint = waypoint.pos - car_pos;
        let car_rotation = Vec2::from_angle(car.rotation());
        let angle = car_rotation.angle_between(to_waypoint);
        let distance = to_waypoint.length();
        let alignment = car_rotation.dot(waypoint.dir);

        Observation {
            velocity: car.velocity(),
            steering_angle: car.steering_angle(),
            wheels_on_track: car.wheels_on_track(track),
            sensors: Environment::sensor_readings(car_index, cars, track),
            next_waypoint: NextWaypoint {
                angle,
                distance,
                alignment,
            },
        }
    }

    fn observe_all(cars: &[Car], track: &Track) -> Vec<Observation> {
        (0..cars.len())
            .map(|car_index| Environment::observe(car_index, cars, track))
            .collect()
    }

    pub fn get_nearest_waypoint(track: &Track, car: &Car) -> (i32, i32) {
        let segments = track.nearest_segments(car.position(), 1);
        let wp_pos = segments[0].end.pos;
        (wp_pos.x as i32, wp_pos.y as i32)
    }

    #[inline]
    fn dt(fixed_time: bool) -> f32 {
        if fixed_time {
            1.0 / 60.0
        } else {
            get_frame_time()
        }
    }

    fn handle_collision(
        &mut self,
        dt: f32,
        collision_point: Vec2,
        intentions: &[Intention],
        this: usize,
        other: usize,
    ) {
        let this_velocity = intentions[this].new_pos - intentions[this].old_pos;
        let other_velocity = intentions[other].new_pos - intentions[other].old_pos;
        let rel_velocity = (this_velocity - other_velocity) / dt;

        let to_collision = (collision_point - intentions[this].new_bbox.center()).normalize();
        let rotational_impulse = (rel_velocity / 50.0).perp_dot(to_collision);

        self.cars[this].reset(intentions[this].old_pos, intentions[this].old_rot, 0.0);
        self.cars[this].impulse(-rel_velocity / 2.0, rotational_impulse);
    }

    pub fn step(&mut self, actions: &[Action], fixed_time: bool) -> Outcome {
        let dt = Environment::dt(fixed_time);
        let intentions = std::iter::zip(
            self.cars.iter_mut(),
            std::iter::zip(self.observations.iter(), actions.iter()),
        )
        .map(|(car, (observation, action))| {
            car.step(
                &observation.wheels_on_track,
                action.steer,
                action.throttle,
                dt,
            )
        })
        .collect::<Vec<_>>();

        for this in 0..intentions.len() - 1 {
            for other in this + 1..intentions.len() {
                if let Some(collision_point) = intentions[this]
                    .new_bbox
                    .collision_point(&intentions[other].new_bbox)
                {
                    self.handle_collision(dt, collision_point, &intentions, this, other);
                    self.handle_collision(dt, collision_point, &intentions, other, this);
                    break;
                }
            }
        }

        for i in 0..self.cars.len() - 1 {
            for j in i + 1..self.cars.len() {
                debug_assert!(!self.cars[i].bbox().collide(self.cars[j].bbox()));
            }
        }

        self.observations = Environment::observe_all(&self.cars, &self.track);
        self.goal
            .outcome(&self.track, &self.cars[0], &self.observations[0])
    }

    pub fn draw(&self, follow_camera: &mut FollowCamera) {
        clear_background(DARKGREEN);
        follow_camera.update(&self.cars[0]);
        self.track.draw(self.cars[0].position());
        self.cars.iter().for_each(|c| c.draw_skid_marks());
        self.cars.iter().for_each(|c| c.draw());
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(Some(0), 0., Box::new(ReachFinish::default()), 1, 42.0)
    }
}
