// External crates
use raylib::prelude::*;
use rayon::prelude::*;
use plotters::prelude::*;
use plotters::style::Color;

// STD library
use std::time::Instant;
use std::path::Path;
// Internal modules
mod point2d;
mod point3d;
mod triangle;
mod screen;
mod transform;
mod texture;
mod geometry;
mod obj;
mod rectangle;
mod camera;

// Internal imports
use crate::rectangle::compute_subdivisions;
use crate::screen::ScreenSpace;
use crate::geometry::{draw_rectangles, vertex_to_screen, inv_triangle_area, point_in_triangle, subdivide};
use crate::triangle::Triangle3D;
use crate::point2d::Point2D;
use crate::point3d::{Point3D, dot3};
use crate::camera::Camera;


fn depth_to_u8(depth: f32) -> u8 {
        if depth <= 0.0 {
                return 255
        }
        let y = 255.0 * ((-depth / 10.0) + 1.0).exp();
        y.round().clamp(0.0, 255.0) as u8
}

fn shade_pixel(r: u8, g: u8, b: u8, a: u8, normal: point3d::Point3D, light: point3d::Point3D) -> (u8, u8, u8, u8) {
        let normalized_normal = point3d::normalize(normal); //unit vector
        let normalized_light = point3d::normalize(light);
        let intensity = (point3d::dot3(normalized_normal, normalized_light) + 1.0) * 0.5;
        (((r as f32) * intensity) as u8, ((g as f32) * intensity) as u8, ((b as f32) * intensity) as u8, a)
}

