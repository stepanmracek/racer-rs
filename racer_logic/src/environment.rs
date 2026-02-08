use crate::{
    car::{Car, Intention},
    constants::SENSOR_REACH,
    follow_camera::FollowCamera,
    goal::{Goal, ReachFinish},
    observation::{NextWaypoint, ObjectType, Observation, SensorReadings},
    physics::segment_vs_rotrect,
    track::{Track, distances_to_segments},
};
use macroquad::prelude::*;
use std::{
    f32::consts::{FRAC_PI_2, PI},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct Environment {
    pub track: Track,
    pub cars: Vec<Car>,
    pub observations: Vec<Observation>,
    goals: Vec<Box<dyn Goal>>,
    off_track_prob: f32,
    track_width: f32,
}

pub struct EnvironmentBuilder {
    seed: Option<u64>,
    off_track_prob: f32,
    goal: Box<dyn Goal>,
    track_width: f32,
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

        let mut cars = (0..cars_count).map(|_| Car::default()).collect::<Vec<_>>();
        Environment::position_cars(&mut cars, off_track_prob, track_width);
        let observations = Environment::observe_all(&cars, &track);
        let goals = (0..cars_count).map(|_| goal.cloned()).collect();

        let mut environment = Self {
            cars,
            track,
            observations,
            goals,
            off_track_prob,
            track_width,
        };
        environment.reset();
        environment
    }

    fn position_cars(cars: &mut [Car], off_track_prob: f32, track_width: f32) {
        let len = cars.len();
        if len == 1 && off_track_prob > 0.0 {
            if macroquad::rand::gen_range(0.0, 1.0) <= off_track_prob {
                let x = macroquad::rand::gen_range(-track_width, track_width);
                cars[0].reset(
                    vec2(x, 100.0),
                    macroquad::rand::gen_range(-PI, PI),
                    macroquad::rand::gen_range(-20., 50.),
                );
            } else {
                cars[0].reset(vec2(0.0, 15.0), FRAC_PI_2, 0.0);
            };
        } else {
            let dx = if len.is_multiple_of(2) { -10.0 } else { 0.0 };
            let xs = std::iter::once(0).chain((1..).flat_map(|n| [n * 20, -n * 20]));
            std::iter::zip(cars.iter_mut(), xs).for_each(|(car, x)| {
                car.reset(vec2(x as f32 + dx, 15.0), FRAC_PI_2, 0.0);
            });
        }
        cars.iter_mut().for_each(|car| car.impulse(Vec2::ZERO, 0.0));
    }

    pub fn reset(&mut self) {
        Environment::position_cars(&mut self.cars, self.off_track_prob, self.track_width);
        self.goals.iter_mut().for_each(|goal| goal.reset());
        self.observations = Environment::observe_all(&self.cars, &self.track);
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

    fn merge(segments: &[Option<f32>], cars: &[Option<f32>]) -> Vec<Option<(ObjectType, f32)>> {
        segments
            .iter()
            .zip(cars.iter())
            .map(|(&segment, &car)| match (segment, car) {
                (Some(segment), Some(car)) => {
                    if segment < car {
                        Some((ObjectType::Track, segment))
                    } else {
                        Some((ObjectType::Car, car))
                    }
                }
                (Some(segment), None) => Some((ObjectType::Track, segment)),
                (None, Some(car)) => Some((ObjectType::Car, car)),
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

    pub fn step(&mut self, actions: &[Action], fixed_time: bool) -> Vec<Outcome> {
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
        std::iter::zip(self.cars.iter(), self.observations.iter())
            .zip(self.goals.iter_mut())
            .map(|((car, observation), goal)| goal.outcome(&self.track, car, observation))
            .collect()
    }

    pub fn draw(&self, follow_camera: &mut FollowCamera) {
        clear_background(DARKGREEN);
        follow_camera.update(&self.cars);
        self.track.draw(follow_camera.target());
        self.cars.iter().for_each(|c| c.draw_skid_marks());
        self.cars.iter().for_each(|c| c.draw());
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(Some(0), 0., Box::new(ReachFinish::default()), 1, 42.0)
    }
}
