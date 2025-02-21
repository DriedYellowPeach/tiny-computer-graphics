use anyhow::{bail, Result};
use image::{imageops, DynamicImage, GenericImage, GenericImageView, Pixel, Rgb};
use nalgebra::{matrix, Vector2, Vector3};
use std::{fs::File, io::BufRead, path::Path};

// NOTE: We need to apply barycentric coordinates
// this will helps us to determine the texture cordinates
// triangle ABC, Point P
// ->             ->   ->   ->
// P = (1 - u - v)A + uB + vC;
#[allow(unused)]
fn barycentric_coordinates(triangle: &[Vector3<f64>], p: Vector3<f64>) -> Vector3<f64> {
    let a = triangle[0];
    let b = triangle[1];
    let c = triangle[2];

    let ab = b - a;
    let ac = c - a;
    let pa = a - p;

    let ret = Vector3::new(ab.x, ac.x, pa.x).cross(&Vector3::new(ab.y, ac.y, pa.y));

    let u = ret.x / ret.z;
    let v = ret.y / ret.z;

    Vector3::new(1.0 - u - v, u, v)
}

fn barycentric_coordinates2(triangle: &[Vector3<f64>], p: Vector3<f64>) -> Vector3<f64> {
    // NOTE: Solve linear system
    // -->      -->      -->
    // BP_x = u BA_x + v BC_x
    // BP_y = u BA_y + v BC_y
    // Solve u v
    let (a, b, c) = (triangle[0], triangle[1], triangle[2]);
    let bp = p - b;
    let ba = a - b;
    let bc = c - b;

    let mat = matrix![
        ba.x, bc.x;
        ba.y, bc.y;
    ];

    // NOTE: if there is no inverse, means the triangle degenerate to a line
    // and the p not on the line
    // we return a Vector3 with negative values
    // so we don't include this point
    mat.try_inverse()
        .map_or(Vector3::new(-1.0, 1.0, 1.0), |mat_inv| {
            let uv = mat_inv * Vector2::new(bp.x, bp.y);
            Vector3::new(uv.x, 1.0 - uv.x - uv.y, uv.y)
        })
}

pub struct Face {
    vertex_idx: Vector3<usize>,
    texture_idx: Vector3<usize>,
}

#[derive(Default)]
pub struct Model {
    pub vertices: Vec<Vector3<f64>>,
    pub textures: Vec<Vector2<f64>>,
    pub faces: Vec<Face>,
    pub texture_color_map: Option<DynamicImage>,
}

impl Model {
    fn parse_vertex(text: &str) -> Result<Vector3<f64>> {
        let parts = text
            .split_whitespace()
            .filter_map(|num| num.parse::<f64>().ok())
            .collect::<Vec<_>>();

        if parts.len() != 3 {
            bail!("Failed to parse vertext line: {text}");
        }

        Ok(Vector3::new(parts[0], parts[1], parts[2]))
    }

    fn parse_texture(text: &str) -> Result<Vector2<f64>> {
        let parts = text
            .split_whitespace()
            .filter_map(|num| num.parse::<f64>().ok())
            .collect::<Vec<_>>();

        if parts.len() < 2 {
            bail!(
                "Failed to parse texture line: {text} {parts:?} {}",
                parts.len()
            );
        }

        Ok(Vector2::new(parts[0], parts[1]))
    }

    fn parse_face(text: &str) -> Result<Face> {
        let parts = text
            .split_whitespace()
            .flat_map(|nums| nums.split('/'))
            .filter_map(|num| num.parse::<usize>().ok().map(|n| n - 1)) // NOTE: in .obj file, the index starts from 1
            .collect::<Vec<_>>();

        if parts.len() != 9 {
            bail!("Failed to parse face line: {text}");
        }

        // NOTE: parts format
        // 0 1 2
        // 3 4 5
        // 6 7 8
        Ok(Face {
            vertex_idx: Vector3::new(parts[0], parts[3], parts[6]),
            texture_idx: Vector3::new(parts[1], parts[4], parts[7]),
        })
    }

    pub fn load_texture<P: AsRef<Path>>(self, texture_path: P) -> Result<Self> {
        let mut m = self;
        let mut img = image::open(texture_path)?;
        imageops::flip_vertical_in_place(&mut img);
        m.texture_color_map = Some(img);

        Ok(m)
    }

