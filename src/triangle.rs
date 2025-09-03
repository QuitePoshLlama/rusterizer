use crate::point2d::Point2D;
use crate::point3d::Point3D;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle3D {
    // vertices
    pub a: Point3D,
    pub b: Point3D,
    pub c: Point3D,
    // texture coordinates
    pub ta: Point2D,
    pub tb: Point2D,
    pub tc: Point2D,
    // normals
    pub na: Point3D,
    pub nb: Point3D,
    pub nc: Point3D,
    // screenspace bounding boxes
    pub bb_start_x: u32,
    pub bb_start_y: u32,
    pub bb_end_x: u32,
    pub bb_end_y: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle2D {
    pub a: Point2D,
    pub b: Point2D,
}
