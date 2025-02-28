use crate::raytracer::{Direction, Position, EPSILON};
use anyhow::{bail, Result};

use super::{Material, Ray, Visible};

// NOTE: Axis Aligned Bounding Box
#[derive(Debug)]
pub struct AABBox {
    low: Position,
    high: Position,
    material: Material,
}

impl AABBox {
    pub fn try_build(low: Position, high: Position, material: Material) -> Result<Self> {
        for i in 0..3 {
            if low.0[i] > high.0[i] {
                bail!("Low position is greater than high position");
            }
        }

        Ok(Self {
            low,
            high,
            material,
        })
    }
}

impl Visible for AABBox {
    fn hit_by_ray(&self, ray: &Ray) -> Option<f64> {
        let mut t_min = f64::MIN;
        let mut t_max = f64::MAX;

        for i in 0..3 {
            let mut t_l = (self.low.0[i] - ray.position.0[i]) / ray.dir.0[i];
            let mut t_h = (self.high.0[i] - ray.position.0[i]) / ray.dir.0[i];

            if t_l > t_h {
                std::mem::swap(&mut t_l, &mut t_h);
            }
            if t_l > t_min {
                t_min = t_l;
            }

            if t_h < t_max {
                t_max = t_h;
            }
        }

        if t_min > t_max || t_min < 0. {
            return None;
        }

        Some(t_min)
    }

    fn material_of(&self, _pos: &Position) -> &Material {
        &self.material
    }

    fn norm_of(&self, pos: &Position) -> Direction {
        // test if on slab perpendicular to x axis
        if (pos.0.x - self.low.0.x).abs() < EPSILON {
            return Direction::new(-1.0, 0.0, 0.0);
        }

        if (pos.0.x - self.high.0.x).abs() < EPSILON {
            return Direction::new(1.0, 0.0, 0.0);
        }

        if (pos.0.y - self.low.0.y).abs() < EPSILON {
            return Direction::new(0.0, -1.0, 0.0);
        }

        if (pos.0.y - self.high.0.y).abs() < EPSILON {
            return Direction::new(0.0, 1.0, 0.0);
        }

        if (pos.0.z - self.low.0.z).abs() < EPSILON {
            return Direction::new(0.0, 0.0, -1.0);
        }

        if (pos.0.z - self.high.0.z).abs() < EPSILON {
            return Direction::new(0.0, 0.0, 1.0);
        }

        Direction::new(0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_aabb_ray_intersect() {
        let test_cases = [
            (
                AABBox::try_build(
                    Position::new(5., -1., -1.),
                    Position::new(7., 1., 1.),
                    Material::IVORY,
                )
                .unwrap(),
                Ray::new(Position::new(0., 0., 0.), Direction::new(1., 0., 0.)),
                Some(5.),
            ),
            (
                AABBox::try_build(
                    Position::new(5., -1., -1.),
                    Position::new(7., 1., 1.),
                    Material::IVORY,
                )
                .unwrap(),
                Ray::new(Position::new(0., 0., 0.), Direction::new(0., 0., 1.)),
                None,
            ),
            (
                AABBox::try_build(
                    Position::new(1., 1., 1.),
                    Position::new(2., 2., 2.),
                    Material::IVORY,
                )
                .unwrap(),
                Ray::new(Position::new(0., 0., 0.), Direction::new(1., 1., 1.)),
                Some(3f64.sqrt()),
            ),
        ];

        for (bbox, ray, expected) in test_cases.into_iter() {
            let output = bbox.hit_by_ray(&ray);
            match (output, expected) {
                (Some(o), Some(e)) => assert_abs_diff_eq!(o, e),
                (Some(_), None) | (None, Some(_)) => {
                    panic!("Input: {bbox:#?}, {ray:#?}\n Expected {expected:?}, Got {output:?}")
                }
                (None, None) => {}
            }
        }
    }

    #[test]
    fn test_aabb_norm() {
        let bbox = AABBox::try_build(
            Position::new(5., 5., 5.),
            Position::new(6., 6., 6.),
            Material::IVORY,
        )
        .unwrap();

        let text_cases = [
            // x axis
            (Position::new(5.0, 5.5, 5.5), Direction::new(-1., 0., 0.)),
            (Position::new(5.0, 5.5, 5.5), Direction::new(-1., 0., 0.)),
            // y axis
            (Position::new(5.0, 5.5, 5.5), Direction::new(-1., 0., 0.)),
            (Position::new(5.0, 5.5, 5.5), Direction::new(-1., 0., 0.)),
            // z axis
            (Position::new(5.0, 5.5, 5.5), Direction::new(-1., 0., 0.)),
            (Position::new(5.0, 5.5, 5.5), Direction::new(-1., 0., 0.)),
            // slightly off position
            (Position::new(5.0, 5.5, 5.5), Direction::new(-1., 0., 0.)),
        ];

        for (p, r) in text_cases.into_iter() {
            let output = bbox.norm_of(&p);
            assert_eq!(
                output, r,
                "Input: {p:#?}, Expected: {r:#?}, Got: {output:#?}"
            );
        }
    }
}
