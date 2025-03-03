use super::Ray;
use crate::raytracer::{Direction, Interval, Position};

use std::borrow::Cow;

pub mod box_3d;
pub mod light;
pub mod material;
pub mod sphere;
pub mod torus;

pub use light::Light;
pub use material::Material;
pub use sphere::{GradientSphere, Sphere};

pub trait Visible: Sync + Send {
    /// return the distance from the origin to the hit point
    // PERF: give another bbox1D to accelerate the hit test
    fn hit_by_ray(&self, ray: &Ray, interval: &Interval) -> Option<f64>;

    /// The material of the object on that position
    fn material_of(&self, pos: &Position) -> Cow<'_, material::Material>;

    /// The normal vector of hit pos
    fn norm_of(&self, pos: &Position) -> Direction;
}
