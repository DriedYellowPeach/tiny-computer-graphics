pub mod background;
pub mod objects;
pub mod ray;
pub mod scene;

pub use objects::{Light, Visible};
pub use ray::{HitPoint, Ray};
pub use scene::{Lambertian, MonteCarlo, RayCastStrategy, Scene};
