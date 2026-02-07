use crate::{controller::Controller, environment::Environment, follow_camera::FollowCamera};
mod finish;
mod init;
mod race;

pub use init::Init;

pub trait State {
    fn step(
        &mut self,
        environment: &mut Environment,
        controllers: &mut [Box<dyn Controller>],
        follow_camera: &mut FollowCamera,
    ) -> Option<Box<dyn State>>;

    fn draw(&mut self, environment: &Environment, follow_camera: &mut FollowCamera);
}
