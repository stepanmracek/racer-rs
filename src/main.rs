use std::f32::consts::FRAC_PI_2;

use macroquad::prelude::*;

use crate::physics::RotRect;

mod car;
mod physics;
mod track;

#[macroquad::main("racer")]
async fn main() {
    let mut car = car::Car::new(0.0, 15.0).await;
    let mut track = track::Track::new();
    for _ in 0..100_000 {
        track.add_random_shape();
    }
    track.compute_rtree();

    let zoom = 8.0;
    let mut camera = Camera2D {
        target: car.position,
        zoom: vec2(zoom / screen_width(), -zoom / screen_height()),
        rotation: -car.rotation.to_degrees() + 90.0,
        ..Default::default()
    };

    let collider = RotRect::new(vec2(0.0, 64.0), vec2(15.0, 42.0), 0.1);

    set_camera(&camera);

    loop {
        clear_background(DARKGREEN);
        let dt = get_frame_time();

        if is_key_pressed(KeyCode::Space) {
            track.add_random_shape();
            track.compute_rtree();
        }
        let wheels_on_track = car.wheels_on_track(&track);
        car.update(&wheels_on_track);

        let car_rotation = car.rotation - FRAC_PI_2;
        let shift = Vec2::from_angle(car_rotation).rotate(vec2(0.0, 50.0));
        camera.rotation = camera.rotation.lerp(-car_rotation.to_degrees(), dt);
        camera.target = camera.target.lerp(car.position + shift, 5.0 * dt);
        camera.zoom = vec2(zoom / screen_width(), -zoom / screen_height());
        set_camera(&camera);

        let rect = Rect::new(car.position.x - 300.0, car.position.y - 200.0, 600.0, 400.0);
        // draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, WHITE);
        track.draw(&rect);
        car.draw(&wheels_on_track);

        let collider_color = if collider.collide(car.bbox()) {
            RED
        } else {
            GREEN
        };
        draw_rectangle_lines_ex(
            collider.center().x,
            collider.center().y,
            collider.size().x,
            collider.size().y,
            1.0,
            DrawRectangleParams {
                offset: vec2(0.5, 0.5),
                rotation: *collider.rotation(),
                color: collider_color,
            },
        );

        set_default_camera();
        let time = (get_time() * 100.0) as usize;
        let hundrets = time % 100;
        let seconds = (time / 100) % 60;
        let minutes = time / 6000;
        let stopwatch = format!("{minutes:02}:{seconds:02}:{hundrets:02}");
        draw_text(&stopwatch, 5.0, 24.0, 32.0, WHITE);

        next_frame().await;
    }
}
