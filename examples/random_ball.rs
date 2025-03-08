use image::RgbImage;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::path::Path;

use tiny_computer_graphics::raytracer::prelude::*;

const SMALL_RADIUS: f64 = 0.2;
const BIG_RADIUS: f64 = 1.5;

fn random_ball_around(rng: &mut StdRng, x: i32, z: i32, big_balls: &[Position]) -> Option<Sphere> {
    let x_min = x as f64;
    let x_max = (x + 1) as f64;
    let y_min = z as f64;
    let y_max = (z + 1) as f64;

    let r = rng.random_range(0.0..1.0);
    let g = rng.random_range(0.0..1.0);
    let b = rng.random_range(0.0..1.0);

    let rubber = Material {
        diffuse_color: Color::new(r, g, b),
        albedo: Albedo::new(0.9, 0.1, 0.0, 0.0),
        specular_exponent: 10.,
        refractive_index: 1.,
    };

    let glass = Material {
        diffuse_color: Color::new(0.0, 0.0, 0.0),
        albedo: Albedo::new(0.0, 0.5, 0.1, 0.8),
        specular_exponent: 125.,
        refractive_index: 1.5,
    };

    let pos = Position::new(
        rng.random_range(x_min + SMALL_RADIUS..x_max - SMALL_RADIUS),
        0.2,
        rng.random_range(y_min + SMALL_RADIUS..y_max - SMALL_RADIUS),
    );

    for big_ball in big_balls {
        if pos.distance_to(big_ball) < BIG_RADIUS + SMALL_RADIUS {
            return None;
        }
    }

    let mat = match rng.random_range(0..10) {
        0..8 => rubber,
        8..10 => glass,
        _ => unreachable!(),
    };

    Some(Sphere::new(pos, SMALL_RADIUS, mat))
}

fn random_scene() -> Scene<DummyBackground> {
    let seed = [42u8; 32]; // 32-byte seed for StdRng
    let mut rng = StdRng::from_seed(seed);

    let mirror2 = Material {
        diffuse_color: Color::new(40. / 255., 40. / 255., 40. / 255.),
        albedo: Albedo::new(1., 0.1, 0.1, 0.0),
        specular_exponent: 30.,
        refractive_index: 1.,
    };

    let floor = AABBox::try_build(
        Position::new(-100., -20., -100.),
        Position::new(100., 0., 100.),
        mirror2.clone(),
    )
    .unwrap();

    let mirror = Material {
        diffuse_color: Color::new(0., 0., 0.),
        albedo: Albedo::new(1., 1., 0.87, 0.0),
        specular_exponent: 1425.,
        refractive_index: 1.,
    };

    let gold = Material {
        diffuse_color: Color::new(0.6, 0.5, 0.3),
        albedo: Albedo::new(0.8, 0.2, 0.0, 0.0),
        specular_exponent: 80.,
        refractive_index: 0.8,
    };

    let glass = Material {
        diffuse_color: Color::new(0.6, 0.7, 0.8),
        albedo: Albedo::new(0.0, 0.2, 0.0, 0.8),
        specular_exponent: 125.,
        refractive_index: 5.0,
    };

    let mut big_ball_pos = vec![Position::new(3., BIG_RADIUS, -4.)];
    let next_pos =
        big_ball_pos[0].move_forward(BIG_RADIUS * 2. + 0.05, &Direction::new(-1., 0., 0.));
    big_ball_pos.push(next_pos);
    let next_pos =
        big_ball_pos[1].move_forward(BIG_RADIUS * 2. + 0.05, &Direction::new(-1., 0., 0.));
    big_ball_pos.push(next_pos);

    let sp_mirror = Sphere::new(big_ball_pos[0], BIG_RADIUS, glass.clone());
    let sp_glass = Sphere::new(big_ball_pos[1], BIG_RADIUS, mirror.clone());
    let sp_gold = Sphere::new(big_ball_pos[2], BIG_RADIUS, gold.clone());

    let l1 = Light::new(Position::new(-20., 20., 20.), 1.5);
    let l2 = Light::new(Position::new(30., 50., -25.), 1.8);
    let l3 = Light::new(Position::new(30., 20., 30.), 1.7);

    let mut scene = Scene::default()
        .add_background(DummyBackground)
        .add_object(floor)
        .add_object(sp_mirror)
        .add_object(sp_glass)
        .add_object(sp_gold)
        .add_light(l1)
        .add_light(l2)
        .add_light(l3);

    for i in -8..8 {
        for j in -10..1 {
            if let Some(ball) = random_ball_around(&mut rng, i, j, &big_ball_pos) {
                scene = scene.add_object(ball);
            }
        }
    }

    scene
}

fn main() {
    // 16:9
    let mut img = RgbImage::new(1600, 900);
    // let mut img = RgbImage::new(160, 90);
    let scene = random_scene();
    let camera = CameraBuilder::new()
        .position(Position::new(-1., 1.5, 3.))
        .adjust_fov_in_degree(60.)
        .forward_to(Direction::new(0., -1., -3.))
        .build();

    camera.render(&scene, &mut img);

    let file_path = file!();
    let file_stem = Path::new(file_path).file_stem().unwrap().to_str().unwrap();

    img.save(format!("output/example_{file_stem}.png")).unwrap();
}
