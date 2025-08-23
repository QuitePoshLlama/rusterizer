// External crates
use rand::Rng;
use num_cpus;
use raylib::prelude::*;

// Internal modules
mod point2d;
mod point3d;
mod triangle;
mod screen;
mod transform;
mod texture;
mod geometry;
mod obj;

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

        let (positions, texcoords, normals, faces) = obj::parse_obj("socrates.obj").expect(".obj file parsing failed");
        let triangles = obj::fan_triangulate_faces(&faces, &positions, &texcoords, &normals);
        let obj_texture = texture::Texture::load("socrates.png").expect("texture image file parsing failed");

        let mut rng = rand::thread_rng();
        let mut triangle_colors: Vec<(u8,u8,u8)> = vec![(0,0,0); triangles.len()];
                        for triangle_color in &mut triangle_colors {
                                *triangle_color = (rng.r#gen::<u8>(), rng.r#gen::<u8>(), rng.r#gen::<u8>());
                        }

        let resolution = point2d::Point2D {x: 1920.0, y: 1080.0};
        let mut screen = screen::ScreenSpace::new(resolution.x as u32, resolution.y as u32);
        let image = raylib::prelude::Image::gen_image_color(resolution.x as i32,resolution.y as i32,raylib::prelude::Color::BLACK);

        let (mut r1, thread) = raylib::init()
                .size(resolution.x as i32, resolution.y as i32)
                .title("Rusterizer")
                .resizable()
                .build();
        r1.set_target_fps(240);
        let mut texture = r1.load_texture_from_image(&thread, &image).expect("raylib texture loading failed");
        let fov: f32 = 30.0_f32.to_radians();
        let mut transformation = transform::Transform { yaw: 0.0, pitch: 0.0, posistion: point3d::Point3D { x: 0.0, y: 0.0, z: 0.0 } };
        let mut new_yaw: f32 = 90.0_f32.to_radians();
        let new_pitch: f32 = 180.0_f32.to_radians();
        let new_posistion = point3d::Point3D { x: 0.0, y: 55.0, z: 300.0 };

        while !r1.window_should_close() {
                screen.clear(0,0,0,255);
                let frame_start = std::time::Instant::now();
                new_yaw = new_yaw + 0.01;
                transformation.update_transform(new_yaw, new_pitch, new_posistion);
                let screenspacetriangles: Vec<triangle::Triangle3D> = triangles
                        .iter()
                        .map(|tri| triangle::Triangle3D {
                                a: geometry::vertex_to_screen(tri.a, &transformation, resolution, fov),
                                b: geometry::vertex_to_screen(tri.b, &transformation, resolution, fov),
                                c: geometry::vertex_to_screen(tri.c, &transformation, resolution, fov),
                                ta: tri.ta,
                                tb: tri.tb,
                                tc: tri.tc,
                                na: tri.na,
                                nb: tri.nb,
                                nc: tri.nc,
                        })
                        .collect();

                for tri in screenspacetriangles.iter() {
                        let min_x = tri.a.x.min(tri.b.x).min(tri.c.x);
                        let min_y = tri.a.y.min(tri.b.y).min(tri.c.y);
                        let max_x = tri.a.x.max(tri.b.x).max(tri.c.x);
                        let max_y = tri.a.y.max(tri.b.y).max(tri.c.y);
                        let block_start_x = (min_x.floor() as u32).clamp(0, screen.width as u32 - 1);
                        let block_start_y = (min_y.floor() as u32).clamp(0, screen.height as u32 - 1);
                        let block_end_x = (max_x.ceil() as u32).clamp(0, screen.width as u32 - 1);
                        let block_end_y = (max_y.ceil() as u32).clamp(0, screen.height as u32 - 1);
                        for y in block_start_y..block_end_y {
                                for x in block_start_x..block_end_x {
                                        let p = point2d::Point2D {
                                                x: x as f32 + 0.5,
                                                y: y as f32 + 0.5,
                                        };
                                        let mut weights = point3d::Point3D { x: 0.0, y: 0.0, z: 0.0 };
                                        if geometry::point_in_triangle(
                                                point2d::Point2D { x: tri.a.x, y: tri.a.y }, 
                                                point2d::Point2D { x: tri.b.x, y: tri.b.y }, 
                                                point2d::Point2D { x: tri.c.x, y: tri.c.y }, 
                                                p, 
                                                &mut weights
                                        ) {
                                                let depths = point3d::Point3D { x: tri.a.z, y: tri.b.z, z: tri.c.z };
                                                let depth: f32 = 1.0 / point3d::dot3(1.0 / depths, weights);
                                                if depth > screen.get_depth(x, y) {
                                                        continue;
                                                }
                                                let texture_coord = point2d::Point2D { 
                                                        x: point3d::dot3(point3d::Point3D { x: tri.ta.x / depths.x, y: tri.tb.x / depths.y, z: tri.tc.x / depths.z }, weights), 
                                                        y: point3d::dot3(point3d::Point3D { x: tri.ta.y / depths.x, y: tri.tb.y / depths.y, z: tri.tc.y / depths.z }, weights),
                                                } * depth;
                                                let normal = point3d::Point3D { 
                                                        x: point3d::dot3(point3d::Point3D { x: tri.na.x / depths.x, y: tri.nb.x / depths.y, z: tri.nc.x / depths.z }, weights), 
                                                        y: point3d::dot3(point3d::Point3D { x: tri.na.y / depths.x, y: tri.nb.y / depths.y, z: tri.nc.y / depths.z }, weights),
                                                        z: point3d::dot3(point3d::Point3D { x: tri.na.z / depths.x, y: tri.nb.z / depths.y, z: tri.nc.z / depths.z }, weights),
                                                } * depth;
                                                screen.set_depth(x, y, depth);
                                                let show_depth: bool = false;
                                                if show_depth {
                                                        let depth_gray: u8 = depth_to_u8(depth);
                                                        screen.set_pixel(x, y, depth_gray, depth_gray, depth_gray, 255);
                                                } else {
                                                        let (r,g,b,a) = obj_texture.sample(texture_coord.x, texture_coord.y);
                                                        let (r,g,b,a) = shade_pixel(r, g, b, a, normal, transformation.transform_direction(point3d::Point3D { x: -1.0, y: 0.0, z: 0.0 }) );
                                                        screen.set_pixel(x, y, r, g, b, a);
                                                }
                                        }
                                }
                        }
                }

                let _ = texture.update_texture(&screen.rgba);
                let window_width = r1.get_screen_width();
                let window_height = r1.get_screen_height();
                let frame_time = frame_start.elapsed();
                let mut d = r1.begin_drawing(&thread);
                d.clear_background(raylib::prelude::Color::BLACK);
                d.draw_texture_pro(
                        &texture,
                        raylib::prelude::Rectangle { x: 0.0, y: 0.0, width: resolution.x as f32, height: resolution.y as f32 },
                        raylib::prelude::Rectangle { x: 0.0, y: 0.0, width: window_width as f32, height: window_height as f32 },
                        raylib::prelude::Vector2 { x: 0.0, y: 0.0 },
                        0.0,
                        raylib::prelude::Color::WHITE
                );
                d.draw_text(&format!("Frame time: {:.2?}", frame_time), 10, 10, 20, raylib::prelude::Color::LIME);
                //let _ = screen.write_bmp("yes.bmp");
        }
}