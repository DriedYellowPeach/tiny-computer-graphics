use super::Ray;
use crate::raytracer::Color;

pub trait Background: Send + Sync {
    fn get_color(&self, ray: &Ray) -> Color;
}

pub struct DummyBackground;

impl Background for DummyBackground {
    fn get_color(&self, _ray: &Ray) -> Color {
        Color::new(0.6, 0.8, 0.4)
    }
}
