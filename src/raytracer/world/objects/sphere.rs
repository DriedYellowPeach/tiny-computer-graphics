use super::{material::Material, Visible};
use crate::raytracer::world::Ray;
use crate::raytracer::{Direction, Position};

#[derive(Clone)]
pub struct Sphere {
    center: Position,
    radius: f64,
    material: Material,
}

impl Sphere {
    pub fn new(center: Position, radius: f64, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Visible for Sphere {
    fn hit_by_ray(&self, ray: &Ray) -> Option<f64> {
        let oc = self.center.as_ref() - ray.position.as_ref();
        let o_c_prime_length = oc.dot(ray.dir.as_ref());
        let d2 = oc.dot(&oc) - o_c_prime_length.powi(2);

        if d2 > self.radius.powi(2) {
            return None;
        }

        let half_chord_length = (self.radius.powi(2) - d2).sqrt();
        let (near, far) = (
            o_c_prime_length - half_chord_length,
            o_c_prime_length + half_chord_length,
        );

        // NOTE: two intersect points behind the camera
        if near < 0. && far < 0. {
            return None;
        }

        // NOTE: near intersect point behind the camera
        if near < 0. {
            return Some(far);
        }

        Some(near)
    }

    fn material_of(&self, _pos: &Position) -> &Material {
        &self.material
    }

    fn norm_of(&self, pos: &Position) -> Direction {
        Direction::from(pos.as_ref() - self.center.as_ref())
    }
}
