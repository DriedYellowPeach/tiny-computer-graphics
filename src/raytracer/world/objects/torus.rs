use crate::raytracer::{Direction, Position};

use super::{Material, Visible};

#[allow(non_snake_case)]
pub struct Torus {
    center: Position,
    // NOTE: r1 is R and r2 is r
    R: f64,
    r: f64,
    material: Material,
}

impl Visible for Torus {
    fn hit_by_ray(&self, _ray: &crate::raytracer::world::Ray) -> Option<f64> {
        todo!()
    }

    fn material_of(&self, _pos: &Position) -> &super::material::Material {
        &self.material
    }

    fn norm_of(&self, pos: &Position) -> crate::raytracer::Direction {
        let rp = pos.as_ref() - self.center.as_ref();

        let sin_theta = rp.z / self.r;
        let cos_theta = (1. - sin_theta.powi(2)).sqrt();
        let cos_phi = rp.x / (self.R + self.r * cos_theta);
        let sin_phi = (1. - cos_phi.powi(2)).sqrt();
        let r_circle_center = Position::new(self.R * cos_phi, self.R * sin_phi, 0.);
        Direction::a_to_b(&r_circle_center, &Position::from(rp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_torus_norm_vector() {
        let torus = Torus {
            center: Position::new(0., 0., 0.),
            R: 2.,
            r: 1.,
            material: Material::default(),
        };

        assert_eq!(
            torus.norm_of(&Position::new(2., 0., 1.)),
            Direction::new(0., 0., 1.)
        );

        assert_eq!(
            torus.norm_of(&Position::new(3., 0., 0.)),
            Direction::new(1., 0., 0.)
        );

        // more trivia angle
        assert_abs_diff_eq!(
            torus.norm_of(&Position::new(2. + 2f64.sqrt() / 2., 0., 2f64.sqrt() / 2.)),
            Direction::new(1., 0., 1.),
        );

        // torus that not at origin
        let torus = Torus {
            center: Position::new(2., 3., 4.),
            R: 2.,
            r: 1.,
            material: Material::default(),
        };

        assert_abs_diff_eq!(
            torus.norm_of(&Position::new(5., 3., 4.)),
            Direction::new(1., 0., 0.)
        );
    }
}
