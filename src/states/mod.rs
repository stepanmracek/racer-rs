use crate::world::World;
mod finish;
mod game;
mod init;

pub use init::Init;

pub trait State {
    fn step(&mut self, world: &mut World) -> Option<Box<dyn State>>;
}
