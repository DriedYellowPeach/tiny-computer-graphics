use super::{
    background::Background,
    objects::{Light, Visible},
    HitPoint, Ray,
};
use crate::raytracer::{Color, Direction, Interval};

const RECURSION_DEPTH: usize = 5;

pub struct Scene<B> {
    pub lights: Vec<Light>,
    objects: Vec<Box<dyn Visible>>,
    background: Option<B>,
    view_range: f64,
}

impl<B> Default for Scene<B> {
    fn default() -> Self {
        Self {
            lights: Vec::new(),
            objects: Vec::new(),
            background: None,
            view_range: 1000.,
        }
    }
}

impl<B> Scene<B>
where
    B: Background,
{
    pub fn add_light(mut self, light: Light) -> Self {
        self.lights.push(light);
        self
    }

    pub fn add_object<V: Visible + 'static>(mut self, object: V) -> Self {
        self.objects.push(Box::new(object));
        self
    }

    pub fn add_background(mut self, background: B) -> Self {
        self.background = Some(background);
        self
    }

    pub fn update_view_range(mut self, view_range: f64) -> Self {
        self.view_range = view_range;
        self
    }

    #[allow(non_snake_case)]
    fn direct_illumination(&self, ray: &Ray, hit_point: &HitPoint) -> (f64, f64) {
        let mut diffuse_light_intensity = 0.;
        let mut specular_light_intensity = 0.;
        // BUG: should be surface_norm or norm_of???
        let N = hit_point.norm();

        for light in &self.lights {
            let to_light = Direction::a_to_b(&hit_point.position, &light.position);
            let hit_point_to_light_dist = light.position.distance_to(&hit_point.position);

            if !to_light.is_acute_angle(&N) {
                continue;
            }

            let shadow_ray = Ray::shadowed(hit_point, &light.position);

            if self.intersect(&shadow_ray).is_some_and(|shadow_hit_point| {
                shadow_hit_point.position.distance_to(&shadow_ray.position)
                    < hit_point_to_light_dist
            }) {
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

    /// Cast a ray on scene
    /// After calculation, return the integrated color of whatever the ray hit
    #[allow(non_snake_case)]
    pub fn cast_ray(&self, ray: &Ray, depth: usize) -> Color {
        // WARN: Background color or Pure black?
        if depth > RECURSION_DEPTH {
            return Color::BLACK;
        }

        // NOTE: Not hit any object in scene, return background color
        let Some(hit_info) = self.intersect(ray) else {
            return self.intersect_background(ray);
        };

        // NOTE: Calculate Reflection and Refraction: Indirect Illumination
        let reflective_color = if hit_info.surface_material().albedo.reflective() > 0. {
            let reflect_ray = ray.reflected(&hit_info);
            self.cast_ray(&reflect_ray, depth + 1)
        } else {
            self.intersect_background(ray)
        };

        let refractive_color = if hit_info.surface_material().albedo.refractive() > 0. {
            let refract_ray = ray.refracted(&hit_info);
            self.cast_ray(&refract_ray, depth + 1)
        } else {
            self.intersect_background(ray)
        };

        // NOTE: Calculate Diffusive and Specular Light: Direct Illumination
        let (diffuse_light_intensity, specular_light_intensity) =
            self.direct_illumination(ray, &hit_info);

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

    /// Check if anything in Scene hit by ray
    pub fn intersect(&self, ray: &Ray) -> Option<HitPoint> {
        // don't use Option, cause at least one thing will be hit, that is background
        // background should fill the whole scene
        let mut min_hit_dist = f64::MAX;
        let mut ret = None;
        let interval = Interval::new(0., self.view_range);

        for obj in self.objects.iter() {
            if let Some(t) = obj.hit_by_ray(ray, &interval) {
                if t >= min_hit_dist {
                    continue;
                }

                min_hit_dist = t;
                let hit_point = ray.at(t);
                let is_outside = ray.dir.dot(&obj.surface_norm(&hit_point)) < 0.;

                ret = Some(HitPoint::new(obj.as_ref(), hit_point, is_outside));
            }
        }

        if min_hit_dist > self.view_range {
            return None;
        }

        ret
    }

    pub fn intersect_background(&self, ray: &Ray) -> Color {
        self.background
            .as_ref()
            .map_or(Color::BLACK, |bg| bg.get_color(ray))
    }
}
