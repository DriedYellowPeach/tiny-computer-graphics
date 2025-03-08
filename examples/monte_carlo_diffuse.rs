use image::RgbImage;

use std::path::Path;

use tiny_computer_graphics::raytracer::{prelude::*, world::scene::MonteCarlo};

fn example_scene() -> Scene<Sky, MonteCarlo> {
    let floor = Material {
        diffuse_color: Color::new(0.5, 0.5, 0.5),
        albedo: Albedo::new(1.0, 0.0, 0.0, 0.0),
        specular_exponent: 10.,
        refractive_index: 1.,
    };

    let red_rubber = Material {
        diffuse_color: Color::new(0.3, 0.1, 0.1),
        albedo: Albedo::new(0.9, 0.1, 0.0, 0.0),
        specular_exponent: 10.,
        refractive_index: 1.,
    };

    let sp1 = Sphere::new(Position::new(0., 2., -5.), 2., red_rubber.clone());
    let floor = Sphere::new(Position::new(0., -1000., 0.), 1000., floor.clone());

    Scene::default()
        .add_background(Sky)
        .add_object(sp1)
        .add_object(floor)
}

fn main() {
    let mut img = RgbImage::new(800, 450);
    let scene = example_scene();
    let camera = CameraBuilder::default()
        .antialiasing(true)
        .position(Position::new(0., 0.8, 0.))
        .build();

    camera.render(&scene, &mut img);

    let file_path = file!();
    let file_stem = Path::new(file_path).file_stem().unwrap().to_str().unwrap();

    img.save(format!("output/example_{file_stem}.png")).unwrap();
}
