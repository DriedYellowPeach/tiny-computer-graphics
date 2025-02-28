use image::{Rgb, RgbImage};
use rayon::prelude::*;

use crate::raytracer::world::Scene;

use super::world::background::Background;

pub fn render<B: Background>(img: &mut RgbImage, scene: &Scene<B>) {
    let width = img.width();
    let height = img.height();

    // NOTE: using multi-threading to do ray scanning
    img.par_pixels_mut().enumerate().for_each(|(idx, pixel)| {
        let film_pixel = scene.camera.pixel_on_film(idx, width, height);
        let ray = scene.camera.ray_to_pixel(film_pixel.x, film_pixel.y);
        let color = scene.cast_ray(&ray, 0);

        *pixel = Rgb::from(color);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raytracer::{
        world::{
            background::DummyBackground,
            objects::{box_3d::AABBox, Light, Material, Sphere},
            Scene,
        },
        Albedo, Color, Position,
    };

    fn example_scene() -> Scene<DummyBackground> {
        let ivory = Material {
            diffuse_color: Color::new(0.4, 0.4, 0.3),
            albedo: Albedo::new(0.6, 0.3, 0.1, 0.0),
            specular_exponent: 50.,
            refractive_index: 1.,
        };

        let red_rubber = Material {
            diffuse_color: Color::new(0.3, 0.1, 0.1),
            albedo: Albedo::new(0.9, 0.1, 0.0, 0.0),
            specular_exponent: 10.,
            refractive_index: 1.,
        };

        // mostly refraction
        // no diffuse color at all
        let glass = Material {
            diffuse_color: Color::new(0.6, 0.7, 0.8),
            albedo: Albedo::new(0.0, 0.5, 0.1, 0.8),
            specular_exponent: 125.,
            refractive_index: 1.5,
        };
        let gold = Material {
            diffuse_color: Color::new(0.6, 0.5, 0.3),
            albedo: Albedo::new(0.5, 0.5, 0.1, 0.0),
            specular_exponent: 80.,
            refractive_index: 0.8,
        };
        let magenta = Material {
            diffuse_color: Color::MAGENTA,
            albedo: Albedo::new(0.3, 0.3, 0.1, 0.0),
            specular_exponent: 20.,
            refractive_index: 0.8,
        };
        let mirror = Material {
            diffuse_color: Color::new(0., 0., 0.),
            albedo: Albedo::new(1., 1., 0.87, 0.0),
            specular_exponent: 1425.,
            refractive_index: 1.,
        };

        let l1 = Light::new(Position::new(-20., 20., 20.), 1.5);
        let l2 = Light::new(Position::new(30., 50., -25.), 1.8);
        let l3 = Light::new(Position::new(30., 20., 30.), 1.7);

        let sp1 = Sphere::new(Position::new(-3., 0., -16.), 2., ivory.clone());
        let sp2 = Sphere::new(Position::new(-1., -1.5, -12.), 2., glass.clone());
        let sp3 = Sphere::new(Position::new(1.5, -0.5, -18.), 3., red_rubber.clone());
        let sp4 = Sphere::new(Position::new(7., 5., -18.), 4., mirror.clone());
        let sp5 = Sphere::new(Position::new(-3., 2.5, -8.), 2., gold.clone());
        let box1 = AABBox::try_build(
            Position::new(4., -4., -15.),
            Position::new(6., -2., -12.),
            magenta.clone(),
        )
        .unwrap();

        Scene::default()
            .add_background(DummyBackground)
            .add_object(sp1)
            .add_object(sp2)
            .add_object(sp3)
            .add_object(sp4)
            .add_object(sp5)
            .add_object(box1)
            .add_light(l1)
            .add_light(l2)
            .add_light(l3)
    }

    #[test]
    fn test_render() {
        let mut img = RgbImage::new(800, 600);
        let scene = example_scene();

        render(&mut img, &scene);
        img.save("output/customized_ray_tracer.png").unwrap();
    }
}
