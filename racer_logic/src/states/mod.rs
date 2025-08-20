use crate::environment::Environment;
mod finish;
mod game;
mod init;

pub use init::Init;

pub trait State {
    fn step(&mut self, environment: &mut Environment) -> Option<Box<dyn State>>;
    fn draw(&mut self, environment: &Environment);
}
