use std::simd::f32x4;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point2Dx4 {
    pub x: f32x4,
    pub y: f32x4,
}

use std::ops::{Add, Sub, Mul, Div};

impl Add for Point2D {
    type Output = Point2D;
    fn add(self, other: Point2D) -> Point2D {
        Point2D { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Sub for Point2D {
    type Output = Point2D;
    fn sub(self, other: Point2D) -> Point2D {
        Point2D { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Mul<f32> for Point2D {
    type Output = Point2D;
    fn mul(self, scalar: f32) -> Point2D {
        Point2D { x: self.x * scalar, y: self.y * scalar }
    }
}

impl Div<f32> for Point2D {
    type Output = Point2D;
    fn div(self, scalar: f32) -> Point2D {
        Point2D { x: self.x / scalar, y: self.y / scalar }
    }
}

impl Add for Point2Dx4 {
    type Output = Point2Dx4;
    fn add(self, other: Point2Dx4) -> Point2Dx4 {
        Point2Dx4 { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Sub for Point2Dx4 {
    type Output = Point2Dx4;
    fn sub(self, other: Point2Dx4) -> Point2Dx4 {
        Point2Dx4 { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Mul<f32x4> for Point2Dx4 {
    type Output = Point2Dx4;
    fn mul(self, scalar: f32x4) -> Point2Dx4 {
        Point2Dx4 { x: self.x * scalar, y: self.y * scalar }
    }
}

impl Div<f32x4> for Point2Dx4 {
    type Output = Point2Dx4;
    fn div(self, scalar: f32x4) -> Point2Dx4 {
        Point2Dx4 { x: self.x / scalar, y: self.y / scalar }
    }
}

#[inline(always)]
pub fn dot2(a: Point2D, b: Point2D) -> f32 {
    a.x * b.x + a.y * b.y
}

#[inline(always)]
pub fn dot2_simd(a: Point2Dx4, b: Point2Dx4) -> f32x4 {
    (a.x * b.x) + (a.y * b.y)
}

#[inline(always)]
pub fn perp(vec: Point2D) -> Point2D {
    Point2D { x: vec.y, y: -vec.x }
}

#[inline(always)]
pub fn perp_simd(vec: Point2Dx4) -> Point2Dx4 {
    Point2Dx4 { x: vec.y, y: -vec.x }
}