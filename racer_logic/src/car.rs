use itertools::Itertools;
use macroquad::prelude::*;
use std::{
    collections::VecDeque,
    f32::consts::{FRAC_PI_2, FRAC_PI_6},
};

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
    skid_marks: std::collections::VecDeque<(Vec2, Vec2)>,
    skidding: bool,
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
            skid_marks: VecDeque::new(),
            skidding: false,
        }
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_velocity(mut self, velocity: f32) -> Self {
        self.velocity = velocity;
        self
    }

    pub async fn load_texture(&mut self) {
        self.texture = Some(load_texture("assets/car1.png").await.unwrap());
    }

    pub fn reset(&mut self, position: &Vec2, rotation: f32, velocity: f32) {
        self.position = *position;
        self.rotation = rotation;
        self.velocity = velocity;
        self.steering_angle = 0.0
    }

    #[inline]
    fn max_steer(&self) -> f32 {
        let velocity = self.velocity.abs();
        if velocity > 170.0 {
            return 0.0;
        }
        FRAC_PI_6 * (1.0 - (velocity / 170.0).powi(2))
    }

    #[inline]
    fn dt(fixed: bool) -> f32 {
        if fixed { 1.0 / 60.0 } else { get_frame_time() }
    }

    pub fn update(
        &mut self,
        wheels_on_track: &[bool; 4],
        steer: f32,
        throttle: f32,
        fixed_time: bool,
    ) {
        let dt = Car::dt(fixed_time);
        let turn_speed = FRAC_PI_6;

        self.steering_angle += steer * turn_speed * dt;
        if steer == 0.0 {
            self.steering_angle = self.steering_angle.lerp(0.0, (10.0 * dt).clamp(0.0, 1.0));
        }
        self.steering_angle = self.steering_angle.clamp(-FRAC_PI_6, FRAC_PI_6);

        let penalty = wheels_on_track
            .iter()
            .filter(|&&on_track| !on_track)
            .map(|_| 0.99)
            .product::<f32>();
        let friction = 0.995 * penalty;

        let acceleration = 50.0;
        self.velocity += throttle * acceleration * dt;
        self.velocity *= friction;

        // skidding ?
        let max_steering = self.max_steer();
        if self.steering_angle.abs() > max_steering {
            self.skidding = true;
            self.steering_angle = self.steering_angle.clamp(-max_steering, max_steering);
            let wheel0 = self.relative_position(&self.wheels[0]);
            let wheel1 = self.relative_position(&self.wheels[1]);
            self.skid_marks.push_back((wheel0, wheel1));
            if self.skid_marks.len() > 100 {
                self.skid_marks.pop_front();
            }
        }

        let pos_dot = Vec2::from_angle(self.rotation) * self.velocity;
        let theta_dot = self.velocity * self.steering_angle.tan() / self.wheel_base;
        self.position += pos_dot * dt;
        self.rotation += theta_dot * dt;

        self.bbox.update(
            self.position_with_offset(self.wheel_base / 2.0),
            self.rotation - FRAC_PI_2,
        );
    }

    pub fn draw(&self) {
        self.skid_marks.iter().tuple_windows().for_each(|(a, b)| {
            if a.0.distance_squared(b.0) < 25.0 {
                draw_line(a.0.x, a.0.y, b.0.x, b.0.y, 1.5, BLACK.with_alpha(0.5));
                draw_line(a.1.x, a.1.y, b.1.x, b.1.y, 1.5, BLACK.with_alpha(0.5));
            }
        });

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

    #[inline]
    pub fn bbox(&self) -> &RotRect {
        &self.bbox
    }

    pub fn sensor_rays(&self, sensor_len: f32) -> Vec<(Vec2, Vec2)> {
        let start = self.windshield_position();
        (-180..180)
            .step_by(20)
            .map(|delta| {
                let angle = Vec2::from_angle(self.rotation + (delta as f32).to_radians());
                let start = start + angle.rotate(vec2(5.0, 0.0));
                let end = start + angle.rotate(vec2(sensor_len, 0.0));
                (start, end)
            })
            .collect()
    }

    #[inline]
    pub fn position(&self) -> &Vec2 {
        &self.position
    }

    #[inline]
    pub fn position_with_offset(&self, offset: f32) -> Vec2 {
        self.position + Vec2::from_angle(self.rotation) * offset
    }

    #[inline]
    pub fn relative_position(&self, pos: &Vec2) -> Vec2 {
        self.position + Vec2::from_angle(self.rotation - FRAC_PI_2).rotate(*pos)
    }

    #[inline]
    pub fn windshield_position(&self) -> Vec2 {
        self.position_with_offset(10.0)
    }

    #[inline]
    pub fn rotation(&self) -> &f32 {
        &self.rotation
    }

    #[inline]
    pub fn velocity(&self) -> &f32 {
        &self.velocity
    }

    #[inline]
    pub fn steering_angle(&self) -> &f32 {
        &self.steering_angle
    }

    #[inline]
    pub fn skidding(&self) -> &bool {
        &self.skidding
    }
}
