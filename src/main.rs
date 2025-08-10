use macroquad::prelude::*;
use std::f32::consts::FRAC_PI_2;

mod car;
mod physics;
mod track;

struct World {
    track: track::Track,
    car: car::Car,
}

impl World {
    async fn new() -> Self {
        let car = car::Car::new(0.0, 15.0).await;
        let mut track = track::Track::new();
        for _ in 0..3 {
            track.add_random_shape();
        }
        track.add_finish();
        track.compute_rtree();
        Self { car, track }
    }
}

trait State {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>>;
}

struct Init {}

impl State for Init {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        set_default_camera();
        draw_text("Press space to start", 5.0, 24.0, 32.0, WHITE);
        if is_key_pressed(KeyCode::Space) {
            Some(Box::new(Game::new(world)))
        } else {
            None
        }
    }
}

struct Game {
    camera: Camera2D,
    zoom: f32,
}

impl Game {
    fn new(world: &World) -> Self {
        let zoom = 8.0;
        let camera = Camera2D {
            target: world.car.position,
            zoom: vec2(zoom / screen_width(), -zoom / screen_height()),
            rotation: -world.car.rotation.to_degrees() + 90.0,
            ..Default::default()
        };
        Self { camera, zoom }
    }
}

impl State for Game {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        clear_background(DARKGREEN);
        let dt = get_frame_time();

        if is_key_pressed(KeyCode::Space) {
            world.track.add_random_shape();
            world.track.compute_rtree();
        }
        let wheels_on_track = world.car.wheels_on_track(&world.track);
        world.car.update(&wheels_on_track);

        let car_rotation = world.car.rotation - FRAC_PI_2;
        let shift = Vec2::from_angle(car_rotation).rotate(vec2(0.0, 50.0));
        //camera.rotation = camera.rotation.lerp(-car_rotation.to_degrees(), dt);
        self.camera.target = self
            .camera
            .target
            .lerp(world.car.position + shift, 5.0 * dt);
        self.camera.zoom = vec2(self.zoom / screen_width(), -self.zoom / screen_height());
        set_camera(&self.camera);

        let rect = Rect::new(
            world.car.position.x - 300.0,
            world.car.position.y - 200.0,
            600.0,
            400.0,
        );
        // draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 3.0, WHITE);
        world.track.draw(&rect);
        world.car.draw();

        set_default_camera();
        let time = (get_time() * 100.0) as usize;
        let hundrets = time % 100;
        let seconds = (time / 100) % 60;
        let minutes = time / 6000;
        let stopwatch = format!("{minutes:02}:{seconds:02}:{hundrets:02}");
        draw_text(&stopwatch, 5.0, 24.0, 32.0, WHITE);

        if world.track.finish(world.car.bbox()) {
            Some(Box::new(Finish {}))
        } else {
            None
        }
    }
}

struct Finish {}

impl State for Finish {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        set_default_camera();
        draw_text("FINISH!", 5.0, 24.0, 32.0, WHITE);
        None
    }
}

#[macroquad::main("racer")]
async fn main() {
    let mut world = World::new().await;
    let mut state: Box<dyn State> = Box::new(Init {});

    loop {
        if let Some(next_state) = state.step(&mut world) {
            state = next_state;
        }

        next_frame().await;
    }
}
