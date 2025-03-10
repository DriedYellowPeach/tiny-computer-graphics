use approx::{relative_eq, AbsDiffEq};
use image::{Pixel, Rgb};
use indicatif::{ProgressState, ProgressStyle};
use nalgebra::{Matrix3x4, Vector3, Vector4};

use std::fmt::Write;
use std::ops::{Add, Div, Mul, Range};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(Vector3<f64>);

pub const EPSILON: f64 = 1e-6;

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(Vector3::new(x, y, z))
    }

    pub fn move_forward(&self, distance: f64, direction: &Direction) -> Self {
        Self::from(self.0 + distance * direction.0)
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        (other.0 - self.0).magnitude()
    }
}

impl From<Vector3<f64>> for Position {
    fn from(v: Vector3<f64>) -> Self {
        Self(v)
    }
}

impl AsRef<Vector3<f64>> for Position {
    fn as_ref(&self) -> &Vector3<f64> {
        &self.0
    }
}

impl AbsDiffEq for Position {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon {
        EPSILON
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        relative_eq!(self.0, other.0, epsilon = epsilon)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Direction(Vector3<f64>);

impl AbsDiffEq for Direction {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon {
        EPSILON
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        relative_eq!(self.0, other.0, epsilon = epsilon)
    }
}

impl Direction {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(Vector3::new(x, y, z).normalize())
    }

    pub fn a_to_b(a: &Position, b: &Position) -> Self {
        Self::from(b.0 - a.0)
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.0.dot(&other.0)
    }

    pub fn reverse(&self) -> Self {
        Self::from(-self.0)
    }

    pub fn is_acute_angle(&self, other: &Self) -> bool {
        self.0.dot(&other.0) > 0.
    }

    #[allow(non_snake_case)]
    pub fn reflection(&self, N: &Self) -> Self {
        let I = self;
        let I_proj = I.0.dot(&N.0) * N.0;

        Self::from(I.0 - 2. * I_proj)
    }

    #[allow(non_snake_case)]
    pub fn refraction(&self, N: &Self, n1: f64, n2: f64) -> Self {
        let N = *N;
        let I = self;

        let cos_theta1 = -I.0.dot(&N.0).clamp(-1., 1.);
        let sin_theta1 = (1. - cos_theta1.powi(2)).sqrt().clamp(-1., 1.);
        let sin_theta2 = (n1 / n2 * sin_theta1).clamp(-1., 1.);
        let cos_theta2 = (1. - sin_theta2.powi(2)).sqrt().clamp(-1., 1.);

        // NOTE: snell's law: vector form
        // L' = (n1/n2) * L + ((n1/n2)cos(theta1) - cos(theta2)) * N
        Self::from((n1 / n2) * I.0 + ((n1 / n2) * cos_theta1 - cos_theta2) * N.0)
    }
}

impl AsRef<Vector3<f64>> for Direction {
    fn as_ref(&self) -> &Vector3<f64> {
        &self.0
    }
}

impl From<Vector3<f64>> for Direction {
    fn from(v: Vector3<f64>) -> Self {
        Self(v.normalize())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Albedo(Vector4<f64>);

impl Albedo {
    pub const fn new(diffusive: f64, specular: f64, reflective: f64, refractive: f64) -> Self {
        Self(Vector4::new(diffusive, specular, reflective, refractive))
    }

    pub fn diffusive(&self) -> f64 {
        self.0.x
    }

    pub fn specular(&self) -> f64 {
        self.0.y
    }

    pub fn reflective(&self) -> f64 {
        self.0.z
    }

    pub fn refractive(&self) -> f64 {
        self.0.w
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Color(Vector3<f64>);

impl From<Vector3<f64>> for Color {
    fn from(v: Vector3<f64>) -> Self {
        Self(v)
    }
}

impl AsRef<Vector3<f64>> for Color {
    fn as_ref(&self) -> &Vector3<f64> {
        &self.0
    }
}

impl From<Color> for Rgb<u8> {
    fn from(color: Color) -> Self {
        let mut v = color.0;

        let max_chan = v.x.max(v.y).max(v.z);

        // NOTE: normalize the max channel to 1
        if max_chan > 1. {
            v *= 1. / max_chan;
        }

        let color = [v.x, v.y, v.z]
            .into_iter()
            .map(|n| (255. * n.clamp(0., 1.)) as u8)
            .collect::<Vec<_>>();

        Rgb::from_slice(&color).to_owned()
    }
}

impl Color {
    // Predefined color constants
    pub const RED: Color = Color(Vector3::new(1.0, 0.0, 0.0));
    pub const GREEN: Color = Color(Vector3::new(0.0, 1.0, 0.0));
    pub const BLUE: Color = Color(Vector3::new(0.0, 0.0, 1.0));
    pub const WHITE: Color = Color(Vector3::new(1.0, 1.0, 1.0));
    pub const BLACK: Color = Color(Vector3::new(0.0, 0.0, 0.0));
    pub const YELLOW: Color = Color(Vector3::new(1.0, 1.0, 0.0));
    pub const CYAN: Color = Color(Vector3::new(0.0, 1.0, 1.0));
    pub const MAGENTA: Color = Color(Vector3::new(1.0, 0.0, 1.0));

    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Vector3::new(r, g, b))
    }

    pub fn apply_intensity(&self, intensity: f64) -> Self {
        Self::from(self.0 * intensity)
    }

    pub fn apply_albedo(
        diffusive: Color,
        specular: Color,
        reflective: Color,
        refractive: Color,
        albedo: &Albedo,
    ) -> Self {
        let mat = Matrix3x4::from_columns(&[diffusive.0, specular.0, reflective.0, refractive.0]);

        Color::from(mat * albedo.0)
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.0 + rhs.0)
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::from(self.0 * rhs)
    }
}

impl Mul<Color> for f64 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for Color {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self::from(self.0 / rhs)
    }
}

pub struct Interval(pub Range<f64>);

impl Interval {
    pub const FULL: Self = Self(f64::NEG_INFINITY..f64::INFINITY);
    pub const POSITIVE: Self = Self(0.0..f64::INFINITY);

    pub fn new(start: f64, end: f64) -> Self {
        Self(start..end)
    }

    pub fn start(&self) -> f64 {
        self.0.start
    }

    pub fn end(&self) -> f64 {
        self.0.end
    }

    pub fn contains(&self, x: f64) -> bool {
        self.0.contains(&x)
    }

    pub fn surronds(&self, x: f64) -> bool {
        !self.0.contains(&x)
    }
}

pub fn progress_bar_style() -> ProgressStyle {
    let text = [
        "[{bar:48.cyan/blue}] {percent}% {spinner:.green}",
        "Elapsed Time     : {elapsed_precise}",
        "ETA              : {eta}",
        "Tracing Progress : {pos}/{len} pixels",
        "Tracing Speed    : {per_sec}",
    ]
    .join("\n");

    ProgressStyle::with_template(&text)
        .unwrap()
        .with_key("per_sec", |state: &ProgressState, w: &mut dyn Write| {
            _ = write!(w, "{:.0} pixels/sec", state.per_sec());
        })
        .progress_chars("#>-")
}
