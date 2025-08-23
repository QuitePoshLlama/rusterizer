#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
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

pub fn dot2(a: Point2D, b: Point2D) -> f32 {
    a.x * b.x + a.y * b.y
}

pub fn perp(vec: Point2D) -> Point2D {
    Point2D { x: vec.y, y: -vec.x }
}
