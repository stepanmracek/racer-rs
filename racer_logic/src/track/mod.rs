mod segment;
mod shape;
#[allow(clippy::module_inception)]
mod track;

pub use track::{Track, distances_to_segments};
