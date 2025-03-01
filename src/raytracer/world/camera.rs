use nalgebra::{Matrix3, Vector2, Vector3};

use super::objects::Material;
use crate::raytracer::{world::Visible, Direction, Position};

#[derive(Debug)]
pub struct Ray {
    pub position: Position,
    pub dir: Direction,
}

pub struct HitPoint<'a> {
    pub obj: &'a dyn Visible,
    pub position: Position,
}

impl<'a> HitPoint<'a> {
    pub fn new(object: &'a dyn Visible, position: Position) -> Self {
        Self {
            obj: object,
            position,
        }
    }

    pub fn surface_material(&self) -> &Material {
        self.obj.material_of(&self.position)
    }

    pub fn surface_norm(&self) -> Direction {
        self.obj.norm_of(&self.position)
    }
}

impl Ray {
    pub fn new(position: Position, dir: Direction) -> Self {
        Self { position, dir }
    }

    #[allow(non_snake_case)]
    pub fn reflected_ray_from_hit_point(ray: &Self, hit_point: &HitPoint) -> Self {
        let N = hit_point.surface_norm();
        let reflect_dir = ray.dir.reflection(&N);

        let reflect_orig = if reflect_dir.is_acute_angle(&N) {
            hit_point.position.move_forward(1e-3, &N)
        } else {
            hit_point.position.move_forward(-1e-3, &N)
        };

        Self::new(reflect_orig, reflect_dir)
    }

    #[allow(non_snake_case)]
    pub fn refracted_ray_from_hit_point(ray: &Self, hit_point: &HitPoint) -> Self {
        let N = hit_point.surface_norm();
        let refract_dir = ray
            .dir
            .refraction(&N, hit_point.surface_material().refractive_index);

        let refract_orig = if refract_dir.is_acute_angle(&N) {
            hit_point.position.move_forward(1e-3, &N)
        } else {
            hit_point.position.move_forward(-1e-3, &N)
        };

        Self::new(refract_orig, refract_dir)
    }

    pub fn shadow_ray(hit_point: &HitPoint, light_pos: &Position) -> Self {
        let to_light = Direction::a_to_b(&hit_point.position, light_pos);
        // WARN: I change the move direction to to_light, previous I use N
        let shadow_orig = hit_point.position.move_forward(1e-3, &to_light);
        Self::new(shadow_orig, to_light)
    }
}

pub struct Camera {
    pub film_distance: f64,
    pub fov: f64,
    pub position: Position,
    pub forward: Direction,
    pub right: Direction,
    pub up: Direction,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            film_distance: 1.,
            fov: 90.,
            position: Position::new(0., 0., 0.),
            forward: Direction::new(0., 0., -1.),
            right: Direction::new(1., 0., 0.),
            up: Direction::new(0., 1., 0.),
        }
    }
}

#[derive(Default)]
pub struct CameraBuilder(Camera);

impl CameraBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn forward_to(&mut self, forward: Direction) -> &mut Self {
        self.0.forward = forward;
        self
    }

    pub fn up_to(&mut self, up: Direction) -> &mut Self {
        self.0.up = up;
        self
    }

    pub fn right_to(&mut self, right: Direction) -> &mut Self {
        self.0.right = right;
        self
    }

    pub fn adjust_screen(&mut self, dist: f64) -> &mut Self {
        self.0.film_distance = dist;
        self
    }

    pub fn adjust_fov_in_degree(&mut self, degree: f64) -> &mut Self {
        self.0.fov = degree.to_radians();
        self
    }

    pub fn adjust_fov_in_radian(&mut self, radian: f64) -> &mut Self {
        self.0.fov = radian;
        self
    }

    pub fn build(self) -> Camera {
        self.0
    }
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the ray: start from camera to the pixel on film
    pub fn ray_to_pixel(&self, x: f64, y: f64) -> Ray {
        let pixel_pos = Vector3::new(x, y, self.film_distance);
        let mat = Matrix3::from_columns(&[
            *self.right.as_ref(),
            *self.up.as_ref(),
            *self.forward.as_ref(),
        ]);
        Ray::new(self.position, Direction::from(mat * pixel_pos))
    }

    /// Mapping the pixel on canvas to the pixel on the film in front of camera
    pub fn pixel_on_film(&self, idx: usize, img_width: u32, img_height: u32) -> Vector2<f64> {
        // NOTE: map pixel to [-1, 1]
        // a.k.a. normalize device coordinates
        let idx = idx as u32;
        let u = (idx % img_width) as f64;
        let v = (idx / img_width) as f64;
        let w = img_width as f64;
        let h = img_height as f64;

        // NOTE: Apply aspect ratio w/h:
        // so dx_ndc/u and dy_ndc/v is equal
        // x_ndc range is [-w/h, w/h]
        // y_ndc range is [-1, 1]
        let x_ndc = (2. * u - w) / h;
        let y_ndc = (h - 2. * v) / h;

        let tan_fov = (self.fov / 2.).to_radians().tan();

        Vector2::new(
            x_ndc * tan_fov * self.film_distance,
            y_ndc * tan_fov * self.film_distance,
        )
    }
}
