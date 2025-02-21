use anyhow::{bail, Result};
use image::GenericImage;

use std::{fs::File, io::BufRead, path::Path};

pub fn naive_draw_line<I: GenericImage>(
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    img: &mut I,
    pixel: I::Pixel,
) {
    let mut t = 0.0f64;

    let x0 = x0 as f64;
    let y0 = y0 as f64;
    let x1 = x1 as f64;
    let y1 = y1 as f64;

    while t < 1.0 {
        t += 0.01;
        let x_prime = (x0 + t * (x1 - x0)).round();
        let y_prime = (y0 + t * (y1 - y0)).round();
        img.put_pixel(x_prime as u32, y_prime as u32, pixel);
    }
}

pub fn draw_line<I: GenericImage>(
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    img: &mut I,
    pixel: I::Pixel,
) {
    let mut steep = false;
    let (mut x0, mut y0, mut x1, mut y1) = (x0 as i64, y0 as i64, x1 as i64, y1 as i64);

    if (x1 - x0).abs() < (y1 - y0).abs() {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
        steep = true;
    }

    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }

    let dx = x1 - x0;
    let dy = y1 - y0;
    let derr2 = dy.abs() * 2;
    let mut err2 = 0;
    let mut y = y0;

    for x in x0..=x1 {
        if steep {
            img.put_pixel(y as u32, x as u32, pixel);
        } else {
            img.put_pixel(x as u32, y as u32, pixel);
        }
        err2 += derr2;

        if err2 > dx {
            y += if y1 > y0 { 1 } else { -1 };
            err2 -= dx * 2;
        }
    }
}

pub struct Model {
    pub vertices: Vec<(f64, f64, f64)>,
    // faces stores index, (a, b, c) means ath, bth, and cth vertices form one fase
    pub faces: Vec<(usize, usize, usize)>,
}

impl Model {
    fn parse_vertex(text: &str) -> Result<(f64, f64, f64)> {
        let parts = text
            .split_whitespace()
            .filter_map(|num| num.parse::<f64>().ok())
            .collect::<Vec<_>>();

        if parts.len() != 3 {
            bail!("Failed to parse vertext line: {text}");
        }

        Ok((parts[0], parts[1], parts[2]))
    }

    fn parse_face(text: &str) -> Result<(usize, usize, usize)> {
        let parts = text
            .split_whitespace()
            .filter_map(|nums| nums.split('/').next())
            .filter_map(|num| num.parse::<usize>().ok()) // NOTE: in .obj file, the index starts from 1
            .map(|n| n - 1)
            .collect::<Vec<_>>();

        if parts.len() != 3 {
            bail!("Failed to parse face line: {text}");
        }

        Ok((parts[0], parts[1], parts[2]))
    }

    pub fn load_model<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut m = Model {
            vertices: vec![],
            faces: vec![],
        };

        let file = File::open(path)?;
        let reader = std::io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?; // Handle Result<String>
            if line.starts_with("v ") {
                m.vertices.push(Self::parse_vertex(&line)?);
                continue;
            }

            if line.starts_with("f ") {
                m.faces.push(Self::parse_face(&line)?);
                continue;
            }
        }

        Ok(m)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use image::{imageops, Rgb, RgbImage};

    #[test]
    fn test_draw_some_lines() {
        let mut img = RgbImage::new(128, 128);
        naive_draw_line(0, 0, 10, 10, &mut img, Rgb([255, 255, 255]));
        naive_draw_line(30, 0, 40, 80, &mut img, Rgb([255, 0, 0]));
        naive_draw_line(100, 40, 30, 10, &mut img, Rgb([0, 255, 0]));
        imageops::flip_vertical_in_place(&mut img);
        img.save("output/naive_line_drawing_algorithm.tga").unwrap();
    }

    #[test]
    fn test_draw_a_triangle() {
        let mut img = RgbImage::new(128, 128);
        let (x1, y1) = (10, 10);
        let (x2, y2) = (70, 50);
        let (x3, y3) = (40, 80);
        draw_line(x1, y1, x2, y2, &mut img, Rgb([255, 0, 0]));
        draw_line(x3, y3, x2, y2, &mut img, Rgb([0, 255, 0]));
        draw_line(x3, y3, x1, y1, &mut img, Rgb([0, 0, 255]));
        imageops::flip_vertical_in_place(&mut img);
        img.save("output/triangle.tga").unwrap();
    }

    #[test]
    fn draw_model_with_line() {
        let height = 1024;
        let width = 1024;
        let mut img = RgbImage::new(width + 100, height + 100);
        let model = Model::load_model("obj/alligator.obj").unwrap();
        let vertex_to_pixel = |n: f64, scale: u32| {
            let scale = scale as f64;
            ((n + 1.0) * scale / 2.0) as u32
        };
        model.faces.iter().for_each(|&(i, j, k)| {
            // let (xi, yi, _) = model.vertices[i];
            // let (xj, yj, _) = model.vertices[j];
            // let (xk, yk, _) = model.vertices[k];
            let tri = [model.vertices[i], model.vertices[j], model.vertices[k]];
            for i in 0..3 {
                let v0 = tri[i];
                let v1 = tri[(i + 1) % 3];

                let x0 = vertex_to_pixel(v0.0, width);
                let y0 = vertex_to_pixel(v0.1, height);

                let x1 = vertex_to_pixel(v1.0, width);
                let y1 = vertex_to_pixel(v1.1, height);
                draw_line(x0, y0, x1, y1, &mut img, Rgb([255, 255, 255]));
            }
        });

        imageops::flip_vertical_in_place(&mut img);
        img.save("output/alligator.tga").unwrap();
    }
}
