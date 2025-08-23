use crate::point2d::{Point2D, perp, dot2};
use crate::point3d::Point3D;
use crate::transform::Transform;

pub fn signed_triangle_area(t1: Point2D, t2: Point2D, p: Point2D) -> f32 {
    let ap = p - t1;
    let t1t2perp: Point2D = perp(t2 - t1);
    dot2(ap, t1t2perp) / 2.0
}

pub fn point_in_triangle(a: Point2D, b: Point2D, c: Point2D, p: Point2D, weights: &mut Point3D) -> bool {
    let area_ab: f32 = signed_triangle_area(a, b, p);
    let area_bc: f32 = signed_triangle_area(b, c, p);
    let area_ca: f32 = signed_triangle_area(c, a, p);
    let in_tri: bool = area_ab >= 0.0 && area_bc >= 0.0 && area_ca >= 0.0;
    let total_area: f32 = area_ab + area_bc + area_ca;
    let inv_area_sum: f32 = 1.0 / total_area;
    weights.x = area_bc * inv_area_sum;
    weights.y = area_ca * inv_area_sum;
    weights.z = area_ab * inv_area_sum;
    in_tri && total_area > 0.0
}

pub fn vertex_to_screen(vertex: Point3D, transform: &Transform, resolution: Point2D, fov: f32) -> Point3D {
    let vertex_world: Point3D = transform.to_world_point(vertex);
    let world_height: f32 = (fov / 2.0).tan() * 2.0;
    let pixels_per_world_unit: f32 = resolution.y / world_height / vertex_world.z;
    let pixel_offset: Point2D = Point2D { x: (vertex_world.x * pixels_per_world_unit), y: (vertex_world.y * pixels_per_world_unit) };
    let vertex_screen: Point2D = resolution / 2.0 + pixel_offset;
    Point3D {x: vertex_screen.x, y: vertex_screen.y, z: vertex_world.z}
}