fn main() {
    let cores = num_cpus::get();
    println!("Number of logical CPU cores: {}", cores);
    
    // Build thread pool based on number of cores
    rayon::ThreadPoolBuilder::new()
        .num_threads(cores)
        .build_global()
        .unwrap();

    // Define Render resolution
    let width = 1920;
    let height = 1080;
    let resolution = Point2D { x: width as f32, y: height as f32 };

    // Compute depth to subdivide screen for given cores
    let depth = compute_subdivisions(cores);
    // Additional depth so threads can steal work if there are regions onscreen with less geometry
    let rects = subdivide(width, height,depth+1);
    println!("Rectangle dimensions for threads: {:?}", rects);

    // Create multiple 'sub-screenspaces' for each thread to work and join later
    let mut rect_buffers: Vec<ScreenSpace> = rects
        .iter()
        .map(|rect| {
            ScreenSpace {
                rect: *rect,
                width: rect.width(),
                height: rect.height(),
                rgba: vec![0; (rect.width() * rect.height() * 4) as usize],
                depth: vec![f32::INFINITY; (rect.width() * rect.height()) as usize],
            }
        })
        .collect();
    
    // Visualize screenSpace split
    draw_rectangles(&rects, width, height, "rectangles.png");
    println!("Saved rectangles.png");
    
    // Load .obj file and texture file
    let (positions, texcoords, normals, faces) = obj::parse_obj("socrates.obj").expect(".obj file parsing failed");
    let triangles = obj::fan_triangulate_faces(&faces, &positions, &texcoords, &normals);
    let obj_texture = texture::Texture::load("socrates.png").expect("texture image file parsing failed");

    // Create main screenspace
    let mut screen = screen::ScreenSpace::new(width, height);
    let image = raylib::prelude::Image::gen_image_color(width as i32, height as i32,raylib::prelude::Color::BLACK);

    // Create raylib handle
    let (mut r1, thread) = raylib::init()
        .size(width as i32, height as i32)
        .title("Rusterizer")
        .resizable()
        .build();
    r1.set_target_fps(240);
    let mut texture = r1.load_texture_from_image(&thread, &image).expect("raylib texture loading failed");
    
    // Initial conditions for objects
    let mut transformation = transform::Transform { yaw: 0.0, pitch: 0.0, posistion: point3d::Point3D { x: 0.0, y: 0.0, z: 0.0 } };
    let mut new_yaw: f32 = 90.0_f32.to_radians();
    let new_pitch: f32 = 180.0_f32.to_radians();
    let mut new_posistion = point3d::Point3D { x: 0.0, y: 55.0, z: 300.0 };
    
    // Initial conditions for camera
    let mut cam: Camera = Camera { fov: 30.0_f32.to_radians(), camera_speed: 1.0, mouse_sensitivity: 0.002, transform: transform::Transform { yaw: 0.0, pitch: 0.0, posistion: point3d::Point3D { x: 0.0, y: 0.0, z: 0.0 }} };

    // Vectors to store timing metrics
    let mut transform_times: Vec<f64> = Vec::new();
    let mut triangle_times: Vec<f64> = Vec::new();
    let mut merge_times: Vec<f64> = Vec::new();
    let mut frame_times: Vec<f64> = Vec::new();

    while !r1.window_should_close() {
        if r1.is_key_pressed(raylib::consts::KeyboardKey::KEY_ESCAPE) {
            break;
        }

        cam.camera_update(&r1);
                    
        // Clear buffers each frame
        for thread_buf in &mut rect_buffers {
            thread_buf.clear(0,0,0,255);
        }

        let frame_start = std::time::Instant::now();
        
        new_yaw += 0.01;
        
        let world_height = (cam.fov * 0.5).tan() * 2.0;
        let scaled_inv_world_height = resolution.y / world_height;

        transformation.update_transform(new_yaw, new_pitch, new_posistion);
        
        let screenspacetriangles: Vec<triangle::Triangle3D> = triangles
            .par_iter() // parallel iterator instead of .iter()
            .map(|tri| {

                let sa = vertex_to_screen(tri.a, &transformation, &cam, resolution, scaled_inv_world_height);
                let sb = vertex_to_screen(tri.b, &transformation, &cam, resolution, scaled_inv_world_height);
                let sc = vertex_to_screen(tri.c, &transformation, &cam, resolution, scaled_inv_world_height);

                let min_x = sa.x.min(sb.x).min(sc.x);
                let min_y = sa.y.min(sb.y).min(sc.y);
                let max_x = sa.x.max(sb.x).max(sc.x);
                let max_y = sa.y.max(sb.y).max(sc.y);

                let block_start_x = (min_x.floor() as u32).clamp(0, screen.width - 1);
                let block_start_y = (min_y.floor() as u32).clamp(0, screen.height - 1);
                let block_end_x = (max_x.ceil() as u32).clamp(0, screen.width - 1);
                let block_end_y = (max_y.ceil() as u32).clamp(0, screen.height - 1);

                Triangle3D {
                    a: sa,
                    b: sb,
                    c: sc,
                    ta: tri.ta,
                    tb: tri.tb,
                    tc: tri.tc,
                    na: tri.na,
                    nb: tri.nb,
                    nc: tri.nc,
                    bb_start_x: block_start_x,
                    bb_start_y: block_start_y,
                    bb_end_x: block_end_x,
                    bb_end_y: block_end_y,
                }
            })
            .collect();
        
        let transform_time = frame_start.elapsed();
        let triangle_start = Instant::now();
        
        // Look into alternatives that let us use unsafe buffer access accross threeads since we can guarantee no collisions
        rect_buffers.par_iter_mut().for_each(|rect_s| {
            for tri in screenspacetriangles.iter() {
                let (area, inv_area) = inv_triangle_area(
                    Point2D { x: tri.a.x, y: tri.a.y }, 
                    Point2D { x: tri.b.x, y: tri.b.y }, 
                    Point2D { x: tri.c.x, y: tri.c.y }, 
                );
                // Use pre-computed bounding boxes + bounds of current thread rectangle
                for y in tri.bb_start_y.max(rect_s.rect.min_y)..tri.bb_end_y.min(rect_s.rect.max_y) {
                    for x in tri.bb_start_x.max(rect_s.rect.min_x)..tri.bb_end_x.min(rect_s.rect.max_x) {
                        let p = Point2D {
                            x: x as f32 + 0.5,
                            y: y as f32 + 0.5,
                        };
                        let mut weights: Point3D = Point3D { x: 0.0, y: 0.0, z: 0.0 };

                        if point_in_triangle(
                            Point2D { x: tri.a.x, y: tri.a.y }, 
                            Point2D { x: tri.b.x, y: tri.b.y }, 
                            Point2D { x: tri.c.x, y: tri.c.y },
                            p, 
                            area,
                            inv_area,
                            &mut weights
                        ) {
                            let depths: Point3D = Point3D { x: tri.a.z, y: tri.b.z, z: tri.c.z };
                            let depth: f32 = 1.0 / dot3(depths, weights);
                            
                            if depth > rect_s.get_depth(x-rect_s.rect.min_x, y-rect_s.rect.min_y) {
                                continue;
                            }

                            let texture_coord: Point2D = Point2D { 
                                x: dot3(Point3D { x: tri.ta.x * depths.x, y: tri.tb.x * depths.y, z: tri.tc.x * depths.z }, weights), 
                                y: dot3(Point3D { x: tri.ta.y * depths.x, y: tri.tb.y * depths.y, z: tri.tc.y * depths.z }, weights),
                            } * depth;

                            let normal: Point3D = Point3D { 
                                x: dot3(Point3D { x: tri.na.x * depths.x, y: tri.nb.x * depths.y, z: tri.nc.x * depths.z }, weights), 
                                y: dot3(Point3D { x: tri.na.y * depths.x, y: tri.nb.y * depths.y, z: tri.nc.y * depths.z }, weights),
                                z: dot3(Point3D { x: tri.na.z * depths.x, y: tri.nb.z * depths.y, z: tri.nc.z * depths.z }, weights),
                            } * depth;

                            rect_s.set_depth(x-rect_s.rect.min_x, y-rect_s.rect.min_y, depth);

                            let show_depth: bool = false;
                            if show_depth {
                                let depth_gray: u8 = depth_to_u8(depth);
                                rect_s.set_pixel(x-rect_s.rect.min_x, y-rect_s.rect.min_y, depth_gray, depth_gray, depth_gray, 255);
                            } else {
                                let (r,g,b,a) = obj_texture.sample(texture_coord.x, texture_coord.y);
                                let (r,g,b,a) = shade_pixel(r, g, b, a, normal, transformation.transform_direction(Point3D { x: -1.0, y: 0.0, z: 0.0 }) );
                                rect_s.set_pixel(x-rect_s.rect.min_x, y-rect_s.rect.min_y, r, g, b, a);
                            }
                        }
                    }
                }
            }
        });
        let triangle_time = triangle_start.elapsed();

        // Directly copy each rect into the main screen buffer. This can be removed if we dont use seperate buffers in the part above
        let merge_start = Instant::now();
        for rect_s in &rect_buffers {
            let rect_width = rect_s.rect.max_x - rect_s.rect.min_x;
            let rect_height = rect_s.rect.max_y - rect_s.rect.min_y;

            for y in 0..rect_height {
                let screen_y = rect_s.rect.min_y + y;
                if screen_y >= screen.height {
                    continue;
                }

                let screen_row_start = ((screen_y * screen.width + rect_s.rect.min_x) * 4) as usize;
                let rect_row_start = (y * rect_width * 4) as usize;

                // Determine the end of the row (clamp to screen width)
                let row_end = screen_row_start + (rect_width.min(screen.width - rect_s.rect.min_x) * 4) as usize;

                // Copy the row directly into screen.rgba
                screen.rgba[screen_row_start..row_end]
                    .copy_from_slice(&rect_s.rgba[rect_row_start..rect_row_start + (row_end - screen_row_start)]);
            }
        }
        let merge_time = merge_start.elapsed();

        // Put it in a window!
        let _ = texture.update_texture(&screen.rgba);
        let window_width = r1.get_screen_width();
        let window_height = r1.get_screen_height();
        let frame_time = frame_start.elapsed();

        // Collect timing data
        transform_times.push(transform_time.as_micros() as f64);
        triangle_times.push(triangle_time.as_micros() as f64);
        merge_times.push(merge_time.as_micros() as f64);
        frame_times.push(frame_time.as_micros() as f64);

        let mut d = r1.begin_drawing(&thread);
        d.clear_background(raylib::prelude::Color::BLACK);
        d.draw_texture_pro(
            &texture,
            raylib::prelude::Rectangle { x: 0.0, y: 0.0, width: resolution.x, height: resolution.y},
            raylib::prelude::Rectangle { x: 0.0, y: 0.0, width: window_width as f32, height: window_height as f32 },
            raylib::prelude::Vector2 { x: 0.0, y: 0.0 },
            0.0,
            raylib::prelude::Color::WHITE
        );
        // Perf stats
        d.draw_text(&format!("Transform time: {:.2?}\nTriangle time: {:.2?}\nMerge time: {:.2?}\nFrame time: {:.2?}", transform_time, triangle_time, merge_time, frame_time), 10, 10, 20, raylib::prelude::Color::LIME);
    }
    use std::env;
    let current_dir = env::current_dir().unwrap();
    plot_all_metrics(&transform_times, &triangle_times, &merge_times, &frame_times, &current_dir.join("performance_metrics.png")).unwrap();
}

