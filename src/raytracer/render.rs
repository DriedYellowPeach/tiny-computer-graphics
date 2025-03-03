use image::{Rgb, RgbImage};
use indicatif::{ParallelProgressIterator, ProgressState, ProgressStyle};
use rayon::iter::ParallelIterator;
use rayon::prelude::*;

use std::fmt::Write;

use super::world::background::Background;
use crate::raytracer::world::Scene;

fn progress_bar_style() -> ProgressStyle {
    let text = [
        "[{bar:48.cyan/blue}] {percent}% {spinner:.green}",
        "Elapsed Time     : {elapsed_precise}",
        "ETA              : {eta}",
        "Tracing Progress : {pos}/{len} rays",
        "Tracing Speed    : {per_sec}",
    ]
    .join("\n");

    ProgressStyle::with_template(&text)
        .unwrap()
        .with_key("per_sec", |state: &ProgressState, w: &mut dyn Write| {
            _ = write!(w, "{:.0} rays/sec", state.per_sec());
        })
        .progress_chars("#>-")
}

pub fn render<B: Background>(img: &mut RgbImage, scene: &Scene<B>) {
    let width = img.width();
    let height = img.height();

    img.par_pixels_mut()
        .progress_with_style(progress_bar_style())
        .enumerate()
        .for_each(|(idx, pixel)| {
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
            objects::{box_3d::AABBox, GradientSphere, Light, Material, Sphere},
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

        let mirror2 = Material {
            diffuse_color: Color::new(40. / 255., 40. / 255., 40. / 255.),
            albedo: Albedo::new(1., 0.1, 0.1, 0.0),
            specular_exponent: 30.,
            refractive_index: 1.,
        };

        let l1 = Light::new(Position::new(-20., 20., 20.), 1.5);
        let l2 = Light::new(Position::new(30., 50., -25.), 1.8);
        let l3 = Light::new(Position::new(30., 20., 30.), 1.7);

        let sp1 = Sphere::new(Position::new(-3., 0., -16.), 2., ivory.clone());
        let sp2 = Sphere::new(Position::new(-1., -1.5, -12.), 2., glass.clone());
        let sp3 = Sphere::new(Position::new(1.5, -0.5, -18.), 3., red_rubber.clone());
        let sp4 = Sphere::new(Position::new(5., 8., -18.), 4., mirror.clone());
        let sp5 = Sphere::new(Position::new(-3., 2.5, -8.), 2., gold.clone());
        let gradient_sp = GradientSphere::new(Position::new(7., 0.5, -10.), 2.);
        let box1 = AABBox::try_build(
            Position::new(4.5, -3.5, -18.),
            Position::new(10., -1.5, -8.),
            magenta.clone(),
        )
        .unwrap();

        let floor = AABBox::try_build(
            Position::new(-100., -20., -100.),
            Position::new(100., -3.5, 100.),
            mirror2.clone(),
        )
        .unwrap();

        Scene::default()
            .add_background(DummyBackground)
            .add_object(sp1)
            .add_object(sp2)
            .add_object(sp3)
            .add_object(sp4)
            .add_object(sp5)
            .add_object(gradient_sp)
            .add_object(floor)
            .add_object(box1)
            .add_light(l1)
            .add_light(l2)
            .add_light(l3)
    }

    #[test]
    fn test_render() {
        let mut img = RgbImage::new(800, 450);
        let scene = example_scene();

        render(&mut img, &scene);
        img.save("output/customized_ray_tracer_4.png").unwrap();
    }

    #[test]
    fn test_rust_zip_end_on_shortest_seq() {
        let short = [1, 3, 5];

        assert_eq!((0..).zip(&short).count(), 3);
    }
}
