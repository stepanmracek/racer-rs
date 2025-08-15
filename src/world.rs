use crate::{car::Car, track::Track};

pub struct World {
    pub track: Track,
    pub car: Car,
}

impl World {
    pub async fn new() -> Self {
        let car = Car::new(0.0, 15.0).await;
        let mut track = Track::new();
        for _ in 0..3 {
            track.add_random_shape();
        }
        track.add_finish();
        track.compute_rtree();
        Self { car, track }
    }
}
