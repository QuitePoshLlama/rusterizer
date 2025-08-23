#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

use std::ops::{Add, Sub, Mul, Div};

impl Add for Point3D {
    type Output = Point3D;
    fn add(self, other: Point3D) -> Point3D {
        Point3D { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
}

impl Add<f32> for Point3D {
    type Output = Point3D;
    fn add(self, scalar: f32) -> Point3D {
        Point3D { x: self.x + scalar, y: self.y + scalar, z: self.z + scalar }
    }
}

impl Sub for Point3D {
    type Output = Point3D;
    fn sub(self, other: Point3D) -> Point3D {
        Point3D { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

impl Mul<f32> for Point3D {
    type Output = Point3D;
    fn mul(self, scalar: f32) -> Point3D {
        Point3D { x: self.x * scalar, y: self.y * scalar, z: self.z * scalar }
    }
}

impl Div<f32> for Point3D {
    type Output = Point3D;
    fn div(self, scalar: f32) -> Point3D {
        Point3D { x: self.x / scalar, y: self.y / scalar, z: self.z / scalar }
    }
}

impl Div<Point3D> for f32 {
    type Output = Point3D;
    fn div(self, rhs: Point3D) -> Point3D {
        Point3D { x: self / rhs.x, y: self / rhs.y, z: self / rhs.z }
    }
}

pub fn dot3(a: Point3D, b: Point3D) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub fn normalize(vec: Point3D) -> Point3D {
    let length = dot3(vec, vec).sqrt();
    if length != 0.0 { vec / length } else { vec }
}
