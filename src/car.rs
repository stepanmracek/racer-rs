use macroquad::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub struct Car {
    texture: Texture2D,
    pub position: Vec2,
    pub rotation: f32,
    velocity: Vec2,
    speed: f32,
}

impl Car {
    pub async fn new(x: f32, y: f32) -> Self {
        Self {
            texture: load_texture("assets/car.png").await.unwrap(),
            position: vec2(x, y),
            rotation: FRAC_PI_2,
            velocity: vec2(0.0, 0.0),
            speed: 0.0,
        }
    }

    pub fn update(&mut self) {
        let rotation_speed = 1.0;
        let acceleration = 100.0;
        let friction = 0.98;
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

    pub fn draw(&self) {
        /*let car_size = vec2(25.0, 11.0);
        draw_rectangle_ex(
            self.position.x,
            self.position.y,
            car_size.x,
            car_size.y,
            DrawRectangleParams {
                offset: vec2(0.5, 0.5),
                rotation: self.rotation,
                color: BLUE,
            },
        );*/
        draw_texture_ex(
            &self.texture,
            self.position.x - self.texture.width() / 40.0,
            self.position.y - self.texture.height() / 40.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.texture.size() / 20.0),
                flip_y: true,
                rotation: self.rotation - FRAC_PI_2,
                ..Default::default()
            },
        );
    }
}
