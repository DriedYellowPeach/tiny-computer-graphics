use super::{
    pixel_to_world,
    step_03_spheres::{scene_intersect, Sphere},
    BACKGROUND_COLOR, FOV, Z,
};
use image::{GenericImage, Rgb};
use nalgebra::Vector3;

pub struct Light {
    pub position: Vector3<f64>,
    pub intensity: f64,
}

impl Light {
    pub fn new(position: Vector3<f64>, intensity: f64) -> Self {
        Self {
            position,
            intensity,
        }
    }
}

pub fn cast_ray(
    orig: &Vector3<f64>,
    ray_dir: &Vector3<f64>,
    spheres: &[Sphere],
    lights: &[Light],
) -> Vector3<f64> {
    let Some((sphere, hit_point)) = scene_intersect(orig, ray_dir, spheres) else {
        return BACKGROUND_COLOR;
    };

    let mut diffuse_light_intensity = 0.;
    for light in lights {
        let light_dir = (light.position - hit_point).normalize();
        let norm = (hit_point - sphere.center).normalize();
        diffuse_light_intensity += light.intensity * light_dir.dot(&norm).max(0.);
    }

    sphere.mat.diffuse_color * diffuse_light_intensity
}

pub fn render<I>(img: &mut I, spheres: &[Sphere], lights: &[Light])
where
    I: GenericImage<Pixel = Rgb<u8>>,
{
    let width = img.width();
    let height = img.height();
    let v3_to_rgb =
        |v: Vector3<f64>| Rgb([(v.x * 255.) as u8, (v.y * 255.) as u8, (v.z * 255.) as u8]);

    let orig = Vector3::new(0., 0., 0.);

    for i in 0..width {
        for j in 0..height {
            let (x, y) = pixel_to_world(i, j, width, height, FOV, Z);
            let ray_dir = Vector3::new(x, y, -1.).normalize();
            let color = cast_ray(&orig, &ray_dir, spheres, lights);
            img.put_pixel(i, j, v3_to_rgb(color));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::step_03_spheres::Material;
    use super::*;
    use image::RgbImage;

    #[test]
    fn test_render_multiple_sphere() {
        let ivory = Material {
            diffuse_color: Vector3::new(0.4, 0.4, 0.3),
        };
        let red_rubber = Material {
            diffuse_color: Vector3::new(0.3, 0.1, 0.1),
        };
        let gold = Material {
            diffuse_color: Vector3::new(0.6, 0.5, 0.3),
        };

        let spheres = [
            Sphere::new(Vector3::new(-3., 0., -16.), 2., ivory.clone()),
            Sphere::new(Vector3::new(-1., -1.5, -12.), 2., red_rubber.clone()),
            Sphere::new(Vector3::new(1.5, -0.5, -18.), 3., red_rubber.clone()),
            Sphere::new(Vector3::new(7., 5., -18.), 4., gold.clone()),
        ];

        let lights = [Light::new(Vector3::new(-20., 20., 20.), 1.5)];

        let mut img = RgbImage::new(1024, 768);
        render(&mut img, &spheres, &lights);
        img.save("output/ray_tracing_step_four_scene.tga").unwrap();
    }
}
