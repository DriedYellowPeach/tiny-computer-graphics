use std::borrow::Cow;

use super::objects::Material;
use crate::raytracer::{world::Visible, Direction, Position};

pub struct HitPoint<'a> {
    pub obj: &'a dyn Visible,
    pub position: Position,
    pub is_outside: bool,
}

impl<'a> HitPoint<'a> {
    pub fn new(object: &'a dyn Visible, position: Position, is_outside: bool) -> Self {
        Self {
            obj: object,
            position,
            is_outside,
        }
    }

    pub fn surface_material(&self) -> Cow<'_, Material> {
        self.obj.material_of(&self.position)
    }

    pub fn surface_norm(&self) -> Direction {
        let norm = self.obj.norm_of(&self.position);
        if self.is_outside {
            norm
        } else {
            norm.reverse()
        }
    }
}

#[derive(Debug)]
pub struct Ray {
    pub position: Position,
    pub dir: Direction,
}

impl Ray {
    pub fn new(position: Position, dir: Direction) -> Self {
        Self { position, dir }
    }

    pub fn at(&self, t: f64) -> Position {
        Position::from(self.position.as_ref() + t * self.dir.as_ref())
    }

    #[allow(non_snake_case)]
    pub fn reflected(&self, hit_point: &HitPoint) -> Self {
        let N = hit_point.surface_norm();
        let reflect_dir = self.dir.reflection(&N);

        let reflect_orig = if reflect_dir.is_acute_angle(&N) {
            hit_point.position.move_forward(1e-3, &N)
        } else {
            hit_point.position.move_forward(-1e-3, &N)
        };

        Self::new(reflect_orig, reflect_dir)
    }

    #[allow(non_snake_case)]
    pub fn refracted(&self, hit_point: &HitPoint) -> Self {
        let N = hit_point.surface_norm();
        let mut n1 = 1.;
        let mut n2 = hit_point.surface_material().refractive_index;

        if !hit_point.is_outside {
            std::mem::swap(&mut n1, &mut n2);
        };

        let refract_dir = self.dir.refraction(&N, n1, n2);

        let refract_orig = if refract_dir.is_acute_angle(&N) {
            hit_point.position.move_forward(1e-3, &N)
        } else {
            hit_point.position.move_forward(-1e-3, &N)
        };

        Self::new(refract_orig, refract_dir)
    }

    pub fn shadowed(hit_point: &HitPoint, light_pos: &Position) -> Self {
        let to_light = Direction::a_to_b(&hit_point.position, light_pos);
        // WARN: I change the move direction to to_light, previous I use N
        let shadow_orig = hit_point.position.move_forward(1e-3, &to_light);
        Self::new(shadow_orig, to_light)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_ray_at() {
        let ray = Ray::new(Position::new(0., 0., 0.), Direction::new(1., 0., 0.));
        assert_abs_diff_eq!(ray.at(3.), Position::new(3., 0., 0.));

        let ray = Ray::new(Position::new(1., 1., 1.), Direction::new(1., 1., 1.));
        assert_abs_diff_eq!(ray.at(3f64.sqrt()), Position::new(2., 2., 2.));
    }
}
