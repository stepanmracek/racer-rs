use crate::car::Car;
use macroquad::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub struct FollowCamera {
    zoom: f32,
    camera_2d: Camera2D,
}

impl FollowCamera {
    pub fn new(car: &Car) -> Self {
        let zoom = 8.0;
        let camera_2d = Camera2D {
            target: *car.position(),
            zoom: vec2(zoom / screen_width(), -zoom / screen_height()),
            rotation: -car.rotation().to_degrees() + 90.0,
            ..Default::default()
        };
        Self { zoom, camera_2d }
    }

    pub fn update(&mut self, car: &Car) {
        let car_rotation = car.rotation() - FRAC_PI_2;
        let car_pos = *car.position();
        let shift = Vec2::from_angle(car_rotation).rotate(vec2(0.0, 50.0));
        let dt = get_frame_time();
        self.camera_2d.rotation = self.camera_2d.rotation.lerp(-car_rotation.to_degrees(), dt);
        self.camera_2d.target = self.camera_2d.target.lerp(car_pos + shift, 5.0 * dt);
        self.camera_2d.zoom = vec2(self.zoom / screen_width(), -self.zoom / screen_height());
        set_camera(&self.camera_2d);
    }
}

impl Clone for FollowCamera {
    fn clone(&self) -> Self {
        Self {
            zoom: self.zoom,
            camera_2d: Camera2D {
                rotation: self.camera_2d.rotation,
                zoom: self.camera_2d.zoom,
                target: self.camera_2d.target,
                offset: self.camera_2d.offset,
                render_target: self.camera_2d.render_target.clone(),
                viewport: self.camera_2d.viewport,
            },
        }
    }
}
