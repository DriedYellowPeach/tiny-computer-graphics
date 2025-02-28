/// step 6 is to cast shadow on bodies
/// cause we are using directional light, our backward ray tracer is not real enough
/// because when tracing back to light source it may intersect with another body
///
///
///
use image::{Pixel, Rgb, RgbImage};
use nalgebra::Vector3;
use rayon::prelude::*;

use super::step_04_lighting::Light;
use super::step_05_specular_lighting::{reflection, scene_intersect, Sphere};
use super::{pixel_to_world, BACKGROUND_COLOR, FOV, Z};

#[allow(non_snake_case)]
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
        let N = (hit_point - sphere.center).normalize();
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
    use super::super::step_05_specular_lighting::Material;
    use super::*;
    use nalgebra::Vector2;

    #[test]
    fn test_render_with_shadow() {
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
        img.save("output/ray_tracing_step_6_scene.tga").unwrap();
    }
}
