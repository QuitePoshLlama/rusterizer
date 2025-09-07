use image::RgbImage;
use rand::Rng;
use image::Rgb;

use crate::point2d::{Point2D, perp, dot2};
use crate::point3d::Point3D;
use crate::transform::Transform;
use crate::rectangle::Rect;
use crate::camera::Camera;

pub fn signed_triangle_area(t1: Point2D, t2: Point2D, p: Point2D) -> f32 {
    let ap = p - t1;
    let t1t2perp: Point2D = perp(t2 - t1);
    dot2(ap, t1t2perp) / 2.0
}

#[inline(always)]
pub fn point_in_triangle(a: Point2D, b: Point2D, c: Point2D, p: Point2D, area: f32, inv_area: f32, weights: &mut Point3D) -> bool {
    // Fail fast on any step
    let area_ab: f32 = signed_triangle_area(a, b, p);
    if !(area_ab >= 0.0) {return false}
    let area_bc: f32 = signed_triangle_area(b, c, p);
    if !(area_bc >= 0.0) {return false}
    let area_ca: f32 = signed_triangle_area(c, a, p);
    if !(area_ca >= 0.0) {return false}
    // Use pre-computed area/inverse once per triangle
    if !(area > 0.0) {return false}
    // Only compute weights if all checks pass
    weights.x = area_bc * inv_area;
    weights.y = area_ca * inv_area;
    weights.z = area_ab * inv_area;
    true
}

#[inline(always)]
pub fn inv_triangle_area(a: Point2D, b: Point2D, c: Point2D) -> (f32,f32) {
    let area = signed_triangle_area(a, b, c);
    (area, 1.0 / area)
}

#[inline(always)]
pub fn vertex_to_screen(vertex: Point3D, transform: &Transform, camera: &Camera, resolution: Point2D, scaled_inv_world_height: f32) -> Point3D {
    
    let vertex_world: Point3D = transform.to_world_point(vertex);
    let vertex_view: Point3D = camera.transform.to_local_point(vertex_world);
    let z_inverted = 1.0 / vertex_view.z;
    
    let pixels_per_world_unit: f32 = scaled_inv_world_height * z_inverted;

    // Apply scaling and shift to center screen (mul add for perf)
    let screen_x = (vertex_view.x * pixels_per_world_unit).mul_add(1.0, resolution.x * 0.5);
    let screen_y = (vertex_view.y * pixels_per_world_unit).mul_add(1.0, resolution.y * 0.5);
    
    // z-buffer is pre-inverted for performance
    Point3D { x: screen_x, y: screen_y, z: z_inverted }
}

/// Subdivide a rectangle evenly with given depth
pub fn subdivide(width: u32, height: u32, depth: u32) -> Vec<Rect> {
    let mut rects = Vec::new();

    let root = Rect {
        min_x: 0,
        min_y: 0,
        max_x: width,
        max_y: height,
    };

    // Alternate spliting the screen vertically and horizontally
    fn recurse(r: Rect, vertical: bool, depth: u32, rects: &mut Vec<Rect>) {
        if depth == 0 {
            rects.push(r);
            return;
        }

        let w = r.max_x - r.min_x;
        let h = r.max_y - r.min_y;

        if vertical {
            let mid = r.min_x + w / 2;
            let left = Rect { min_x: r.min_x, min_y: r.min_y, max_x: mid, max_y: r.max_y };
            let right = Rect { min_x: mid, min_y: r.min_y, max_x: r.max_x, max_y: r.max_y };
            recurse(left, !vertical, depth - 1, rects);
            recurse(right, !vertical, depth - 1, rects);
        } else {
            let mid = r.min_y + h / 2;
            let top = Rect { min_x: r.min_x, min_y: r.min_y, max_x: r.max_x, max_y: mid };
            let bottom = Rect { min_x: r.min_x, min_y: mid, max_x: r.max_x, max_y: r.max_y };
            recurse(top, !vertical, depth - 1, rects);
            recurse(bottom, !vertical, depth - 1, rects);
        }
    }

    recurse(root, true, depth, &mut rects);
    rects
}

/// Save rectangles to an image file to represent areas of screen rendered by individual threads later (NOT USED IN RENDERING PIPELINE)
pub fn draw_rectangles(rects: &[Rect], width: u32, height: u32, filename: &str) {
    let mut img = RgbImage::new(width, height);
    let mut rng = rand::thread_rng();

    for rect in rects {
        let color = Rgb([
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
        ]);

        for y in rect.min_y..=rect.max_y {
            for x in rect.min_x..=rect.max_x {
                if x < width && y < height {
                    img.put_pixel(x, y, color);
                }
            }
        }
    }

    img.save(filename).expect("Failed to save image");
}