    pub fn load_model<P: AsRef<Path>>(self, obj_path: P) -> Result<Self> {
        let mut m = self;
        let file = File::open(obj_path)?;
        let reader = std::io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?; // Handle Result<String>
            if line.starts_with("v ") {
                m.vertices.push(Self::parse_vertex(&line)?);
                continue;
            }

            if line.starts_with("vt ") {
                m.textures.push(Self::parse_texture(&line)?);
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

fn get_light_intensity(tri: &[Vector3<f64>]) -> f64 {
    let t0 = Vector3::new(tri[0].x, tri[0].y, tri[0].z);
    let t1 = Vector3::new(tri[1].x, tri[1].y, tri[1].z);
    let t2 = Vector3::new(tri[2].x, tri[2].y, tri[2].z);
    let orth = (t2 - t0).cross(&(t1 - t0)).normalize();

    orth.dot(&Vector3::new(0.0, 0.0, -1.0).normalize())
}

fn world_to_screen(v: &Vector3<f64>, width: u32, height: u32) -> Vector3<f64> {
    let w = width as f64;
    let h = height as f64;

    Vector3::new(
        (v.x + 1.0) * w / 2.0 + 0.5,
        (v.y + 1.0) * h / 2.0 + 0.5,
        v.z,
    )
}

fn bound_box(pts: &[Vector3<f64>], width: u32, height: u32) -> (Vector2<f64>, Vector2<f64>) {
    let w = width as f64;
    let h = height as f64;

    let mut bboxmin = Vector2::new(f64::MAX, f64::MAX);
    let mut bboxmax = Vector2::new(f64::MIN, f64::MIN);
    let clamp = Vector2::new(w - 1.0, h - 1.0);

    // NOTE: consider all three points
    for v in pts.iter() {
        // NOTE: iter from x to y
        for j in 0..2 {
            bboxmin[j] = 0.0f64.max(bboxmin[j].min(v[j]));
            bboxmax[j] = clamp[j].min(bboxmax[j].max(v[j]));
        }
    }

    (bboxmin, bboxmax)
}

pub fn rasterize_3d_triangle<I>(
    pts: &[Vector3<f64>],
    textures: &[Vector2<f64>],
    z_buffer: &mut [f64],
    img: &mut I,
    model: &Model,
) where
    I: GenericImage<Pixel = Rgb<u8>>,
{
    // NOTE: step 1: before scale world coordinates to screen, get intensity
    let intensity = get_light_intensity(pts);

    // NOTE: step 2: world coordinates to screen
    let pts = pts
        .iter()
        .map(|v| world_to_screen(v, img.width(), img.height()))
        .collect::<Vec<_>>();

    // NOTE: step 3: get bounding box
    let (bboxmin, bboxmax) = bound_box(&pts, img.width(), img.height());

    for x in bboxmin.x as u32..=bboxmax.x as u32 {
        for y in bboxmin.y as u32..=bboxmax.y as u32 {
            let p = Vector3::new(x as f64, y as f64, 0.0);
            let coe = barycentric_coordinates2(&pts, p);
            let z_idx = (p.x as u32 + p.y as u32 * img.width()) as usize;

            // NOTE: test is in triangle
            // if not, don't draw
            if coe.iter().any(|&x| x < 0.0) || intensity < 0.0 {
                continue;
            }

            // TODO: write it using matrix multiplication
            // NOTE: apply texture if can

            let pixel = if let Some(ref color_map) = model.texture_color_map {
                let p_texture = coe.x * textures[0] + coe.y * textures[1] + coe.z * textures[2];
                let texture_w = color_map.width() as f64 * p_texture.x;
                let texture_h = color_map.height() as f64 * p_texture.y;
                let rgb = color_map
                    .get_pixel(texture_w as u32, texture_h as u32)
                    .to_rgb();
                Vector3::new(rgb[0] as f64, rgb[1] as f64, rgb[2] as f64)
            } else {
                Vector3::new(255.0, 255.0, 255.0)
            };

            // NOTE: step 4: get current color for current pixel

            // NOTE: step 5: apply intensity to color
            let color_bit = (pixel * intensity)
                .map(|x| x.clamp(0.0, 255.0) as u8)
                .into();

            if z_buffer[z_idx] < p.z {
                z_buffer[z_idx] = p.z;
                img.put_pixel(p.x as u32, p.y as u32, Rgb(color_bit));
            }
        }
    }
}

pub fn draw_model<I>(model: Model, img: &mut I)
where
    I: GenericImage<Pixel = Rgb<u8>>,
{
    let mut z_buffer = vec![f64::MIN; (img.width() * img.height()) as usize];

    model.faces.iter().for_each(|face| {
        let v0 = model.vertices[face.vertex_idx.x];
        let v1 = model.vertices[face.vertex_idx.y];
        let v2 = model.vertices[face.vertex_idx.z];

        let t0 = model.textures[face.texture_idx.x];
        let t1 = model.textures[face.texture_idx.y];
        let t2 = model.textures[face.texture_idx.z];

        let pts = [v0, v1, v2];
        let textures = [t0, t1, t2];

        rasterize_3d_triangle(&pts, &textures, &mut z_buffer, img, &model);
    });
}

#[cfg(test)]
mod tests {
    use image::{imageops, RgbImage};

    use super::*;

    #[test]
    fn test_barycentric_coordinates() {
        let triangle = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(4.0, 0.0, 0.0),
            Vector3::new(0.0, 4.0, 0.0),
        ];

        let p = Vector3::new(1.0, 1.0, 0.0);
        let b = barycentric_coordinates(&triangle, p);
        assert_eq!(p, triangle[0] * b.x + triangle[1] * b.y + triangle[2] * b.z);

        let triangle = [
            Vector3::new(1.0, 1.0, 5.0),
            Vector3::new(4.0, 0.0, 3.0),
            Vector3::new(0.0, 4.0, 1.0),
        ];

        let p = Vector3::new(1.0, 1.0, 5.0);
        let b = barycentric_coordinates(&triangle, p);
        assert_eq!(p, triangle[0] * b.x + triangle[1] * b.y + triangle[2] * b.z);
        assert_eq!(b, Vector3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_draw_head_removing_hidden_faces() {
        let mut img = RgbImage::new(800, 800);
        let model = Model::default().load_model("obj/head.obj").unwrap();

        draw_model(model, &mut img);

        imageops::flip_vertical_in_place(&mut img);
        img.save("output/head_using_barycentric_2.tga").unwrap();
    }

    #[test]
    fn test_draw_head_with_texture() {
        let mut img = RgbImage::new(800, 800);
        let model = Model::default()
            .load_model("obj/head.obj")
            .unwrap()
            .load_texture("obj/african_head_diffuse.tga")
            .unwrap();

        draw_model(model, &mut img);

        imageops::flip_vertical_in_place(&mut img);
        img.save("output/head_with_texture.tga").unwrap();
    }
}
