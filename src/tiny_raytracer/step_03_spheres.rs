use super::pixel_to_world;
use image::{GenericImage, Rgb};
use nalgebra::Vector3;

pub const BACKGROUND_COLOR: Vector3<f64> = Vector3::new(0.2, 0.7, 0.8);
pub const FOV: f64 = 90.;
pub const Z: f64 = 1.;

#[derive(Clone)]
pub struct Material {
    pub diffuse_color: Vector3<f64>,
}

pub struct Sphere {
    pub center: Vector3<f64>,
    pub radius: f64,
    pub mat: Material,
}

impl Sphere {
    pub fn new(center: Vector3<f64>, radius: f64, mat: Material) -> Self {
        Self {
            center,
            radius,
            mat,
        }
    }

    /// Check if the ray intersects with the sphere
    ///
    /// If intersected, return the distance from the light origin to the intersection point
    /// If not, return None
    pub fn ray_intersect(&self, orig: &Vector3<f64>, ray_dir: &Vector3<f64>) -> Option<f64> {
        // NOTE: c is sphere center
        // o is camera origin
        // c_prime is c's projection on light directional vector
        let oc = self.center - orig;
        let o_c_prime_length = oc.dot(ray_dir);
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

/// Calculate the nearest intersection point from `orig`, with direction `ray_dir`, to any point in `spheres`
///
/// Returen the intersected sphere and the intersection point
pub fn scene_intersect<'a>(
    orig: &Vector3<f64>,
    ray_dir: &Vector3<f64>,
    spheres: &'a [Sphere],
) -> Option<(&'a Sphere, Vector3<f64>)> {
    assert!((ray_dir.magnitude() - 1.0).abs() < 1e-6);
    let mut min_hit_dist = f64::MAX;
    let mut ret = None;
    for sphere in spheres {
        if let Some(hit_dist) = sphere.ray_intersect(orig, ray_dir) {
            // NOTE: intersect behind the previous point, ignore this
            if hit_dist >= min_hit_dist {
                continue;
            }
            min_hit_dist = hit_dist;
            let hit_point = orig + ray_dir * hit_dist;
            ret = Some((sphere, hit_point));
        }
    }

    if min_hit_dist > 1000. {
        return None;
    }

    ret
}

pub fn cast_ray(orig: &Vector3<f64>, ray_dir: &Vector3<f64>, spheres: &[Sphere]) -> Vector3<f64> {
    let Some((sphere, _hit_point)) = scene_intersect(orig, ray_dir, spheres) else {
        return BACKGROUND_COLOR;
    };

    sphere.mat.diffuse_color
}

pub fn render<I>(img: &mut I, spheres: &[Sphere])
where
    I: GenericImage<Pixel = Rgb<u8>>,
{
    let width = img.width();
    let height = img.height();
    let v3_to_rgb =
        |v: Vector3<f64>| Rgb([(v.x * 255.) as u8, (v.y * 255.) as u8, (v.z * 255.) as u8]);

    for i in 0..width {
        for j in 0..height {
            let (x, y) = pixel_to_world(i, j, width, height, FOV, Z);
            let ray_dir = Vector3::new(x, y, -1.).normalize();
            let color = cast_ray(&Vector3::new(0., 0., 0.), &ray_dir, spheres);
            img.put_pixel(i, j, v3_to_rgb(color));
        }
    }
}

#[cfg(test)]
mod tests {
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

        let mut img = RgbImage::new(1024, 768);
        render(&mut img, &spheres);
        img.save("output/ray_tracing_step_three_scene.tga").unwrap();
    }
}
