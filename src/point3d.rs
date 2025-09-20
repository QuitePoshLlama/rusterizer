use std::simd::f32x4;
use std::simd::StdFloat;
use std::simd::num::SimdFloat;
use std::simd::cmp::SimdPartialEq;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point3Dx4 {
    pub x: f32x4,
    pub y: f32x4,
    pub z: f32x4,
}

use std::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign};

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

// AddAssign (for +=)
impl AddAssign for Point3D {
    fn add_assign(&mut self, rhs: Point3D) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Point3D {
    type Output = Point3D;
    fn sub(self, other: Point3D) -> Point3D {
        Point3D { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

// SubAssign (for -=)
impl SubAssign for Point3D {
    fn sub_assign(&mut self, rhs: Point3D) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
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

impl Add for Point3Dx4 {
    type Output = Point3Dx4;
    fn add(self, other: Point3Dx4) -> Point3Dx4 {
        Point3Dx4 { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
}

impl Add<f32x4> for Point3Dx4 {
    type Output = Point3Dx4;
    fn add(self, scalar: f32x4) -> Point3Dx4 {
        Point3Dx4 { x: self.x + scalar, y: self.y + scalar, z: self.z + scalar }
    }
}

impl Sub for Point3Dx4 {
    type Output = Point3Dx4;
    fn sub(self, other: Point3Dx4) -> Point3Dx4 {
        Point3Dx4 { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

impl Mul<f32x4> for Point3Dx4 {
    type Output = Point3Dx4;
    fn mul(self, scalar: f32x4) -> Point3Dx4 {
        Point3Dx4 { x: self.x * scalar, y: self.y * scalar, z: self.z * scalar }
    }
}

impl Div<f32x4> for Point3Dx4 {
    type Output = Point3Dx4;
    fn div(self, scalar: f32x4) -> Point3Dx4 {
        Point3Dx4 { x: self.x / scalar, y: self.y / scalar, z: self.z / scalar }
    }
}

impl Div<Point3Dx4> for f32x4 {
    type Output = Point3Dx4;
    fn div(self, rhs: Point3Dx4) -> Point3Dx4 {
        Point3Dx4 { x: self / rhs.x, y: self / rhs.y, z: self / rhs.z }
    }
}

#[inline(always)]
pub fn dot3(a: Point3D, b: Point3D) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[inline(always)]
pub fn dot3_simd(a: Point3Dx4, b: Point3Dx4) -> f32x4 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[inline(always)]
pub fn normalize(vec: Point3D) -> Point3D {
    let length = dot3(vec, vec).sqrt();
    if length != 0.0 { vec / length } else { vec }
}

#[inline(always)]
pub fn normalize_simd(vec: Point3Dx4) -> Point3Dx4 {
    let length = dot3_simd(vec, vec).sqrt();
    
    let mask = length.simd_ne(f32x4::splat(0.0));

    // Safe reciprocal: if length != 0 use 1/length else 1.0
    let inv_length = mask.select(length.recip(), f32x4::splat(1.0));

    Point3Dx4 {
        x: vec.x * inv_length,
        y: vec.y * inv_length,
        z: vec.z * inv_length,
    }
}
