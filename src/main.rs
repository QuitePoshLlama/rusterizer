use std::fs::File;
use std::io::{Write, BufWriter};
use std::ops::{Add, Sub, Mul, Div};
use std::time::Instant;
use std::path::Path;
use rand::Rng;
use std::io::{BufRead, BufReader};
use anyhow::{Result, anyhow};
use raylib::prelude::*;
use bytemuck::cast_slice_mut;
use image::{DynamicImage, GenericImageView};

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point2D {
    x: f32,
    y: f32,
}

impl Add for Point2D {
    type Output = Point2D;

    fn add(self, other: Point2D) -> Point2D {
        Point2D {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point2D {
    type Output = Point2D;

    fn sub(self, other: Point2D) -> Point2D {
        Point2D {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// Scalar multiplication
impl Mul<f32> for Point2D {
    type Output = Point2D;

    fn mul(self, scalar: f32) -> Point2D {
        Point2D {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

// Scalar division
impl Div<f32> for Point2D {
    type Output = Point2D;

    fn div(self, scalar: f32) -> Point2D {
        Point2D {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

// Scalar division (inverse)
impl Div<Point3D> for f32 {
    type Output = Point3D;

    fn div(self, rhs: Point3D) -> Point3D {
        Point3D {
            x: self / rhs.x,
            y: self / rhs.y,
            z: self / rhs.z,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

// Vector + Vector
impl Add for Point3D {
    type Output = Point3D;

    fn add(self, other: Point3D) -> Point3D {
        Point3D {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

// Vector + Scalar
impl Add<f32> for Point3D {
    type Output = Point3D;

    fn add(self, scalar: f32) -> Point3D {
        Point3D {
            x: self.x + scalar,
            y: self.y + scalar,
            z: self.z + scalar,
        }
    }
}

impl Sub for Point3D {
    type Output = Point3D;

    fn sub(self, other: Point3D) -> Point3D {
        Point3D {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// Scalar multiplication
impl Mul<f32> for Point3D {
    type Output = Point3D;

    fn mul(self, scalar: f32) -> Point3D {
        Point3D {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

// Scalar division
impl Div<f32> for Point3D {
    type Output = Point3D;

    fn div(self, scalar: f32) -> Point3D {
        Point3D {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

fn dot2(a: Point2D, b: Point2D) -> f32 {
    return a.x * b.x + a.y * b.y;
}

fn dot3(a: Point3D, b: Point3D) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn perp(vec: Point2D) -> Point2D {
    return Point2D { 
        x: vec.y,
        y: - vec.x,
    }
}

fn normalize(vec: Point3D) -> Point3D {
    let length = dot3(vec, vec).sqrt();
    if length != 0.0 {
        vec / length
    } else {
        vec
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Triangle3D {
    // vertexes
    a: Point3D,
    b: Point3D,
    c: Point3D,

    // texture coords
    ta: Point2D,
    tb: Point2D,
    tc: Point2D,

    // vertex normals
    na: Point3D,
    nb: Point3D,
    nc: Point3D,
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Triangle2D {
    a: Point2D,
    b: Point2D,
}

#[derive(Debug)]
struct ScreenSpace {
    width: u32,     // in pixels
    height: u32,
    size: usize,
    rgba: Vec<u8>,     // 0-255 values for 8-bit color depth + alpha channel
    depth: Vec<f32>, //depth buffer
}

impl ScreenSpace {
    fn new(width: u32, height: u32) -> Self {
        let size_calc = (width * height) as usize;
        Self {
            width,
            height,
            size: size_calc,
            rgba: vec![0; size_calc * 4],
            depth: vec![f32::INFINITY; size_calc],
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, red: u8, green: u8, blue: u8, alpha: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        let i = ((y * self.width + x) * 4) as usize;
        self.rgba[i] = red;
        self.rgba[i + 1] = green;
        self.rgba[i + 2] = blue;
        self.rgba[i + 3] = alpha;
    }
    
    fn get_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None
        }
        let i = ((y * self.width + x) * 4) as usize;
        Some((
            self.rgba[i],     // R
            self.rgba[i + 1], // G
            self.rgba[i + 2], // B
            self.rgba[i + 3], // Alpha
        ))
    }

    fn set_depth(&mut self, x: u32, y: u32, value: f32) {
        let i = (y * self.width + x) as usize;
        self.depth[i] = value;
    }

    fn get_depth(&self, x: u32, y: u32) -> f32 {
        let i = (y * self.width + x) as usize;
        self.depth[i]
    }

    fn clear(&mut self, r: u8, g: u8, b: u8, a: u8) {
        let color: u32 = u32::from_le_bytes([r, g, b, a]); // RGBA as 0xAABBGGRR
        let buf_as_u32: &mut [u32] = cast_slice_mut(&mut self.rgba);
        buf_as_u32.fill(color);
        self.depth = vec![f32::INFINITY; self.size];
    }
        
    pub fn write_bmp(&self, path: &str) -> Result<()> {
        let width: u32 = self.width;
        let height: u32 = self.height;
        let row_stride: u32 = (3 * width + 3) & !3; // BMP row alignment: pad to 4-byte multiple
        let pixel_array_size: u32 = row_stride * height;
        let file_size: u32 = 54 + pixel_array_size;

        let mut file = BufWriter::new(File::create(path)?);

        // === BMP HEADER ===
        file.write_all(b"BM")?;                                // Signature
        file.write_all(&(file_size as u32).to_le_bytes())?;    // File size
        file.write_all(&[0u8; 4])?;                            // Reserved
        file.write_all(&54u32.to_le_bytes())?;                 // Pixel data offset

        // === DIB HEADER (BITMAPINFOHEADER) ===
        file.write_all(&[40u8, 0, 0, 0])?;                     // Header size
        file.write_all(&(width as i32).to_le_bytes())?;        // Width
        file.write_all(&(height as i32).to_le_bytes())?;       // Height
        file.write_all(&[1, 0])?;                              // Color planes
        file.write_all(&[24, 0])?;                             // Bits per pixel
        file.write_all(&[0u8; 4])?;                            // Compression (none)
        file.write_all(&(pixel_array_size as u32).to_le_bytes())?; // Raw bitmap size
        file.write_all(&[0u8; 4])?;                            // Print resolution X
        file.write_all(&[0u8; 4])?;                            // Print resolution Y
        file.write_all(&[0u8; 4])?;                            // Palette colors
        file.write_all(&[0u8; 4])?;                            // Important colors

        // === Pixel Data (bottom-up BGR) ===
        let padding = vec![0u8; (row_stride - width * 3) as usize];
        for y in (0..height).rev() { // BMP is bottom-up
            for x in 0..width {
                let i = ((y * width + x) * 4) as usize;
                let r = self.rgba[i];
                let g = self.rgba[i + 1];
                let b = self.rgba[i + 2];
                file.write_all(&[b, g, r])?; // BMP is BGR
            }
            file.write_all(&padding)?; // Row padding
        }

        Ok(())
    }
}

fn signed_triangle_area(t1: Point2D, t2: Point2D, p: Point2D) -> f32 {
    let ap = p - t1;
    let t1t2perp: Point2D = perp(t2 - t1);
    return dot2(ap, t1t2perp) / 2.0;
}

fn point_in_triangle(a: Point2D, b: Point2D, c: Point2D, p: Point2D, weights: &mut Point3D) -> bool {
    let area_ab: f32 = signed_triangle_area(a, b, p);
    let area_bc: f32 = signed_triangle_area(b, c, p);
    let area_ca: f32 = signed_triangle_area(c, a, p);
    let in_tri: bool = area_ab >= 0.0 && area_bc >= 0.0 && area_ca >= 0.0;

    let total_area: f32 = area_ab + area_bc + area_ca;
    let inv_area_sum: f32 = 1.0 / total_area;
    weights.x = area_bc * inv_area_sum;
    weights.y = area_ca * inv_area_sum;
    weights.z = area_ab * inv_area_sum;

    return in_tri && total_area > 0.0;
}

fn vertex_to_screen(vertex: Point3D, transform: &Transform, resolution: Point2D, fov: f32) -> Point3D {
    
    let vertex_world: Point3D = transform.to_world_point(vertex);

    let world_height: f32 = (fov / 2.0).tan() * 2.0;

    let pixels_per_world_unit: f32 = resolution.y / world_height / vertex_world.z;
    let pixel_offset: Point2D = Point2D { x: (vertex_world.x * pixels_per_world_unit), y: (vertex_world.y * pixels_per_world_unit) };
    let vertex_screen: Point2D = resolution / 2.0 + pixel_offset;
    return Point3D {x: vertex_screen.x, y: vertex_screen.y, z: vertex_world.z};
}

#[derive(Debug)]
struct Face {
    v_indices: Vec<usize>,
    vt_indices: Vec<usize>,
    vn_indices: Vec<usize>,
}

fn parse_obj(path: &str) -> Result<(Vec<Point3D>, Vec<Point2D>, Vec<Point3D>, Vec<Face>)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut positions: Vec<Point3D> = Vec::new();
    let mut texcoords: Vec<Point2D> = Vec::new();
    let mut normals: Vec<Point3D> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() || tokens[0].starts_with('#') {
            continue
        }

        match tokens[0] {
            "v" => {
                let x = tokens[1].parse()?;
                let y = tokens[2].parse()?;
                let z = tokens[3].parse()?;
                positions.push(Point3D { x, y, z })
            }
            "vt" => {
                let u = tokens[1].parse()?;
                let v = tokens[2].parse()?;
                texcoords.push(Point2D { x: u, y: v })
            }
            "vn" => {
                let x = tokens[1].parse()?;
                let y = tokens[2].parse()?;
                let z = tokens[3].parse()?;
                normals.push(Point3D { x, y, z })
            }
            "f" => {
                let mut face_v_indices = Vec::new();
                let mut face_vt_indices = Vec::new();
                let mut face_vn_indices = Vec::new();

                for part in &tokens[1..] {
                    let (v_index, vt_index, vn_index) = parse_face_vertex(part)?;
                    face_v_indices.push(v_index);
                    face_vt_indices.push(vt_index.unwrap_or(0));
                    face_vn_indices.push(vn_index.unwrap_or(0));
                }
                faces.push(Face { v_indices: face_v_indices, vt_indices: face_vt_indices, vn_indices: face_vn_indices })
            }
            _ => {}
        }
    }

    Ok((positions, texcoords, normals, faces))
}

// Parses f v, f v/vt, f v//vn, f v/vt/vn
fn parse_face_vertex(s: &str) -> Result<(usize, Option<usize>, Option<usize>)> {
    let parts: Vec<&str> = s.split('/').collect();

    let v = parts.get(0).ok_or_else(|| anyhow!("Missing vertex index"))?
        .parse::<usize>()? - 1;

    let vt = match parts.get(1) {
        Some(&"") | None => None,
        Some(s) => Some(s.parse::<usize>()? - 1),
    };

    let vn = match parts.get(2) {
        None => None,
        Some(&"") => None,
        Some(s) => Some(s.parse::<usize>()? - 1),
    };

    Ok((v, vt, vn))
}

fn fan_triangulate_faces(faces: &[Face], vertices: &[Point3D], texture_coords: &[Point2D], vertex_normals: &[Point3D]) -> Vec<Triangle3D> {
    let mut triangles: Vec<Triangle3D> = Vec::new();

    for face in faces {
        let v_indices: &Vec<usize> = &face.v_indices;
        let vt_indices: &Vec<usize> = &face.vt_indices;
        let vn_indices: &Vec<usize> = &face.vn_indices;
        if v_indices.len() < 3 {
            continue // skip faces already triangled
        }

        for i in 1..v_indices.len() - 1 {
            let a: Point3D = vertices[v_indices[0]];
            let b: Point3D = vertices[v_indices[i]];
            let c: Point3D = vertices[v_indices[i + 1]];

            let ta: Point2D = texture_coords[vt_indices[0]];
            let tb: Point2D = texture_coords[vt_indices[i]];
            let tc: Point2D = texture_coords[vt_indices[i+1]];

            let na: Point3D = vertex_normals[vn_indices[0]];
            let nb: Point3D = vertex_normals[vn_indices[i]];
            let nc: Point3D = vertex_normals[vn_indices[i+1]];

            triangles.push(Triangle3D { a, b, c, ta, tb, tc, na, nb, nc });
        }
    }

    return triangles
}

/*This struct / implementation will give us methods to generate basis vectors with
which to take a given point and transform its posistion in world space
*/

fn transform_vector(ihat: Point3D, jhat: Point3D, khat: Point3D, v: Point3D) -> Point3D {
    return ihat * v.x + jhat * v.y + khat * v.z
}
struct Transform {
    yaw: f32,
    pitch: f32,
    posistion: Point3D,
}

impl Transform {
    fn update_transform(&mut self, new_yaw: f32, new_pitch: f32, new_position: Point3D) {
        self.yaw = new_yaw;
        self.pitch = new_pitch;
        self.posistion = new_position;
    }

    fn get_basis_vectors(&self) -> (Point3D, Point3D, Point3D) {
        let ihat_yaw: Point3D = Point3D { x: self.yaw.cos(), y: 0.0, z: self.yaw.sin() };
        let jhat_yaw: Point3D = Point3D { x: 0.0, y: 1.0, z: 0.0 };
        let khat_yaw: Point3D = Point3D { x: -self.yaw.sin(), y: 0.0, z: self.yaw.cos() };

        let ihat_pitch: Point3D = Point3D { x: 1.0, y: 0.0, z: 0.0 };
        let jhat_pitch: Point3D = Point3D { x: 0.0, y: self.pitch.cos(), z: -self.pitch.sin() };
        let khat_pitch: Point3D = Point3D { x: 0.0, y: self.pitch.sin(), z: self.pitch.cos() };

        let ihat: Point3D = transform_vector(ihat_yaw, jhat_yaw, khat_yaw, ihat_pitch);
        let jhat: Point3D = transform_vector(ihat_yaw, jhat_yaw, khat_yaw, jhat_pitch);
        let khat: Point3D = transform_vector(ihat_yaw, jhat_yaw, khat_yaw, khat_pitch);

        return (ihat, jhat, khat)
    }

    fn to_world_point(&self, point: Point3D) -> Point3D {
        let (ihat, jhat, khat) = self.get_basis_vectors();
        return transform_vector(ihat, jhat, khat, point) + self.posistion;
    }

    /// Transform a direction vector (e.g., normal) without applying position
    fn transform_direction(&self, dir: Point3D) -> Point3D {
        let (ihat, jhat, khat) = self.get_basis_vectors();
        transform_vector(ihat, jhat, khat, dir)
    }
}

fn depth_to_u8(depth: f32) -> u8 {
    if depth <= 0.0 {
        return 255
    }

    let y = 255.0 * ((-depth / 10.0) + 1.0).exp();
    y.round().clamp(0.0, 255.0) as u8
}

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>, // interleaved RGBA (8-bit per channel)
}

impl Texture {
    /// Load an image into raw RGBA8 buffer
    pub fn load<P: AsRef<Path>>(path: P) -> image::ImageResult<Self> {
        let img: DynamicImage = image::open(path)?;
        let (width, height) = img.dimensions();
        let rgba_img = img.to_rgba8();
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);

        // Flip vertically
        for y in (0..height).rev() {
            let row_start = (y * width * 4) as usize;
            let row_end = row_start + (width * 4) as usize;
            rgba.extend_from_slice(&rgba_img.as_raw()[row_start..row_end]);
        }

        Ok(Self { width, height, rgba })
    }

    /// Sample using normalized coords (0.0–1.0).
    /// Returns (r, g, b, a) as 4 u8 values.
    pub fn sample(&self, u: f32, v: f32) -> (u8, u8, u8, u8) {
        let u = u.fract();
        let v = v.fract();

        let x = (u * (self.width as f32 - 1.0)).round() as u32;
        let y = (v * (self.height as f32 - 1.0)).round() as u32;
        let idx = ((y * self.width + x) * 4) as usize;

        (
            self.rgba[idx],
            self.rgba[idx + 1],
            self.rgba[idx + 2],
            self.rgba[idx + 3],
        )
    }
}

fn shade_pixel(r: u8, g: u8, b: u8, a: u8, normal: Point3D, light: Point3D) -> (u8, u8, u8, u8) {
    let normalized_normal = normalize(normal); //unit vector
    let normalized_light = normalize(light);
    let intensity = (dot3(normalized_normal, normalized_light) + 1.0) * 0.5;
    return (((r as f32) * intensity) as u8, ((g as f32) * intensity) as u8, ((b as f32) * intensity) as u8, a)
}

fn main() {
    // Multithread this shit later :D
    let cores = num_cpus::get();
    println!("Number of logical CPU cores: {}", cores);

/* 
    let _triangle2 = Triangle3D {
        a: Point3D { x: -5.0, y: -5.0, z: 0.0},
        b: Point3D { x: 5.0, y: -5.0, z: 0.0},
        c: Point3D { x: -5.0, y: 5.0, z: 0.0},
    };
    let _triangle1= Triangle3D {
        a: Point3D { x: 5.0, y: 5.0, z: 0.0},
        b: Point3D { x: 5.0, y: -5.0, z: 0.0},
        c: Point3D { x: -5.0, y: 5.0, z: 0.0},
    };
*/

    let (positions, texcoords, normals, faces) = parse_obj("socrates.obj").expect(".obj file parsing failed");
    // println!("{obj:#?}");
    //println!("Loaded obj file");

    let triangles = fan_triangulate_faces(&faces, &positions, &texcoords, &normals);
    //println!("Triangulated faces");

    let obj_texture: Texture = Texture::load("socrates.png").expect("texture image file parsing failed");

    let mut rng = rand::thread_rng();
    let mut triangle_colors: Vec<(u8,u8,u8)> = vec![(0,0,0); triangles.len()];
    
    for triangle_color in &mut triangle_colors {
        *triangle_color = (rng.r#gen(), rng.r#gen(), rng.r#gen());
    }
    
    let resolution = Point2D {x: 1920.0, y: 1080.0};
    let mut screen = ScreenSpace::new(resolution.x as u32, resolution.y as u32);
    
    let mut image = raylib::prelude::Image::gen_image_color(resolution.x as i32,resolution.y as i32,Color::BLACK);

    let (mut r1, thread) = raylib::init()
        .size(resolution.x as i32, resolution.y as i32)
        .title("Rusterizer")
        .resizable()
        .build();

    r1.set_target_fps(240);

    let mut texture = r1.load_texture_from_image(&thread, &image).expect("raylib texture loading failed");

    let fov: f32 = 30.0_f32.to_radians();

    let mut transformation: Transform = Transform { yaw: 0.0, pitch: 0.0, posistion: Point3D { x: 0.0, y: 0.0, z: 0.0 } };

    let mut new_yaw: f32 = 90.0_f32.to_radians();
    let mut new_pitch: f32 = 180.0_f32.to_radians();
    let mut new_posistion: Point3D = Point3D { x: 0.0, y: 55.0, z: 300.0 };

    while !r1.window_should_close() {

        screen.clear(0,0,0,255);

        let frame_start = Instant::now();
        
        new_yaw = new_yaw + 0.01;
        //new_pitch = new_pitch + 0.001;

        transformation.update_transform(new_yaw, new_pitch, new_posistion);

        let screenspacetriangles: Vec<Triangle3D> = triangles
            .iter()
            .map(|tri| Triangle3D {
                a: vertex_to_screen(tri.a, &transformation, resolution, fov),
                b: vertex_to_screen(tri.b, &transformation, resolution, fov),
                c: vertex_to_screen(tri.c, &transformation, resolution, fov),
                ta: tri.ta,
                tb: tri.tb,
                tc: tri.tc,
                na: tri.na,
                nb: tri.nb,
                nc: tri.nc,
            })
            .collect();

        //println!("Converted to screenspace");

        // Loop over all pixels and check if inside triangle
        for (index, tri) in screenspacetriangles.iter().enumerate() {
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
                        &mut weights
                    ) {
                        let depths: Point3D = Point3D { x: tri.a.z, y: tri.b.z, z: tri.c.z };
                        let depth: f32 = 1.0 / dot3(1.0 / depths, weights);
                        
                        if depth > screen.get_depth(x, y) {
                            continue;
                        }

                        let texture_coord: Point2D = Point2D { 
                            x: dot3(Point3D { x: tri.ta.x / depths.x, y: tri.tb.x / depths.y, z: tri.tc.x / depths.z }, weights), 
                            y: dot3(Point3D { x: tri.ta.y / depths.x, y: tri.tb.y / depths.y, z: tri.tc.y / depths.z }, weights),
                        } * depth;

                        let normal: Point3D = Point3D { 
                            x: dot3(Point3D { x: tri.na.x / depths.x, y: tri.nb.x / depths.y, z: tri.nc.x / depths.z }, weights), 
                            y: dot3(Point3D { x: tri.na.y / depths.x, y: tri.nb.y / depths.y, z: tri.nc.y / depths.z }, weights),
                            z: dot3(Point3D { x: tri.na.z / depths.x, y: tri.nb.z / depths.y, z: tri.nc.z / depths.z }, weights),
                        } * depth;

                        // let (r,g,b): &(u8, u8, u8) = &triangle_colors[index];

                        screen.set_depth(x, y, depth);

                        let show_depth: bool = false;
                        if show_depth {
                            let depth_gray: u8 = depth_to_u8(depth);
                            screen.set_pixel(x, y, depth_gray, depth_gray, depth_gray, 255);
                        } else {
                            let (r,g,b,a) = obj_texture.sample(texture_coord.x, texture_coord.y);
                            let (r,g,b,a) = shade_pixel(r, g, b, a, normal, transformation.transform_direction(Point3D { x: -1.0, y: 0.0, z: 0.0 }) );
                            screen.set_pixel(x, y, r, g, b, a);
                        }
                        
                        
                    }
                }
            }
        }

        //println!("Drew pixels");

        let result = texture.update_texture(&screen.rgba);
        //println!("{result:?}");
        let window_width = r1.get_screen_width();
        let window_height = r1.get_screen_height();
        
        let frame_time = frame_start.elapsed();

        let mut d = r1.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        // Draw the texture scaled to the window
        d.draw_texture_pro(
            &texture,
            Rectangle { x: 0.0, y: 0.0, width: resolution.x as f32, height: resolution.y as f32 },
            Rectangle { x: 0.0, y: 0.0, width: window_width as f32, height: window_height as f32 },
            Vector2 { x: 0.0, y: 0.0 },
            0.0,
            Color::WHITE
        );
        d.draw_text(&format!("Frame time: {:.2?}", frame_time), 10, 10, 20, Color::LIME);

        //let _ = screen.write_bmp("yes.bmp");ß
        //println!("Frame time: {:.2?}", frame_time);
    }
}
 