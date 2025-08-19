use crate::{
    car::Car,
    follow_camera::FollowCamera,
    track::{Track, sensor_readings},
};
use macroquad::prelude::*;

pub struct World {
    pub track: Track,
    pub car: Car,
}

impl World {
    pub async fn new() -> Self {
        let car = Car::new(0.0, 15.0).await;
        let mut track = Track::new();
        for _ in 0..100 {
            track.add_random_shape();
        }
        track.add_finish();
        track.compute_rtree();
        Self { car, track }
    }

    pub fn draw(&self, follow_camera: &mut FollowCamera) {
        clear_background(DARKGREEN);
        follow_camera.update(&self.car);
        self.track.draw(&self.car);
        self.car.draw();

        let sensor_len = 200.0;
        let x = self.car.position + Vec2::from_angle(self.car.rotation) * sensor_len * 0.5;
        let nearest_segments = self.track.nearest_segments(&x, 5);
        let sensor_rays = self.car.sensor_rays(sensor_len);
        let readings = sensor_readings(&nearest_segments, &sensor_rays);

        for (d, (start, end)) in readings.iter().zip(sensor_rays) {
            draw_line(start.x, start.y, end.x, end.y, 0.3, GREEN.with_alpha(0.2));
            if let Some(d) = d {
                let p = (end - start).normalize() * *d + start;
                draw_circle(p.x, p.y, 1.0, RED);
            }
        }
    }
}
