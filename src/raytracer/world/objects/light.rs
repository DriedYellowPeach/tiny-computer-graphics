use crate::raytracer::Position;

#[derive(Debug)]
pub struct Light {
    pub position: Position,
    pub intensity: f64,
}

impl Light {
    pub fn new(position: Position, intensity: f64) -> Self {
        Self {
            position,
            intensity,
        }
    }
}
