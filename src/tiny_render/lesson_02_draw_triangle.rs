use image::{imageops, GenericImage};
use nalgebra as na;

#[derive(Clone)]
pub struct Point2D {
    pub x: u32,
    pub y: u32,
}

fn slope(pa: &Point2D, pb: &Point2D) -> f64 {
    (pb.y as f64 - pa.y as f64) / (pb.x as f64 - pa.x as f64)
}

pub fn naive_draw_triangle<I: GenericImage>(
    p0: Point2D,
    p1: Point2D,
    p2: Point2D,
    img: &mut I,
    pixel: I::Pixel,
) {
    let mut vertices = [p0, p1, p2];
    vertices.sort_by(|a, b| a.y.cmp(&b.y));

    // assume type I triangle
    // ---a------------------
    // ----**----------------
    // -----****-------------
    // ------******----------
    // -------********b------
    // --------*******-------
    // ---------****---------
    // ----------c-----------
    let ya = vertices[2].y;
    let yb = vertices[1].y;
    let yc = vertices[0].y;

    // left bound ac, right bound ab
    // y from ya down to yb
    let k_ac = slope(&vertices[2], &vertices[0]);
    let k_ab = slope(&vertices[2], &vertices[1]);
    let k_bc = slope(&vertices[1], &vertices[0]);
    let mut left_bound_x = vertices[2].x as f64;
    let mut right_bound_x = left_bound_x;
    // when dy is 1, dx change
    let dx_left = -1.0 / k_ac;
    let dx_right = -1.0 / k_ab;
    let dx_right2 = -1.0 / k_bc;

    for y in (yb..=ya).rev() {
        for x in left_bound_x as u32..=right_bound_x as u32 {
            img.put_pixel(x, y, pixel);
        }

        left_bound_x += dx_left;
        right_bound_x += dx_right;
    }

    for y in (yc..=yb).rev() {
        for x in left_bound_x as u32..=right_bound_x as u32 {
            img.put_pixel(x, y, pixel);
        }

        left_bound_x += dx_left;
        right_bound_x += dx_right2;
    }

    imageops::flip_vertical_in_place(img);
}

pub fn draw_triangle_upper_and_down<I: GenericImage>(
    p0: Point2D,
    p1: Point2D,
    p2: Point2D,
    img: &mut I,
    pixel: I::Pixel,
) {
    let mut p0 = p0;
    let mut p1 = p1;
    let mut p2 = p2;

    if p0.y > p1.y {
        std::mem::swap(&mut p0, &mut p1);
    }
    if p0.y > p2.y {
        std::mem::swap(&mut p0, &mut p2);
    }
    if p1.y > p2.y {
        std::mem::swap(&mut p1, &mut p2);
    }

    let total_height = p2.y - p0.y;

    for y in p0.y..=p1.y {
        let segment_height = p1.y - p0.y + 1;
        let alpha = (y as f64 - p0.y as f64) / total_height as f64;
        let beta = (y as f64 - p0.y as f64) / segment_height as f64;
        let a_x = (p0.x as f64 + (p2.x as f64 - p0.x as f64) * alpha) as u32;
        let b_x = (p0.x as f64 + (p1.x as f64 - p0.x as f64) * beta) as u32;

        for x in a_x.min(b_x)..=a_x.max(b_x) {
            img.put_pixel(x, y, pixel);
        }
    }

    for y in p1.y..=p2.y {
        let segment_height = p2.y - p1.y + 1;
        let alpha = (y as f64 - p0.y as f64) / total_height as f64;
        let beta = (y as f64 - p1.y as f64) / segment_height as f64;
        let a_x = (p0.x as f64 + (p2.x as f64 - p0.x as f64) * alpha) as u32;
        let b_x = (p1.x as f64 + (p2.x as f64 - p1.x as f64) * beta) as u32;

        for x in a_x.min(b_x)..=a_x.max(b_x) {
            img.put_pixel(x, y, pixel);
        }
    }
}

fn is_in_triangle(p: &Point2D, t0: &Point2D, t1: &Point2D, t2: &Point2D) -> bool {
    // NOTE: Solve linear system
    // -->    -->    -->
    // BP = u BA + v BC
    // Solve u v
    let bp = na::Vector2::new(p.x as f64 - t1.x as f64, p.y as f64 - t1.y as f64);
    let ba = na::Vector2::new(t0.x as f64 - t1.x as f64, t0.y as f64 - t1.y as f64);
    let bc = na::Vector2::new(t2.x as f64 - t1.x as f64, t2.y as f64 - t1.y as f64);

    let mat = na::matrix![
        ba.x, bc.x;
        ba.y, bc.y;
    ];

    // NOTE: if there is no inverse, means the triangle degenerate
    // We don't draw the line
    // return false
    mat.try_inverse().map_or(false, |mat_inv| {
        let uv = mat_inv * bp;
        uv.x >= 0.0 && uv.y >= 0.0 && uv.x + uv.y <= 1.0
    })
}

