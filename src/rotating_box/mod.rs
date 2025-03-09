use image::{imageops::flip_vertical_in_place, Rgb, RgbImage};
use imageproc::drawing::draw_line_segment_mut;
use nalgebra::{matrix, Matrix4, Vector3, Vector4};
use rand::Rng;

const FRAME: usize = 60;

pub struct Box3D {
    // NOTE: world coordinates
    vertices: Vec<Vector4<f64>>,
}

impl Box3D {
    pub fn new(low: Vector4<f64>, high: Vector4<f64>) -> Self {
        let dx = Vector4::new(high.x - low.x, 0.0, 0.0, 0.);
        let dy = Vector4::new(0.0, high.y - low.y, 0.0, 0.);
        let dz = Vector4::new(0.0, 0.0, high.z - low.z, 0.);
        let vertices = vec![
            low,
            low + dx,
            low + dy,
            low + dz,
            high,
            high - dx,
            high - dy,
            high - dz,
        ];
        Self { vertices }
    }

    fn low(&self) -> Vector4<f64> {
        self.vertices[0]
    }

    fn high(&self) -> Vector4<f64> {
        self.vertices[4]
    }

    pub fn rotate(&mut self, theta: f64) {
        let rot = matrix![
            theta.cos(), 0., theta.sin(), 0.;
            0., 1., 0., 0.;
            -theta.sin(), 0., theta.cos(), 0.;
            0., 0., 0., 1.;
        ];
        for v in &mut self.vertices {
            *v = rot * *v;
        }
    }

    pub fn rotate_aroud_axis(&mut self, theta: f64, axis: Vector3<f64>) {
        let w = axis.normalize();
        let t = Vector3::new(w.x, w.y, w.z + 1.);
        let u = t.cross(&w).normalize();
        let v = w.cross(&u).normalize();
        let w = Vector4::new(w.x, w.y, w.z, 0.);
        let v = Vector4::new(v.x, v.y, v.z, 0.);
        let u = Vector4::new(u.x, u.y, u.z, 0.);
        let d_low_high = self.high() - self.low();
        let cube_center = self.low() + d_low_high / 2.;
        let world_to_local = Matrix4::from_columns(&[u, v, w, cube_center]);
        let local_to_world = world_to_local.try_inverse().unwrap();
        let rot = matrix![
            theta.cos(), -theta.sin(), 0.0, 0.0;
            theta.sin(), theta.cos(),  0.0, 0.0;
            0.0,         0.0,          1.0, 0.0;
            0.0,         0.0,          0.0, 1.0;
        ];

        for v in &mut self.vertices {
            *v = world_to_local * rot * local_to_world * *v;
        }
    }

    fn edges(&self) -> Vec<(Vector4<f64>, Vector4<f64>)> {
        let mut edges = Vec::new();
        let edges_indices = [
            (0, 1),
            (0, 2),
            (0, 3),
            (3, 6),
            (1, 6),
            (2, 5),
            (2, 7),
            (1, 7),
            (3, 5),
            (4, 5),
            (4, 6),
            (4, 7),
        ];

        for &(i, j) in &edges_indices {
            edges.push((self.vertices[i], self.vertices[j]));
        }

        edges
    }
}

pub struct Camera {
    origin: Vector4<f64>,
    forward: Vector4<f64>,
    up: Vector4<f64>,
    right: Vector4<f64>,
    view_volume: Box3D,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            origin: Vector4::new(0., 0., 0., 1.),
            forward: Vector4::new(0., 0., -1., 0.),
            up: Vector4::new(0., 1., 0., 0.),
            right: Vector4::new(1., 0., 0., 0.),
            // NOTE: Camera Space
            view_volume: Box3D::new(
                Vector4::new(-3f64.sqrt() / 3., -3f64.sqrt() / 3., -100., 1.),
                Vector4::new(3f64.sqrt() / 3., 3f64.sqrt() / 3., -1., 1.),
            ),
        }
    }
}

impl Camera {
    fn world_to_cam_transform(&self) -> Matrix4<f64> {
        Matrix4::from_columns(&[self.right, self.up, -self.forward, self.origin])
    }

    fn cam_to_world_transform(&self) -> Matrix4<f64> {
        self.world_to_cam_transform().try_inverse().unwrap()
    }

    pub fn perspective_transform(&self) -> Matrix4<f64> {
        let n = self.view_volume.high().z;
        let f = self.view_volume.low().z;
        matrix![
            n ,    0.,    0. ,     0. ;
            0.,    n ,    0. ,     0. ;
            0.,    0.,   n+f ,  -f * n;
            0.,    0.,    0. ,     0. ;
        ]
    }

    fn orth_perspective_transform(&self) -> Matrix4<f64> {
        let n = self.view_volume.high().z;
        let f = self.view_volume.low().z;
        let l = self.view_volume.low().x;
        let r = self.view_volume.high().x;
        let b = self.view_volume.low().y;
        let t = self.view_volume.high().y;
        matrix![
            2. * n / (r - l), 0.               , (l + r) / (l - r) , 0.                   ;
            0.              , 2. * n / (t - b) , (b + t) / (b - t) , 0.                   ;
            0.              , 0.               , (f + n) / (n - f) , -2. * f * n / (f - n);
            0.              , 0.               , 1.                , 0.                   ;
        ]
    }

    fn view_port_transform(&self, width: f64, height: f64) -> Matrix4<f64> {
        matrix![
            width / 2., 0.         , 0., (width - 1.) / 2. ;
            0.        , height / 2., 0., (height - 1.) / 2.;
            0.        , 0.         , 1., 0.                ;
            0.        , 0.         , 0., 1.                ;
        ]
    }

