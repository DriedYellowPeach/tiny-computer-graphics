use image::{Pixel, Rgb, RgbImage};
use nalgebra::{Vector3, Vector4};
use rayon::prelude::*;

use super::{
    pixel_to_world, step_04_lighting::Light, step_05_specular_lighting::reflection,
    BACKGROUND_COLOR, FOV, REFLECT_DEPTH, Z,
};

#[derive(Clone)]
pub struct Material {
    pub diffuse_color: Vector3<f64>,
    // NOTE: albedo is reflection ratio
    // albedo.x: on diffuse light
    // albedo.y: on specular light
    // albedo.z: on reflection light
    pub albedo: Vector4<f64>,
    pub specular_exponent: f64,
    pub refractive_index: f64,
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

// Haha,  it doesn't work like I supposed! Maybe works in 2D
// but for 3D, it's so hard to figure out
// fn refract_v1(I: &Vector3<f64>, N: &Vector3<f64>, refractive_index: f64) -> Vector3<f64> {
//     let n1 = 1.;
//     let n2 = refractive_index;
//     let cos_theta1 = -I.dot(N).clamp(-1., 1.);
//     let sin_theta1 = (1. - cos_theta1.powi(2)).sqrt().clamp(-1., 1.);
//     let sin_theta2 = (n1 * sin_theta1 / n2).clamp(-1., 1.);
//     // NOTE: theta3 = theta1 - theta2
//     let theta1 = cos_theta1.acos();
//     let theta2 = sin_theta2.asin();
//     let theta3 = theta1 - theta2;
//
//     let rotate_mat = matrix![
//         theta3.cos(), theta3.sin();
//         -theta3.sin(), theta3.cos();
//     ];
//
//     rotate_mat * I
// }

// NOTE: refract takes unit ray direction vector and normal vector of the surface
// with reflactive_index inside the object
// returns the refract ray
#[allow(non_snake_case)]
fn refraction(I: &Vector3<f64>, N: &Vector3<f64>, refractive_index: f64) -> Vector3<f64> {
    let mut n1 = 1.;
    let mut n2 = refractive_index;
    let mut N = *N;

    let mut cos_theta1 = -I.dot(&N).clamp(-1., 1.);

    // NOTE: this means inside the object
    // we swap the setting and then calculate
    if cos_theta1 < 0. {
        cos_theta1 = -cos_theta1;
        std::mem::swap(&mut n1, &mut n2);
        N = -N;
    }

    let sin_theta1 = (1. - cos_theta1.powi(2)).sqrt().clamp(-1., 1.);
    let sin_theta2 = (n1 / n2 * sin_theta1).clamp(-1., 1.);
    let cos_theta2 = (1. - sin_theta2.powi(2)).sqrt().clamp(-1., 1.);

    // NOTE: snell's law: vector form
    // L' = (n1/n2) * L + ((n1/n2)cos(theta1) - cos(theta2)) * N
    (n1 / n2) * I + ((n1 / n2) * cos_theta1 - cos_theta2) * N
}

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

#[allow(non_snake_case)]
pub fn cast_ray(
    orig: &Vector3<f64>,
    ray_dir: &Vector3<f64>,
    spheres: &[Sphere],
    lights: &[Light],
    depth: usize,
) -> Vector3<f64> {
    if depth > REFLECT_DEPTH {
        return BACKGROUND_COLOR;
        // return Vector3::from_element(0.);
    }

    let Some((sphere, hit_point)) = scene_intersect(orig, ray_dir, spheres) else {
        return BACKGROUND_COLOR;
    };

    let N = (hit_point - sphere.center).normalize();

    // NOTE: Calculate Reflection and Refraction
    // both of them require
    // now we intersect with object, to cast the reflection ray
    let reflect_dir = reflection(ray_dir, &N).normalize();
    let refract_dir = refraction(ray_dir, &N, sphere.mat.refractive_index).normalize();

    let reflect_orig = if reflect_dir.dot(&N) > 0. {
        hit_point + N * 1e-3
    } else {
        hit_point - N * 1e-3
    };

    let refract_orig = if refract_dir.dot(&N) > 0. {
        hit_point + N * 1e-3
    } else {
        hit_point - N * 1e-3
    };

    let reflect_color = if sphere.mat.albedo.z > 0. {
        cast_ray(&reflect_orig, &reflect_dir, spheres, lights, depth + 1)
    } else {
        Vector3::from_element(0.)
    };
    let refract_color = if sphere.mat.albedo.w > 0. {
        cast_ray(&refract_orig, &refract_dir, spheres, lights, depth + 1)
    } else {
        Vector3::from_element(0.)
    };

    let mut diffuse_light_intensity = 0.;
    let mut specular_light_intensity = 0.;
    for light in lights {
        // NOTE: this light_dir start at hit point
        let light_dir = (light.position - hit_point).normalize();
        // NOTE: apply shadow here by checking if anything between hit point to light source
        // try solution one
        let hit_point_to_light = (light.position - hit_point).magnitude();
        if light_dir.dot(&N) < 0. {
            continue;
        }
        // NOTE: move shadow_orig away from sphere, cause we don't want to intersect with ourselves
        let shadow_orig = hit_point + N * 1e-3;
        // if let Some((_sphere, hit_point)) = scene_intersect(shadow_orig, ray_dir, spheres)
        if let Some((_sphere, shadow_hit_point)) =
            scene_intersect(&shadow_orig, &light_dir, spheres)
        {
            // NOTE: if shadow_hit_point is closer to light source than hit point, then it's in shadow
            if (shadow_hit_point - shadow_orig).magnitude() < hit_point_to_light {
                continue;
            }
        }
        // NOTE:
        // -light_dir point at hit point
        // reflaction start from hit point
        // reverse toward hit point, so as ray_dir
        let reverse_reflect_light_dir = -reflection(&(-light_dir), &N);
        // NOTE:
        // The larger specular_exponent, the smaller the specular light intensity
        // makes the specular lighting more concentrated
        let to_expo = ray_dir
            .dot(&reverse_reflect_light_dir)
            .max(0.)
            .powf(sphere.mat.specular_exponent);
        diffuse_light_intensity += light.intensity * light_dir.dot(&N).max(0.);
        specular_light_intensity += light.intensity * to_expo;
    }

    let albedo = sphere.mat.albedo;
    let white = Vector3::new(1., 1., 1.);

    sphere.mat.diffuse_color * diffuse_light_intensity * albedo.x
        + white * specular_light_intensity * albedo.y
        + reflect_color * albedo.z
        + refract_color * albedo.w
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
        let color = cast_ray(&orig, &ray_dir, spheres, lights, 0);

        *pixel = v3_to_rgb(color);
    });
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;

    #[test]
    fn test_snells_law() {
        let n = Vector3::new(0., 1., 0.);

        // TEST: no refraction
        let theta1 = 0.0f64;
        let i = Vector3::new(theta1.sin(), -theta1.cos(), 0.);
        let i_prime = refraction(&i, &n, 0.5);
        assert_eq!(i_prime, Vector3::new(0., -1., 0.));

        // TEST: 45 degree
        let theta1 = PI / 4.;
        let i = Vector3::new(theta1.sin(), -theta1.cos(), 0.);
        dbg!(i);
        let n2 = 1. / 0.9;
        let i_prime = refraction(&i, &n, n2);
        assert!(
            (i_prime - Vector3::new(0.636396, -0.771362, 0.)).abs() < Vector3::from_element(1e-6)
        );
    }

    #[test]
    fn test_render_with_refraction() {
        // some reflection, no refraction
        let ivory = Material {
            diffuse_color: Vector3::new(0.4, 0.4, 0.3),
            albedo: Vector4::new(0.6, 0.3, 0.1, 0.0),
            specular_exponent: 50.,
            refractive_index: 1.,
        };

        let red_rubber = Material {
            diffuse_color: Vector3::new(0.3, 0.1, 0.1),
            albedo: Vector4::new(0.9, 0.1, 0.0, 0.0),
            specular_exponent: 10.,
            refractive_index: 1.,
        };

        // mostly refraction
        // no diffuse color at all
        let glass = Material {
            diffuse_color: Vector3::new(0.6, 0.7, 0.8),
            albedo: Vector4::new(0.0, 0.5, 0.1, 0.8),
            specular_exponent: 125.,
            refractive_index: 1.5,
        };
        let gold = Material {
            diffuse_color: Vector3::new(0.6, 0.5, 0.3),
            albedo: Vector4::new(0.5, 0.5, 0.1, 0.0),
            specular_exponent: 80.,
            refractive_index: 0.8,
        };
        let mirror = Material {
            diffuse_color: Vector3::new(0., 0., 0.),
            albedo: Vector4::new(1., 1., 0.87, 0.0),
            specular_exponent: 1425.,
            refractive_index: 1.,
        };

        let spheres = [
            Sphere::new(Vector3::new(-3., 0., -16.), 2., ivory.clone()),
            Sphere::new(Vector3::new(-1., -1.5, -12.), 2., glass.clone()),
            Sphere::new(Vector3::new(1.5, -0.5, -18.), 3., red_rubber.clone()),
            Sphere::new(Vector3::new(7., 5., -18.), 4., mirror.clone()),
            Sphere::new(Vector3::new(-3., 2.5, -8.), 2., gold.clone()),
        ];

        let lights = [
            Light::new(Vector3::new(-20., 20., 20.), 1.5),
            Light::new(Vector3::new(30., 50., -25.), 1.8),
            Light::new(Vector3::new(30., 20., 30.), 1.7),
        ];

        let mut img = RgbImage::new(2048, 1536);
        multi_thread_render(&mut img, &spheres, &lights);
        img.save("output/ray_tracing_step_8_scene.tga").unwrap();
    }
}
