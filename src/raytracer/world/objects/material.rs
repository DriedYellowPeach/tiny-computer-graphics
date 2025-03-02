use crate::raytracer::{Albedo, Color};

#[derive(Clone, Debug, Default)]
pub struct Material {
    pub diffuse_color: Color,
    // NOTE: albedo represents reflectivity of the surface
    // albedo.x: on diffuse light
    // albedo.y: on specular light
    // albedo.z: on reflection light
    // albedo.w: on refraction light
    pub albedo: Albedo,
    pub specular_exponent: f64,
    pub refractive_index: f64,
}

impl Material {
    pub const fn new(
        diffuse_color: Color,
        albedo: Albedo,
        specular_exponent: f64,
        refractive_index: f64,
    ) -> Self {
        Self {
            diffuse_color,
            albedo,
            specular_exponent,
            refractive_index,
        }
    }

    pub const IVORY: Material = Material::new(
        Color::new(0.4, 0.4, 0.3),
        Albedo::new(0.6, 0.3, 0.1, 0.0),
        50.,
        1.,
    );
}