fn plot_all_metrics(
    transform_times: &Vec<f64>,
    triangle_times: &Vec<f64>,
    merge_times: &Vec<f64>,
    frame_times: &Vec<f64>,
    filename: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if transform_times.is_empty() {
        return Err("No data to plot".into());
    }

    let max_time = [transform_times, triangle_times, merge_times, frame_times]
        .iter()
        .flat_map(|v| v.iter())
        .fold(0.0f64, |acc, &x| acc.max(x));

    let root = BitMapBackend::new(filename.to_str().unwrap(), (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Performance Metrics Over Time", ("sans-serif", 30))
        .margin(50)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..(transform_times.len() as i32), 0.0..max_time)?;

    chart
        .configure_mesh()
        .x_desc("Frame")
        .y_desc("Time (μs)")
        .draw()?;

    let series_data = [
        (transform_times, &BLUE, "Transform"),
        (triangle_times, &RED, "Triangle"),
        (merge_times, &GREEN, "Merge"),
        (frame_times, &MAGENTA, "Frame"),
    ];

    for (times, color, label) in series_data {
        chart
            .draw_series(LineSeries::new(
                times.iter().enumerate().map(|(i, &v)| (i as i32, v)),
                color,
            ))?
            .label(label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], color));
    }

    chart
        .configure_series_labels()
        .background_style(&RGBAColor(255, 255, 255, 0.8))
        .border_style(&BLACK)
        .draw()?;

    println!("Successfully saved {}", filename.display());
    Ok(())
}