use crate::constants::SENSOR_REACH;
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub enum ObjectType {
    Track,
    Car,
}

#[derive(Debug, Clone)]
pub struct SensorReadings {
    pub rays: Vec<(Vec2, Vec2)>,
    pub distances: Vec<Option<(ObjectType, f32)>>,
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

impl From<Observation> for Vec<f32> {
    fn from(o: Observation) -> Vec<f32> {
        let mut ans = vec![
            o.velocity,
            o.steering_angle,
            o.next_waypoint.angle,
            o.next_waypoint.distance,
        ];
        ans.extend(o.wheels_on_track.iter().map(|&w| if w { 1.0 } else { 0.0 }));
        ans.extend(o.sensors.distances.iter().map(|r| {
            if let Some((_, r)) = r {
                *r
            } else {
                SENSOR_REACH
            }
        }));
        ans
    }
}

pub fn get_distance(d: &Option<(ObjectType, f32)>) -> f32 {
    if let Some((_, d)) = d {
        *d
    } else {
        SENSOR_REACH
    }
}
