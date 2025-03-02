use super::{material::Material, Visible};
use crate::raytracer::world::Ray;
use crate::raytracer::{Direction, Position};

#[derive(Clone, Debug)]
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
        // NOTE:
        //     ->  ->
        // a=  d * d
        //         ->  ->
        // b = 2 * d * CQ
        //     ->   ->
        // c = CQ * CQ - r * r
        let cq = ray.position.as_ref() - self.center.as_ref();

        let a = ray.dir.dot(&ray.dir);
        let b = 2. * ray.dir.as_ref().dot(&cq);
        let c = cq.dot(&cq) - self.radius.powi(2);

        let descriminant = b.powi(2) - 4. * a * c;

        if descriminant < 0. {
            return None;
        }

        let near = (-b - descriminant.sqrt()) / (2. * a);
        let far = (-b + descriminant.sqrt()) / (2. * a);

        if near < 0. && far < 0. {
            return None;
        }
        //
        // if near < 0. {
        //     return Some(far);
        // }

        Some(near)
    }

    fn material_of(&self, _pos: &Position) -> &Material {
        &self.material
    }

    fn norm_of(&self, pos: &Position) -> Direction {
        Direction::from(pos.as_ref() - self.center.as_ref())
    }
}

pub struct GradientSphere(Sphere);

impl GradientSphere {
    pub fn new(center: Position, radius: f64) -> Self {
        Self(Sphere::new(center, radius, Material::default()))
    }
}

impl Visible for GradientSphere {
    fn hit_by_ray(&self, ray: &Ray) -> Option<f64> {
        self.0.hit_by_ray(ray)
    }

    fn material_of(&self, pos: &Position) -> &super::material::Material {}

    fn norm_of(&self, pos: &Position) -> Direction {
        self.0.norm_of(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_hit_by_ray() {
        // two intersection
        let ray = Ray::new(Position::new(0., 0., 0.), Direction::new(1., 1., 1.));
        let sphere = Sphere::new(Position::new(2., 2., 2.), 1., Material::default());
        let l = 2. * 3f64.sqrt() - 1.;

        assert_abs_diff_eq!(sphere.hit_by_ray(&ray).unwrap(), l, epsilon = 1e-6);

        // no intersection
        let ray = Ray::new(Position::new(0., 0., 0.), Direction::new(0., 0., 1.));
        assert!(sphere.hit_by_ray(&ray).is_none());

        // one
        let ray = Ray::new(Position::new(2., 1., 0.), Direction::new(0., 0., 1.));
        assert_abs_diff_eq!(sphere.hit_by_ray(&ray).unwrap(), 2., epsilon = 1e-6);
    }
}
