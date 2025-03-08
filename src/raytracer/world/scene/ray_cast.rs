use crate::raytracer::world::{background::Background, HitPoint, Ray};
use rand::Rng;

use crate::raytracer::{Color, Direction};

use super::SceneData;

const RECURSION_DEPTH: usize = 5;

pub trait RayCastStrategy: Send + Sync {
    fn cast_ray<B: Background>(&self, scene: &SceneData<B>, ray: &Ray, depth: usize) -> Color;
}

pub struct Lambertian;

impl Lambertian {
    #[allow(non_snake_case)]
    fn direct_illumination<B: Background>(
        &self,
        scene_data: &SceneData<B>,
        ray: &Ray,
        hit_point: &HitPoint,
    ) -> (f64, f64) {
        let mut diffuse_light_intensity = 0.;
        let mut specular_light_intensity = 0.;
        // BUG: should be surface_norm or norm_of???
        let N = hit_point.norm();

        for light in &scene_data.lights {
            let to_light = Direction::a_to_b(&hit_point.position, &light.position);
            let hit_point_to_light_dist = light.position.distance_to(&hit_point.position);

            if !to_light.is_acute_angle(&N) {
                continue;
            }

            let shadow_ray = Ray::shadowed(hit_point, &light.position);

            if scene_data
                .intersect(&shadow_ray)
                .is_some_and(|shadow_hit_point| {
                    shadow_hit_point.position.distance_to(&shadow_ray.position)
                        < hit_point_to_light_dist
                })
            {
                continue;
            }

            let reverse_reflect_light_dir = to_light.reverse().reflection(&N).reverse();
            let to_expo = ray
                .dir
                .dot(&reverse_reflect_light_dir)
                .max(0.)
                .powf(hit_point.surface_material().specular_exponent);

            diffuse_light_intensity += light.intensity * to_light.dot(&N).max(0.);
            specular_light_intensity += light.intensity * to_expo;
        }

        (diffuse_light_intensity, specular_light_intensity)
    }
}

impl RayCastStrategy for Lambertian {
    fn cast_ray<B: Background>(&self, scene: &SceneData<B>, ray: &Ray, depth: usize) -> Color {
        // WARN: Background color or Pure black?
        if depth > RECURSION_DEPTH {
            return Color::BLACK;
        }

        // NOTE: Not hit any object in scene, return background color
        let Some(hit_info) = scene.intersect(ray) else {
            return scene.intersect_background(ray);
        };

        // NOTE: Calculate Reflection and Refraction: Indirect Illumination
        let reflective_color = if hit_info.surface_material().albedo.reflective() > 0. {
            let reflect_ray = ray.reflected(&hit_info);
            self.cast_ray(scene, &reflect_ray, depth + 1)
        } else {
            scene.intersect_background(ray)
        };

        let refractive_color = if hit_info.surface_material().albedo.refractive() > 0. {
            let refract_ray = ray.refracted(&hit_info);
            self.cast_ray(scene, &refract_ray, depth + 1)
        } else {
            scene.intersect_background(ray)
        };

        // NOTE: Calculate Diffusive and Specular Light: Direct Illumination
        let (diffuse_light_intensity, specular_light_intensity) =
            self.direct_illumination(scene, ray, &hit_info);

        let albedo = &hit_info.surface_material().albedo;
        let diffuse_color = hit_info
            .surface_material()
            .diffuse_color
            .apply_intensity(diffuse_light_intensity);
        let specular_color = Color::WHITE.apply_intensity(specular_light_intensity);

        Color::apply_albedo(
            diffuse_color,
            specular_color,
            reflective_color,
            refractive_color,
            albedo,
        )
    }
}

pub struct MonteCarlo {
    recursion_depth: usize,
}

impl Default for MonteCarlo {
    fn default() -> Self {
        Self {
            recursion_depth: 10,
        }
    }
}

impl MonteCarlo {
    pub fn new(recursion_depth: usize) -> Self {
        Self { recursion_depth }
    }

    fn diffusive_ray_on_hemisphere(&self, hit: &HitPoint) -> Ray {
        let mut rng = rand::rng();
        let mut dir = Direction::new(
            rng.random_range(-1f64..1.),
            rng.random_range(-1f64..1.),
            rng.random_range(-1f64..1.),
        );

        if !dir.is_acute_angle(&hit.norm()) {
            dir = dir.reverse();
        }

        Ray::new(hit.position, dir)
    }
}

impl RayCastStrategy for MonteCarlo {
    fn cast_ray<B: Background>(&self, scene: &SceneData<B>, ray: &Ray, depth: usize) -> Color {
        if depth > self.recursion_depth {
            return Color::BLACK;
        }

        // NOTE: Not hit any object in scene, return background color
        let Some(hit_p) = scene.intersect(ray) else {
            return scene.intersect_background(ray);
        };

        let diffusive_ray = self.diffusive_ray_on_hemisphere(&hit_p);
        0.5 * self.cast_ray(scene, &diffusive_ray, depth + 1)
    }
}
