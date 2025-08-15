use macroquad::prelude::*;

mod car;
mod follow_camera;
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
    follow_camera: follow_camera::FollowCamera,
}

impl Game {
    fn new(world: &World) -> Self {
        let follow_camera = follow_camera::FollowCamera::new(&world.car);
        Self { follow_camera }
    }

    fn draw_stopwatch(&self) {
        set_default_camera();
        let time = (get_time() * 100.0) as usize;
        let hundrets = time % 100;
        let seconds = (time / 100) % 60;
        let minutes = time / 6000;
        let stopwatch = format!("{minutes:02}:{seconds:02}:{hundrets:02}");
        draw_text(&stopwatch, 5.0, 24.0, 32.0, WHITE);
    }
}

impl State for Game {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>> {
        clear_background(DARKGREEN);

        let wheels_on_track = world.car.wheels_on_track(&world.track);
        world.car.update(&wheels_on_track);
        self.follow_camera.update(&world.car);
        world.track.draw(&world.car);
        world.car.draw();
        self.draw_stopwatch();

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
