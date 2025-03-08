use super::{
    background::{Background, DummyBackground},
    objects::{Light, Visible},
    HitPoint, Ray,
};
use crate::raytracer::{Color, Interval};

mod ray_cast;

pub use ray_cast::{Lambertian, MonteCarlo, RayCastStrategy};

pub struct SceneData<B = DummyBackground> {
    lights: Vec<Light>,
    objects: Vec<Box<dyn Visible>>,
    background: Option<B>,
    view_range: f64,
}

impl<B: Background> SceneData<B> {
    /// Check if anything in Scene hit by ray
    pub fn intersect(&self, ray: &Ray) -> Option<HitPoint> {
        // don't use Option, cause at least one thing will be hit, that is background
        // background should fill the whole scene
        let mut min_hit_dist = f64::MAX;
        let mut ret = None;
        // TODO: set interval start so there is no need to move ray origin
        let interval = Interval::new(1e-3, self.view_range);

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

pub struct Scene<B = DummyBackground, S = Lambertian> {
    scene_data: SceneData<B>,
    ray_caster: S,
}

impl<B> Default for Scene<B, Lambertian> {
    fn default() -> Self {
        self::Scene {
            scene_data: SceneData {
                lights: Vec::new(),
                objects: Vec::new(),
                background: None,
                view_range: 1000.,
            },
            ray_caster: Lambertian,
        }
    }
}

impl<B> Default for Scene<B, MonteCarlo> {
    fn default() -> Self {
        self::Scene {
            scene_data: SceneData {
                lights: Vec::new(),
                objects: Vec::new(),
                background: None,
                view_range: 1000.,
            },
            ray_caster: MonteCarlo::default(),
        }
    }
}

impl<B, S> Scene<B, S>
where
    B: Background,
    S: RayCastStrategy,
{
    pub fn add_light(mut self, light: Light) -> Self {
        self.scene_data.lights.push(light);
        self
    }

    pub fn add_object<V: Visible + 'static>(mut self, object: V) -> Self {
        self.scene_data.objects.push(Box::new(object));
        self
    }

    pub fn add_background(mut self, background: B) -> Self {
        self.scene_data.background = Some(background);
        self
    }

    pub fn update_view_range(mut self, view_range: f64) -> Self {
        self.scene_data.view_range = view_range;
        self
    }

    pub fn cast_ray(&self, ray: &Ray) -> Color {
        self.ray_caster.cast_ray(&self.scene_data, ray, 0)
    }
}
