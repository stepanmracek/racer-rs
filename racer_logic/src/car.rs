use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_6};

use crate::{physics::RotRect, track::Track};

pub struct Car {
    texture: Option<Texture2D>,
    position: Vec2,
    rotation: f32,
    velocity: f32,
    steering_angle: f32,
    wheels: [Vec2; 4],
    wheel_base: f32,
    bbox: RotRect,
}

impl Car {
    pub fn new(x: f32, y: f32) -> Self {
        let wheel_base = 14.0;
        let position = vec2(x, y);
        let rotation = FRAC_PI_2;
        Self {
            texture: None,
            position,
            rotation,
            velocity: 0.0,
            steering_angle: 0.0,
            wheel_base,
            wheels: [
                vec2(4.5, wheel_base),  // front right
                vec2(-4.5, wheel_base), // front left
                vec2(4.5, 0.0),         // rear right
                vec2(-4.5, 0.0),        // rear left
            ],
            bbox: RotRect::new(
                position + Vec2::from_angle(rotation) * wheel_base / 2.0,
                vec2(10.0, 25.0),
                0.0,
            ),
        }
    }

    pub async fn load_texture(&mut self) {
        self.texture = Some(load_texture("assets/car.png").await.unwrap());
    }

    pub fn reset(&mut self, position: &Vec2, rotation: f32, velocity: f32) {
        self.position = *position;
        self.rotation = rotation;
        self.velocity = velocity;
        self.steering_angle = 0.0
    }

    pub fn update(
        &mut self,
        wheels_on_track: &[bool; 4],
        steer: f32,
        throttle: f32,
        fixed_time: bool,
    ) {
        let dt = if fixed_time {
            1.0 / 60.0
        } else {
            get_frame_time()
        };
        let turn_speed = FRAC_PI_6;

        self.steering_angle += steer * turn_speed * dt;
        if steer == 0.0 {
            self.steering_angle = self.steering_angle.lerp(0.0, (10.0 * dt).clamp(0.0, 1.0));
        }
        self.steering_angle = self.steering_angle.clamp(-FRAC_PI_6, FRAC_PI_6);

        let acceleration = 50.0;
        self.velocity += throttle * acceleration * dt;

        let penalty = wheels_on_track
            .iter()
            .filter(|&&on_track| !on_track)
            .map(|_| 0.99)
            .product::<f32>();
        let friction = 0.995 * penalty;
        self.velocity *= friction;

        let pos_dot = Vec2::from_angle(self.rotation) * self.velocity;
        let theta_dot = self.velocity * self.steering_angle.tan() / self.wheel_base;
        self.position += pos_dot * dt;
        self.rotation += theta_dot * dt;
        self.bbox.update(
            self.position + Vec2::from_angle(self.rotation) * self.wheel_base / 2.0,
            self.rotation - FRAC_PI_2,
        );
    }

    pub fn draw(&self) {
        let draw_rot = self.rotation - FRAC_PI_2;
        let rot_vec = Vec2::from_angle(self.rotation);
        let orientation = Vec2::from_angle(draw_rot);

        for (i, &wheel) in self.wheels.iter().enumerate() {
            let wheel_pos = self.position + orientation.rotate(wheel);
            let mut wheel_rot = draw_rot;
            if i < 2 {
                wheel_rot += self.steering_angle;
            }
            draw_rectangle_ex(
                wheel_pos.x,
                wheel_pos.y,
                1.5,
                3.0,
                DrawRectangleParams {
                    rotation: wheel_rot,
                    color: BLACK,
                    offset: vec2(0.5, 0.5),
                },
            );
        }

        if let Some(texture) = &self.texture {
            let texture_pos = (self.position + rot_vec * self.wheel_base / 2.0)
                - vec2(texture.width() / 40.0, texture.height() / 40.0);
            draw_texture_ex(
                texture,
                texture_pos.x,
                texture_pos.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(texture.size() / 20.0),
                    flip_y: true,
                    rotation: draw_rot,
                    ..Default::default()
                },
            );
        }
    }

    pub fn wheels_on_track(&self, track: &Track) -> [bool; 4] {
        let orientation = Vec2::from_angle(self.rotation - FRAC_PI_2);
        let mut ans = [false; 4];
        for (i, wheel) in self.wheels.iter().enumerate() {
            let pos = self.position + orientation.rotate(*wheel);
            let on_track = track.on_track(&pos);
            ans[i] = on_track;
        }
        ans
    }

    pub fn bbox(&self) -> &RotRect {
        &self.bbox
    }

    pub fn sensor_rays(&self, sensor_len: f32) -> Vec<(Vec2, Vec2)> {
        let start_offset = 10.0;
        let start = self.position + Vec2::from_angle(self.rotation) * start_offset;
        (-60..=60)
            .step_by(10)
            .map(|delta| {
                let angle = Vec2::from_angle(self.rotation + (delta as f32).to_radians());
                let start = start + angle.rotate(vec2(5.0, 0.0));
                let end = start + angle.rotate(vec2(sensor_len, 0.0));
                (start, end)
            })
            .collect()
    }

    pub fn position(&self) -> &Vec2 {
        &self.position
    }

    pub fn rotation(&self) -> &f32 {
        &self.rotation
    }

    pub fn velocity(&self) -> &f32 {
        &self.velocity
    }

    pub fn steering_angle(&self) -> &f32 {
        &self.steering_angle
    }
}
