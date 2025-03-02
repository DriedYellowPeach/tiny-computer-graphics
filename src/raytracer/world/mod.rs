pub mod background;
pub mod camera;
pub mod objects;
pub mod ray;
pub mod scene;

pub use camera::Camera;
pub use objects::{Light, Visible};
pub use ray::{HitPoint, Ray};
pub use scene::Scene;
