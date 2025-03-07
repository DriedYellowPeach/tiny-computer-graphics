use image::{Rgb, RgbImage};
use indicatif::ParallelProgressIterator;
use nalgebra::{Matrix3, Vector2, Vector3};
use rayon::{iter::ParallelIterator, prelude::*};

use crate::raytracer::{progress_bar_style, world::Ray, Direction, Position};

use super::world::{background::Background, Scene};

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
    fn film_pixel(&self, idx: usize, img_width: u32, img_height: u32) -> Vector2<f64> {
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

    pub fn render<B: Background>(&self, scene: &Scene<B>, img: &mut RgbImage) {
        let width = img.width();
        let height = img.height();

        img.par_pixels_mut()
            .progress_with_style(progress_bar_style())
            .enumerate()
            .for_each(|(idx, pixel)| {
                let pxl = self.film_pixel(idx, width, height);
                let ray = self.ray_to_pixel(pxl.x, pxl.y);
                let color = scene.cast_ray(&ray, 0);

                *pixel = Rgb::from(color);
            });
    }
}
