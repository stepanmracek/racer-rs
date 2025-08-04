use macroquad::prelude::*;

mod car;
mod track;

#[macroquad::main("racer")]
async fn main() {
    let mut car = car::Car::new(0.0, 0.0).await;
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

    set_camera(&camera);

    loop {
        clear_background(DARKGREEN);
        let dt = get_frame_time();

        if is_key_pressed(KeyCode::Space) {
            track.add_random_shape();
            track.compute_rtree();
        }
        let on_track = track.on_track(&car.position);
        car.update(on_track);

        camera.rotation = camera.rotation.lerp(-car.rotation.to_degrees() + 90.0, dt);
        camera.target = camera.target.lerp(car.position + vec2(0.0, 50.0), dt);
        camera.zoom = vec2(zoom / screen_width(), -zoom / screen_height());
        set_camera(&camera);

        let rect = Rect::new(car.position.x - 300.0, car.position.y - 200.0, 600.0, 400.0);
        // draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, WHITE);
        track.draw(&rect);
        car.draw();
        /*if on_track {
            draw_circle(car.position.x, car.position.y, 3.0, YELLOW);
        }*/

        next_frame().await;
    }
}
