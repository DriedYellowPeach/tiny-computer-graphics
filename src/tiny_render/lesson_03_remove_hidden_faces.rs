use super::Point2D;
use image::GenericImage;
use na::Vector3;
use nalgebra::{self as na, Vector2};

pub struct Model {
    pub vertices: Vec<(f64, f64, f64)>,
    // faces stores index, (a, b, c) means ath, bth, and cth vertices form one fase
    pub faces: Vec<(usize, usize, usize)>,
}

pub fn rasterize_2d<I: GenericImage>(
    p0: Point2D,
    p1: Point2D,
    y_buffer: &mut [isize],
    img: &mut I,
    pixel: I::Pixel,
) {
    let (mut p0, mut p1) = (p0, p1);
    if p0.x > p1.x {
        std::mem::swap(&mut p0, &mut p1);
    }

    for x in p0.x..=p1.x {
        let y = if p0.x == p1.x {
            p0.y.max(p1.y) as isize
        } else {
            let t = (x - p0.x) as f64 / (p1.x - p0.x) as f64;
            (p0.y as f64 + t * (p1.y as f64 - p0.y as f64)) as isize
        };

        // NOTE: y_buffer is like a infinity hole
        // for each object, we want to take records the shallowest pixel
        if y_buffer[x as usize] < y {
            y_buffer[x as usize] = y;
            for y in 0..img.height() {
                img.put_pixel(x, y, pixel);
            }
        }
    }
}

fn normal_vector(a: Vector3<f64>, b: Vector3<f64>, c: Vector3<f64>) -> Vector3<f64> {
    let u = b - a;
    let v = c - a;
    u.cross(&v)
}

fn is_in_triangle(
    p: &Vector3<f64>,
    t0: &Vector3<f64>,
    t1: &Vector3<f64>,
    t2: &Vector3<f64>,
) -> bool {
    // NOTE: Solve linear system
    // -->    -->    -->
    // BP = u BA + v BC
    // Solve u v
    let bp = na::Vector2::new(p.x - t1.x, p.y - t1.y);
    let ba = na::Vector2::new(t0.x - t1.x, t0.y - t1.y);
    let bc = na::Vector2::new(t2.x - t1.x, t2.y - t1.y);

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

pub fn rasterize_3d<I: GenericImage>(
    pts: &[Vector3<f64>],
    z_buffer: &mut [f64],
    img: &mut I,
    pixel: I::Pixel,
) {
    let mut bboxmin = Vector2::new(f64::MAX, f64::MAX);
    let mut bboxmax = Vector2::new(f64::MIN, f64::MIN);
    let clamp = Vector2::new(img.width() as f64 - 1.0, img.height() as f64 - 1.0);

    for v in pts.iter() {
        // NOTE: iter from x to y
        for j in 0..2 {
            bboxmin[j] = 0.0f64.max(bboxmin[j].min(v[j]));
            bboxmax[j] = clamp[j].min(bboxmax[j].max(v[j]));
        }
    }

    // for x in bboxmin.x.floor() as u32..=bboxmax.x.ceil() as u32 {
    //     for y in bboxmin.y.floor() as u32..=bboxmax.y.ceil() as u32 {
    for x in bboxmin.x as u32..=bboxmax.x as u32 {
        for y in bboxmin.y as u32..=bboxmax.y as u32 {
            let mut p = Vector3::new(x as f64, y as f64, 0.0);
            let norm = normal_vector(pts[0], pts[1], pts[2]);
            let a = norm.x;
            let b = norm.y;
            let c = norm.z;
            p.z = (a * pts[0].x + b * pts[0].y + c * pts[0].z - a * p.x - b * p.y) / c;
            let idx = (p.x as u32 + p.y as u32 * img.width()) as usize;

            if !is_in_triangle(&p, &pts[0], &pts[1], &pts[2]) {
                continue;
            }

            if z_buffer[idx] < p.z {
                z_buffer[idx] = p.z;
                img.put_pixel(p.x as u32, p.y as u32, pixel);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tiny_render::Model;
    use image::{imageops, Rgb, RgbImage};

    #[test]
    fn test_rasterized_3d() {
        let mut img = RgbImage::new(400, 400);
        let mut z_buffer = vec![f64::MIN; (img.width() * img.height()) as usize];

        let pts = [
            Vector3::new(50.0, 100.0, 0.0),
            Vector3::new(200.0, 200.0, 0.0),
            Vector3::new(300.0, 50.0, 0.0),
        ];

        rasterize_3d(&pts, &mut z_buffer, &mut img, Rgb([255, 255, 255]));
        imageops::flip_vertical_in_place(&mut img);
        img.save("output/head_removing_hidden_faces.tga").unwrap();
    }

    #[test]
    fn test_draw_head_removing_hidden_faces() {
        let mut img = RgbImage::new(800, 800);
        let mut z_buffer = vec![f64::MIN; (img.width() * img.height()) as usize];
        let model = Model::load_model("obj/head.obj").unwrap();
        let scale = |p: f64, scl: u32| (p + 1.0) * (scl as f64) / 2.0 + 0.5;

        let get_intensity = |tri: &[Vector3<f64>]| {
            let t0 = na::Vector3::new(tri[0].x, tri[0].y, tri[0].z);
            let t1 = na::Vector3::new(tri[1].x, tri[1].y, tri[1].z);
            let t2 = na::Vector3::new(tri[2].x, tri[2].y, tri[2].z);
            let orth = (t2 - t0).cross(&(t1 - t0)).normalize();

            orth.dot(&na::Vector3::new(0.0, 0.0, -1.0).normalize())
        };

        model.faces.iter().for_each(|face| {
            let v0 = model.vertices[face.0];
            let v1 = model.vertices[face.1];
            let v2 = model.vertices[face.2];

            // should get intensity before scale
            let pts_before_scale = [v0, v1, v2]
                .into_iter()
                .map(|v| Vector3::new(v.0, v.1, v.2))
                .collect::<Vec<_>>();

            let intensity = get_intensity(&pts_before_scale);

            let pts = pts_before_scale
                .into_iter()
                .map(|v| Vector3::new(scale(v.x, img.width()), scale(v.y, img.height()), v.z))
                .collect::<Vec<_>>();

            if intensity > 0.0 {
                let color_bit = (Vector3::new(255.0, 255.0, 255.0) * intensity)
                    .map(|x| x.clamp(0.0, 255.0) as u8)
                    .into();
                rasterize_3d(&pts, &mut z_buffer, &mut img, Rgb(color_bit));
            }
        });

        imageops::flip_vertical_in_place(&mut img);
        img.save("output/head_removing_hidden_faces.tga").unwrap();
    }

    #[test]
    fn test_render_2d() {
        let mut img = RgbImage::new(800, 16);
        let mut y_buffer = vec![isize::MIN; img.width() as usize];

        rasterize_2d(
            Point2D { x: 20, y: 32 },
            Point2D { x: 744, y: 400 },
            &mut y_buffer,
            &mut img,
            Rgb([255, 0, 0]),
        );

        rasterize_2d(
            Point2D { x: 120, y: 434 },
            Point2D { x: 444, y: 400 },
            &mut y_buffer,
            &mut img,
            Rgb([0, 255, 0]),
        );

        rasterize_2d(
            Point2D { x: 330, y: 463 },
            Point2D { x: 594, y: 200 },
            &mut y_buffer,
            &mut img,
            Rgb([0, 0, 255]),
        );

        imageops::flip_vertical_in_place(&mut img);
        img.save("output/2d_scene_example_rasterize_1d.tga")
            .unwrap();
    }
}