    pub fn all_in_one_transform(&self) -> Matrix4<f64> {
        let n = self.view_volume.high().z;
        let f = self.view_volume.low().z;
        let l = self.view_volume.low().x;
        let r = self.view_volume.high().x;
        let b = self.view_volume.low().y;
        let t = self.view_volume.high().y;
        matrix![
            (2. * n) / (r - l),  0.               , (l + r) / (l - r), 0.                    ;
            0.                , (2. * n) / (t - b), (b + t) / (b - t), 0.                    ;
            0.                ,  0.               , (f + n) / (n - f), (2. * f * n) / (f - n);
            0.                ,  0.               , 1.               , 0.                    ;
        ]
    }

    pub fn render(&self, width: usize, height: usize, bx: &Box3D) {
        let mut img = image::RgbImage::new(width as u32, height as u32);
        let mvp = self.view_port_transform(width as f64, height as f64);
        let mper = self.orth_perspective_transform();
        let mcam = self.cam_to_world_transform();

        for (p1, p2) in bx.edges() {
            let p1_per = mvp * mper * mcam * p1;
            let p2_per = mvp * mper * mcam * p2;
            let p1_2d = ((p1_per.x / p1_per.w) as f32, (p1_per.y / p1_per.w) as f32);
            let p2_2d = ((p2_per.x / p2_per.w) as f32, (p2_per.y / p2_per.w) as f32);

            draw_line_segment_mut(&mut img, p1_2d, p2_2d, Rgb([255, 255, 255]));
        }

        flip_vertical_in_place(&mut img);
        img.save("output/rotation_box.png").unwrap();
    }

    fn draw_boxes(&self, img: &mut RgbImage, bx: &mut [Box3D], width: usize, height: usize) {
        let mvp = self.view_port_transform(width as f64, height as f64);
        let mper = self.orth_perspective_transform();
        let mcam = self.cam_to_world_transform();
        let mut rng = rand::rng();

        for b in bx {
            let random_color = Rgb([
                rng.random_range(0..255),
                rng.random_range(0..255),
                rng.random_range(0..255),
            ]);
            for (p1, p2) in b.edges() {
                let p1_per = mvp * mper * mcam * p1;
                let p2_per = mvp * mper * mcam * p2;
                let p1_2d = ((p1_per.x / p1_per.w) as f32, (p1_per.y / p1_per.w) as f32);
                let p2_2d = ((p2_per.x / p2_per.w) as f32, (p2_per.y / p2_per.w) as f32);

                draw_line_segment_mut(img, p1_2d, p2_2d, random_color);
            }
        }
    }

    pub fn render_rotation_box(&self, width: usize, height: usize, bx: &mut [Box3D]) {
        // frame is 60
        for t in 0..=FRAME {
            // bx.rotate(std::f64::consts::PI / 60.);
            let mut img = image::RgbImage::new(width as u32, height as u32);
            self.draw_boxes(&mut img, bx, width, height);

            flip_vertical_in_place(&mut img);
            img.save(format!("output/rotation/rotation_box_{t}.png"))
                .unwrap();

            // bx.rotate(2. * std::f64::consts::PI / FRAME as f64);
            for b in bx.iter_mut() {
                b.rotate_aroud_axis(
                    2. * std::f64::consts::PI / FRAME as f64,
                    Vector3::new(1., 1., -1.),
                );
            }
        }
    }
}

#[test]
fn test_render() {
    let theta = 30f64.to_radians();
    let camera = Camera {
        origin: Vector4::new(-5., 12., 15., 1.),
        forward: Vector4::new(0., -theta.sin(), -theta.cos(), 0.),
        ..Camera::default()
    };
    let bx = Box3D::new(
        Vector4::new(-2., -2., -15., 1.),
        Vector4::new(2., 2., -11., 1.),
    );

    camera.render(800, 800, &bx);
}

#[test]
fn test_render_rotation_box() {
    let theta = 30f64.to_radians();
    let camera = Camera {
        origin: Vector4::new(-5., 12., 15., 1.),
        forward: Vector4::new(0., -theta.sin(), -theta.cos(), 0.),
        ..Camera::default()
    };
    let mut boxes = vec![Box3D::new(
        Vector4::new(-2., -2., -2., 1.),
        Vector4::new(2., 2., 2., 1.),
    )];

    camera.render_rotation_box(200, 200, &mut boxes);
}

#[test]
fn test_render_rotation_box_around_axis() {
    let theta = 30f64.to_radians();
    let camera = Camera {
        origin: Vector4::new(-5., 18., 30., 1.),
        forward: Vector4::new(0., -theta.sin(), -theta.cos(), 0.),
        ..Camera::default()
    };
    let mut boxes = vec![
        Box3D::new(
            Vector4::new(-8., -12., -15., 1.),
            Vector4::new(-4., -8., -11., 1.),
        ),
        Box3D::new(
            Vector4::new(-12., -4., -19., 1.),
            Vector4::new(-6., 2., -13., 1.),
        ),
        Box3D::new(
            Vector4::new(-2., -4., -17., 1.),
            Vector4::new(6., 4., -9., 1.),
        ),
        Box3D::new(
            Vector4::new(-15., -11., -26., 1.),
            Vector4::new(7., 11., -4., 1.),
        ),
    ];

    camera.render_rotation_box(200, 200, &mut boxes);
}

#[test]
fn test_cam_transform() {
    let camera = Camera {
        origin: Vector4::new(0., 2., 2., 1.),
        ..Camera::default()
    };

    let cam_to_world = camera.cam_to_world_transform();
    let p = Vector4::new(1., 1., 1., 1.);
    println!("{:?}", cam_to_world * p);
}
