/// Part 1: understandable raytracing
/// Step 2: define a Sphere and check if ray intersects with this sphere
use image::{GenericImage, Rgb};
use nalgebra::Vector3;

pub struct Sphere {
    center: Vector3<f64>,
    radius: f64,
}

impl Sphere {
    pub fn new(center: Vector3<f64>, radius: f64) -> Self {
        Self { center, radius }
    }

    /// Check if the ray intersects with the sphere
    ///
    /// If intersected, return the distance from the light origin to the intersection point
    /// If not, return None
    pub fn ray_intersect(&self, orig: &Vector3<f64>, light_dir: &Vector3<f64>) -> Option<f64> {
        // NOTE: c is sphere center
        // o is camera origin
        // c_prime is c's projection on light directional vector
        let oc = self.center - orig;
        let o_c_prime_length = oc.dot(light_dir);
        let d2 = oc.dot(&oc) - o_c_prime_length.powi(2);

        if d2 > self.radius.powi(2) {
            return None;
        }

        let half_chord_length = (self.radius.powi(2) - d2).sqrt();
        let (near, far) = (
            o_c_prime_length - half_chord_length,
            o_c_prime_length + half_chord_length,
        );

        // NOTE: two intersect points behind the camera
        if near < 0. && far < 0. {
            return None;
        }

        // NOTE: near intersect point behind the camera
        if near < 0. {
            return Some(far);
        }

        Some(near)
    }
}

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

pub fn cast_ray(orig: &Vector3<f64>, light_dir: &Vector3<f64>, sphere: &Sphere) -> [u8; 3] {
    if sphere.ray_intersect(orig, light_dir).is_none() {
        return [128, 128, 0];
    }

    [255, 0, 0]
}

/// Map pixel to world coordinate
///
/// fov is the field of view in degree
pub fn pixel_to_world(
    u: u32,
    v: u32,
    width: u32,
    height: u32,
    fov: f64,
    screen_dist: f64,
) -> (f64, f64) {
    // NOTE: map pixel to [-1, 1]
    // a.k.a. normalize device coordinates
    let u = u as f64;
    let v = v as f64;
    let w = width as f64;
    let h = height as f64;

    // NOTE: apply aspect ratio w/h
    // so dx_ndc/u and dy_ndc/v is equal
    let x_ndc = (2. * u / w - 1.) * w / h;
    let y_ndc = 1. - 2. * v / h;

    let tan_fov = (fov * 0.5).to_radians().tan();

    (x_ndc * tan_fov * screen_dist, y_ndc * tan_fov * screen_dist)
}

pub fn render<I>(img: &mut I, sphere: &Sphere)
where
    I: GenericImage<Pixel = Rgb<u8>>,
{
    let width = img.width();
    let height = img.height();
    println!("width: {}, height: {}", width, height);

    for i in 0..width {
        for j in 0..height {
            let (x, y) = pixel_to_world(i, j, width, height, 60., 1.);
            let light_dir = Vector3::new(x, y, -1.).normalize();
            let color = cast_ray(&Vector3::new(0., 0., 0.), &light_dir, sphere);
            img.put_pixel(i, j, Rgb(color));
        }
    }
}

#[cfg(test)]
pub mod tests {
    use image::RgbImage;

    use super::*;

    #[test]
    fn test_render() {
        let mut img = RgbImage::new(800, 800);
        render(&mut img, &Sphere::new(Vector3::new(200., 200., -600.), 50.));

        img.save("output/ray_tracing_step_two_scene.tga").unwrap();
    }

    #[test]
    fn test_render_img_with_height_width_different() {
        let mut img = RgbImage::new(1024, 768);
        render(&mut img, &Sphere::new(Vector3::new(200., 200., -600.), 30.));

        img.save("output/ray_tracing_step_two_scene2.tga").unwrap();
    }

    #[test]
    fn test_ray_intersect() {
        let sphere = Sphere::new(Vector3::new(0., 0., -50.), 20.);
        let orig = Vector3::new(0., 0., 0.);
        let dir = Vector3::new(0., 0., -1.);
        assert_eq!(sphere.ray_intersect(&orig, &dir), Some(30.));
    }
}
