/// step 5: apply phong light model, add specular lighting
///
///
///
use image::{GenericImage, Pixel, Rgb, RgbImage};
use nalgebra::{Vector2, Vector3};
use rayon::prelude::*;

use super::{pixel_to_world, step_04_lighting::Light, BACKGROUND_COLOR, FOV, Z};

#[derive(Clone)]
pub struct Material {
    pub diffuse_color: Vector3<f64>,
    pub albedo: Vector2<f64>,
    pub specular_exponent: f64,
}

impl Material {
    pub fn new(diffuse_color: Vector3<f64>, albedo: Vector2<f64>, specular_exponent: f64) -> Self {
        Self {
            diffuse_color,
            albedo,
            specular_exponent,
        }
    }
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

/// Calculate the reflaction vector
///
/// `I` is the vector from hit point to the light source
/// `N` is the normal vector on the hit point
#[allow(non_snake_case)]
pub fn reflection(I: &Vector3<f64>, N: &Vector3<f64>) -> Vector3<f64> {
    assert!((N.magnitude() - 1.0).abs() <= 1e-6);
    let I_proj = I.dot(N) * N;

    I - 2. * I_proj
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
    let mut specular_light_intensity = 0.;
    for light in lights {
        // NOTE: this light_dir start at hit point
        let light_dir = (light.position - hit_point).normalize();
        let norm = (hit_point - sphere.center).normalize();
        // NOTE:
        // -light_dir point at hit point
        // reflaction start from hit point
        // reverse toward hit point, so as ray_dir
        let reverse_reflect_light_dir = -reflection(&(-light_dir), &norm);
        // NOTE:
        // The larger specular_exponent, the smaller the specular light intensity
        // makes the specular lighting more concentrated
        let to_expo = ray_dir
            .dot(&reverse_reflect_light_dir)
            .max(0.)
            .powf(sphere.mat.specular_exponent);
        diffuse_light_intensity += light.intensity * light_dir.dot(&norm).max(0.);
        specular_light_intensity += light.intensity * to_expo;
    }

    let albedo = sphere.mat.albedo;
    let white = Vector3::new(1., 1., 1.);

    sphere.mat.diffuse_color * diffuse_light_intensity * albedo.x
        + white * specular_light_intensity * albedo.y
}

pub fn render<I>(img: &mut I, spheres: &[Sphere], lights: &[Light])
where
    I: GenericImage<Pixel = Rgb<u8>>,
{
    let width = img.width();
    let height = img.height();
    // NOTE:
    // As we add too much light source, the ratio in v3 in not <= 1 anymore
    // we need to scale them back, the scale ration is 1/maxof(x, y, z)
    // then the largest channel will back to 1.
    let v3_to_rgb = |v: Vector3<f64>| {
        let mut v = v;
        let max_chan = v.x.max(v.y).max(v.z);

        if max_chan > 1. {
            v *= 1. / max_chan;
        }

        let color = [v.x, v.y, v.z]
            .into_iter()
            .map(|n| (255. * n.clamp(0., 1.)) as u8)
            .collect::<Vec<_>>();

        Rgb::from_slice(&color).to_owned()
    };

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

pub fn multi_thread_render(img: &mut RgbImage, spheres: &[Sphere], lights: &[Light]) {
    let width = img.width();
    let height = img.height();
    // NOTE:
    // As we add too much light source, the ratio in v3 in not <= 1 anymore
    // we need to scale them back, the scale ration is 1/maxof(x, y, z)
    // then the largest channel will back to 1.
    let v3_to_rgb = |v: Vector3<f64>| {
        let mut v = v;
        let max_chan = v.x.max(v.y).max(v.z);

        if max_chan > 1. {
            v *= 1. / max_chan;
        }

        let color = [v.x, v.y, v.z]
            .into_iter()
            .map(|n| (255. * n.clamp(0., 1.)) as u8)
            .collect::<Vec<_>>();

        Rgb::from_slice(&color).to_owned()
    };

    let orig = Vector3::new(0., 0., 0.);

    img.par_pixels_mut().enumerate().for_each(|(idx, pixel)| {
        let x = idx as u32 % width;
        let y = idx as u32 / width;
        let (x, y) = pixel_to_world(x, y, width, height, FOV, Z);
        let ray_dir = Vector3::new(x, y, -1.).normalize();
        let color = cast_ray(&orig, &ray_dir, spheres, lights);

        *pixel = v3_to_rgb(color);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    #[test]
    fn test_render_with_multi_thread() {
        let mut img = RgbImage::new(1024, 768);
        let width = img.width();
        let height = img.height();
        img.par_pixels_mut().enumerate().for_each(|(idx, pixel)| {
            let x = idx as u32 % width;
            let y = idx as u32 / width;
            let red = 255 * x / width;
            let green = 255 * y / height;
            *pixel = Rgb([red as u8, green as u8, 0]);
        });
        img.save("output/ray_tracing_step_5_multi_thread.tga")
            .unwrap();
    }

    #[test]
    fn test_render_with_specular_lighting_multi_thread() {
        let ivory = Material {
            diffuse_color: Vector3::new(0.4, 0.4, 0.3),
            albedo: Vector2::new(0.6, 0.3),
            specular_exponent: 50.,
        };
        let red_rubber = Material {
            diffuse_color: Vector3::new(0.3, 0.1, 0.1),
            albedo: Vector2::new(0.9, 0.1),
            specular_exponent: 10.,
        };
        let gold = Material {
            diffuse_color: Vector3::new(0.6, 0.5, 0.3),
            albedo: Vector2::new(0.5, 0.5),
            specular_exponent: 80.,
        };

        let spheres = [
            Sphere::new(Vector3::new(-3., 0., -16.), 2., ivory.clone()),
            Sphere::new(Vector3::new(-1., -1.5, -12.), 2., red_rubber.clone()),
            Sphere::new(Vector3::new(1.5, -0.5, -18.), 3., red_rubber.clone()),
            Sphere::new(Vector3::new(7., 5., -18.), 4., gold.clone()),
        ];

        let lights = [
            Light::new(Vector3::new(-20., 20., 20.), 1.5),
            Light::new(Vector3::new(30., 50., -25.), 1.8),
            Light::new(Vector3::new(30., 20., 30.), 1.7),
        ];

        let mut img = RgbImage::new(1024, 768);
        multi_thread_render(&mut img, &spheres, &lights);
        img.save("output/ray_tracing_step_five_scene.tga").unwrap();
    }

    #[test]
    fn test_render_with_specular_lighting() {
        let ivory = Material {
            diffuse_color: Vector3::new(0.4, 0.4, 0.3),
            albedo: Vector2::new(0.6, 0.3),
            specular_exponent: 50.,
        };
        let red_rubber = Material {
            diffuse_color: Vector3::new(0.3, 0.1, 0.1),
            albedo: Vector2::new(0.9, 0.1),
            specular_exponent: 10.,
        };
        let gold = Material {
            diffuse_color: Vector3::new(0.6, 0.5, 0.3),
            albedo: Vector2::new(0.5, 0.5),
            specular_exponent: 80.,
        };

        let spheres = [
            Sphere::new(Vector3::new(-3., 0., -16.), 2., ivory.clone()),
            Sphere::new(Vector3::new(-1., -1.5, -12.), 2., red_rubber.clone()),
            Sphere::new(Vector3::new(1.5, -0.5, -18.), 3., red_rubber.clone()),
            Sphere::new(Vector3::new(7., 5., -18.), 4., gold.clone()),
        ];

        let lights = [
            Light::new(Vector3::new(-20., 20., 20.), 1.5),
            Light::new(Vector3::new(30., 50., -25.), 1.8),
            Light::new(Vector3::new(30., 20., 30.), 1.7),
        ];

        let mut img = RgbImage::new(1024, 768);
        // PERF: without multithread, this takes 2.51s
        // with multithread, it takes only 0.38s
        render(&mut img, &spheres, &lights);
        img.save("output/ray_tracing_step_five_scene.tga").unwrap();
    }
}
