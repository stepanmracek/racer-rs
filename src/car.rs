use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use crate::track::Track;

pub struct Car {
    texture: Texture2D,
    pub position: Vec2,
    pub rotation: f32,
    velocity: f32,
    steering_angle: f32,
    wheels: [Vec2; 4],
    wheel_base: f32,
}

impl Car {
    pub async fn new(x: f32, y: f32) -> Self {
        let wheel_base = 14.0;
        Self {
            texture: load_texture("assets/car.png").await.unwrap(),
            position: vec2(x, y),
            rotation: FRAC_PI_2,
            velocity: 0.0,
            steering_angle: 0.0,
            wheel_base,
            wheels: [
                vec2(4.5, wheel_base),  // front right
                vec2(-4.5, wheel_base), // front left
                vec2(4.5, 0.0),         // rear right
                vec2(-4.5, 0.0),        // rear left
            ],
        }
    }

    pub fn update(&mut self, wheels_on_track: &[bool; 4]) {
        let dt = get_frame_time();
        let (left, right) = (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right));
        let (up, down) = (is_key_down(KeyCode::Up), is_key_down(KeyCode::Down));

        let turn_speed = FRAC_PI_4;
        if left {
            self.steering_angle += turn_speed * dt;
        }
        if right {
            self.steering_angle -= turn_speed * dt;
        }
        if !left && !right {
            self.steering_angle = self.steering_angle.lerp(0.0, (10.0 * dt).clamp(0.0, 1.0));
        }
        self.steering_angle = self.steering_angle.clamp(-FRAC_PI_4, FRAC_PI_4);

        let acceleration = 50.0;
        let penalty = wheels_on_track
            .iter()
            .filter(|&&on_track| !on_track)
            .map(|_| 0.95)
            .product::<f32>();
        let friction = 0.995 * penalty;

        if up {
            self.velocity += acceleration * dt;
        }
        if down {
            self.velocity -= acceleration * dt;
        }
        self.velocity *= friction;

        let pos_dot = Vec2::from_angle(self.rotation) * self.velocity;
        let theta_dot = self.velocity * self.steering_angle.tan() / self.wheel_base;
        self.position += pos_dot * dt;
        self.rotation += theta_dot * dt;
    }

    pub fn draw(&self, wheels_on_track: &[bool; 4]) {
        let draw_rot = self.rotation - FRAC_PI_2;
        let rot_vec = Vec2::from_angle(self.rotation);

        let orientation = Vec2::from_angle(draw_rot);
        for (i, (&wheel, &on_track)) in self.wheels.iter().zip(wheels_on_track).enumerate() {
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
                    color: if on_track { BLACK } else { RED },
                    offset: vec2(0.5, 0.5),
                },
            );
        }

        let texture_pos = (self.position + rot_vec * self.wheel_base / 2.0)
            - vec2(self.texture.width() / 40.0, self.texture.height() / 40.0);
        draw_texture_ex(
            &self.texture,
            texture_pos.x,
            texture_pos.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.texture.size() / 20.0),
                flip_y: true,
                rotation: draw_rot,
                ..Default::default()
            },
        );
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
}
