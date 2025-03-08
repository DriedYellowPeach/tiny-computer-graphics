pub mod basics;
pub mod camera;
pub mod world;

pub use basics::*;

pub mod prelude {
    pub use super::{
        basics::*,
        camera::{Camera, CameraBuilder},
        world::{background::DummyBackground, objects::*, scene::Scene},
    };
}
