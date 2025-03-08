use super::Ray;
use crate::raytracer::Color;

pub trait Background: Send + Sync {
    fn get_color(&self, ray: &Ray) -> Color;
}

pub struct DummyBackground;

impl Background for DummyBackground {
    fn get_color(&self, ray: &Ray) -> Color {
        let y_proj = ray.dir.as_ref().y;
        let a = y_proj * 0.5 + 0.5;
        a * Color::new(0.6, 0.8, 0.4) + (1. - a) * Color::new(0.8, 0.8, 0.8)
    }
}

pub struct Sky;

impl Background for Sky {
    fn get_color(&self, ray: &Ray) -> Color {
        let y_proj = ray.dir.as_ref().y;
        let a = y_proj * 0.5 + 0.5;
        (1. - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}
