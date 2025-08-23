use crate::point2d::Point2D;
use crate::point3d::Point3D;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle3D {
    pub a: Point3D,
    pub b: Point3D,
    pub c: Point3D,
    pub ta: Point2D,
    pub tb: Point2D,
    pub tc: Point2D,
    pub na: Point3D,
    pub nb: Point3D,
    pub nc: Point3D,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle2D {
    pub a: Point2D,
    pub b: Point2D,
}