pub fn draw_triangle_using_bounding_box<I: GenericImage>(
    t0: Point2D,
    t1: Point2D,
    t2: Point2D,
    img: &mut I,
    pixel: I::Pixel,
) {
    let x_left = t0.x.min(t1.x).min(t2.x);
    let x_right = t0.x.max(t1.x).max(t2.x).min(img.width() - 1);
    let y_bottom = t0.y.min(t1.y).min(t2.y);
    let y_top = t0.y.max(t1.y).max(t2.y).min(img.height() - 1);

    for x in x_left..=x_right {
        for y in y_bottom..=y_top {
            if is_in_triangle(&Point2D { x, y }, &t0, &t1, &t2) {
                img.put_pixel(x, y, pixel);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiny_render::Model;
    use image::{Rgb, RgbImage};
    use rand::Rng;

    #[test]
    fn test_triangle() {
        let mut img = RgbImage::new(800, 800);
        let p0 = Point2D { x: 200, y: 600 };
        let p1 = Point2D { x: 600, y: 400 };
        let p2 = Point2D { x: 400, y: 100 };
        let pixel = Rgb([255, 0, 0]);
        naive_draw_triangle(p0, p1, p2, &mut img, pixel);
        img.save("output/triangle_filled.tga").unwrap();
    }

    #[test]
    fn test_draw_triangle_upper_and_down() {
        let mut img = RgbImage::new(800, 800);
        let p0 = Point2D { x: 200, y: 600 };
        let p1 = Point2D { x: 600, y: 400 };
        let p2 = Point2D { x: 400, y: 100 };
        let pixel = Rgb([255, 0, 0]);
        draw_triangle_upper_and_down(p0, p1, p2, &mut img, pixel);

        let p0 = Point2D { x: 10, y: 90 };
        let p1 = Point2D { x: 20, y: 20 };
        let p2 = Point2D { x: 80, y: 150 };
        let pixel = Rgb([0, 255, 0]);
        draw_triangle_upper_and_down(p0, p1, p2, &mut img, pixel);
        imageops::flip_vertical_in_place(&mut img);
        img.save("output/triangle_filled_upper_and_down.tga")
            .unwrap();
    }

    #[test]
    fn test_draw_triangle_using_bounding_box() {
        let mut img = RgbImage::new(800, 800);
        let p0 = Point2D { x: 200, y: 600 };
        let p1 = Point2D { x: 600, y: 400 };
        let p2 = Point2D { x: 400, y: 100 };
        let pixel = Rgb([255, 0, 0]);
        draw_triangle_using_bounding_box(p0, p1, p2, &mut img, pixel);

        let p0 = Point2D { x: 10, y: 90 };
        let p1 = Point2D { x: 20, y: 20 };
        let p2 = Point2D { x: 80, y: 150 };
        let pixel = Rgb([0, 255, 0]);
        draw_triangle_using_bounding_box(p0, p1, p2, &mut img, pixel);

        imageops::flip_vertical_in_place(&mut img);
        img.save("output/triangle_bounding_box.tga").unwrap();
    }

    #[test]
    fn test_draw_illuminated_head() {
        let mut img = RgbImage::new(800, 800);
        let model = Model::load_model("obj/head.obj").unwrap();

        let vertex_to_pixel = |n: f64, scale: u32| {
            let scale = scale as f64;
            ((n + 1.0) * scale / 2.0) as u32
        };

        let get_intensity = |tri: [(f64, f64, f64); 3]| {
            let t0 = na::Vector3::new(tri[0].0, tri[0].1, tri[0].2);
            let t1 = na::Vector3::new(tri[1].0, tri[1].1, tri[1].2);
            let t2 = na::Vector3::new(tri[2].0, tri[2].1, tri[2].2);
            let orth = (t2 - t0).cross(&(t1 - t0)).normalize();

            orth.dot(&na::Vector3::new(0.0, 0.0, -1.0).normalize())
        };

        model.faces.iter().for_each(|&(i, j, k)| {
            let tri = [model.vertices[i], model.vertices[j], model.vertices[k]];
            let intensity = get_intensity(tri);

            let mut tri_2d = tri
                .iter()
                .map(|&(x, y, _z)| Point2D {
                    x: vertex_to_pixel(x, img.width()),
                    y: vertex_to_pixel(y, img.height()),
                })
                .collect::<Vec<_>>();

            if intensity > 0.0 {
                draw_triangle_using_bounding_box(
                    tri_2d.pop().unwrap(),
                    tri_2d.pop().unwrap(),
                    tri_2d.pop().unwrap(),
                    &mut img,
                    Rgb([
                        (255.0 * intensity) as u8,
                        (255.0 * intensity) as u8,
                        (255.0 * intensity) as u8,
                    ]),
                );
            }
        });

        imageops::flip_vertical_in_place(&mut img);
        img.save("output/illuminated_head.tga").unwrap();
    }

    #[test]
    fn test_draw_colorful_head() {
        let mut img = RgbImage::new(2048, 2048);
        let model = Model::load_model("obj/head.obj").unwrap();

        let vertex_to_pixel = |n: f64, scale: u32| {
            let scale = scale as f64;
            ((n + 1.0) * scale / 2.0) as u32
        };

        let mut rng = rand::rng();

        let mut get_random_color = || {
            let r = rng.random_range(0..=255);
            let g = rng.random_range(0..=255);
            let b = rng.random_range(0..=255);
            Rgb([r, g, b])
        };

        model.faces.iter().for_each(|&(i, j, k)| {
            let tri = [model.vertices[i], model.vertices[j], model.vertices[k]];
            let mut tri = tri
                .iter()
                .map(|&(x, y, _z)| Point2D {
                    x: vertex_to_pixel(x, img.width()),
                    y: vertex_to_pixel(y, img.height()),
                })
                .collect::<Vec<_>>();
            draw_triangle_using_bounding_box(
                tri.pop().unwrap(),
                tri.pop().unwrap(),
                tri.pop().unwrap(),
                &mut img,
                get_random_color(),
            );
        });

        imageops::flip_vertical_in_place(&mut img);
        img.save("output/colorful_head.tga").unwrap();
    }
}
