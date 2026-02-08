use std::{collections::HashSet, f32::consts::FRAC_PI_2};

use crate::{
    car::Car,
    environment::{Environment, NextWaypoint, Observation, Outcome},
    track::Track,
};

pub trait Goal {
    fn outcome(&mut self, track: &Track, car: &Car, observation: &Observation) -> Outcome;
    fn reset(&mut self);
    fn cloned(&self) -> Box<dyn Goal>;
}

#[derive(Default, Clone)]
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
        let velocity = car.velocity();
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
        let truncated = self.out_of_track_in_row >= 300;
        let terminated = finish_line || truncated;

        Outcome {
            terminated,
            truncated,
            reward,
        }
    }

    fn reset(&mut self) {
        self.out_of_track_in_row = 0;
        self.rewarded_waypoints.clear();
    }

    fn cloned(&self) -> Box<dyn Goal> {
        Box::new(self.clone())
    }
}

#[derive(Default, Clone)]
pub struct BackToTrack {
    prev_wp_observation: Option<NextWaypoint>,
}

impl Goal for BackToTrack {
    fn outcome(&mut self, track: &Track, _car: &Car, observation: &Observation) -> Outcome {
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
            && observation.next_waypoint.distance < track.width() / 2.0
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

    fn reset(&mut self) {
        self.prev_wp_observation.take();
    }

    fn cloned(&self) -> Box<dyn Goal> {
        Box::new(self.clone())
    }
}
