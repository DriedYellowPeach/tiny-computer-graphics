pub mod background;
pub mod camera;
pub mod objects;
pub mod scene;

pub use camera::{Camera, HitPoint, Ray};
pub use objects::{Light, Visible};
pub use scene::Scene;
