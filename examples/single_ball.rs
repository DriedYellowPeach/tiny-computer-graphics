use image::RgbImage;

use std::env;
use std::path::Path;

use tiny_computer_graphics::raytracer::prelude::*;

fn example_scene() -> Scene<DummyBackground> {
    let red_rubber = Material {
        diffuse_color: Color::new(0.3, 0.1, 0.1),
        albedo: Albedo::new(0.9, 0.1, 0.0, 0.0),
        specular_exponent: 10.,
        refractive_index: 1.,
    };

    let l1 = Light::new(Position::new(-20., 20., 20.), 1.5);
    let l2 = Light::new(Position::new(30., 50., -25.), 1.8);
    let l3 = Light::new(Position::new(30., 20., 30.), 1.7);

    let sp1 = Sphere::new(Position::new(-3., 0., -16.), 8., red_rubber.clone());
    Scene::default()
        .add_background(DummyBackground)
        .add_object(sp1)
        .add_light(l1)
        .add_light(l2)
        .add_light(l3)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut antialias = false; // Default value
                               // Check for --antialias flag
    for arg in args.iter() {
        if arg == "--antialias" {
            antialias = true;
        }
    }

    let mut img = RgbImage::new(800, 450);
    let scene = example_scene();
    let camera = CameraBuilder::default().antialiasing(antialias).build();

    camera.render(&scene, &mut img);

    let file_path = file!();
    let file_stem = Path::new(file_path).file_stem().unwrap().to_str().unwrap();
    let suffix = if antialias { "_antialias" } else { "" };

    img.save(format!("output/example_{file_stem}{suffix}.png"))
        .unwrap();
}
