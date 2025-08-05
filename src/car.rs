use macroquad::prelude::*;
use std::f32::consts::FRAC_PI_2;

use crate::track::Track;

pub struct Car {
    texture: Texture2D,
    pub position: Vec2,
    pub rotation: f32,
    velocity: Vec2,
    speed: f32,
    wheels: [Vec2; 4],
}

impl Car {
    pub async fn new(x: f32, y: f32) -> Self {
        Self {
            texture: load_texture("assets/car.png").await.unwrap(),
            position: vec2(x, y),
            rotation: FRAC_PI_2,
            velocity: vec2(0.0, 0.0),
            speed: 0.0,
            wheels: [
                vec2(5.0, 7.0),
                vec2(-5.0, 7.0),
                vec2(5.0, -7.0),
                vec2(-5.0, -7.0),
            ],
        }
    }

    pub fn update(&mut self, wheels_on_track: &[bool; 4]) {
        let rotation_speed = 1.0 * self.speed.signum();
        let acceleration = 100.0;
        let penalty = wheels_on_track
            .iter()
            .filter(|&&on_track| !on_track)
            .map(|_| 0.95)
            .product::<f32>();
        let friction = 0.98 * penalty;
        let dt = get_frame_time();

        if is_key_down(KeyCode::Up) {
            self.speed += acceleration * dt;
        }
        if is_key_down(KeyCode::Down) {
            self.speed -= acceleration * dt;
        }

        // Rotation only when moving
        if self.speed.abs() > 5.0 {
            if is_key_down(KeyCode::Left) {
                self.rotation += rotation_speed * dt;
            }
            if is_key_down(KeyCode::Right) {
                self.rotation -= rotation_speed * dt;
            }
        }

        self.speed *= friction;
        self.velocity = vec2(self.rotation.cos(), self.rotation.sin()) * self.speed;
        self.position += self.velocity * dt;
    }

    pub fn draw(&self, wheels_on_track: &[bool; 4]) {
        let draw_rot = self.rotation - FRAC_PI_2;
        draw_texture_ex(
            &self.texture,
            self.position.x - self.texture.width() / 40.0,
            self.position.y - self.texture.height() / 40.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.texture.size() / 20.0),
                flip_y: true,
                rotation: draw_rot,
                ..Default::default()
            },
        );

        let orientation = Vec2::from_angle(draw_rot);
        for (&wheel, &on_track) in self.wheels.iter().zip(wheels_on_track) {
            let pos = self.position + orientation.rotate(wheel);
            if on_track {
                draw_circle(pos.x, pos.y, 1.5, YELLOW);
            }
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
}
