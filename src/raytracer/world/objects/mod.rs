use super::camera::Ray;
use crate::raytracer::{Direction, Position};

pub mod box_3d;
pub mod light;
pub mod material;
pub mod sphere;
pub mod torus;

pub use light::Light;
pub use material::Material;
pub use sphere::Sphere;

pub trait Visible: Sync + Send {
    /// return the distance from the origin to the hit point
    // FIX: maybe return the hit point position directly?
    fn hit_by_ray(&self, ray: &Ray) -> Option<f64>;

    /// The material of the object on that position
    fn material_of(&self, pos: &Position) -> &material::Material;

    /// The normal vector of hit pos
    fn norm_of(&self, pos: &Position) -> Direction;
}
