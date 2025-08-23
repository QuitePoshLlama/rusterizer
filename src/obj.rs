use std::fs::File;
use std::io::{BufRead, BufReader};
use anyhow::{Result, anyhow};
use crate::point2d::Point2D;
use crate::point3d::Point3D;
use crate::triangle::Triangle3D;

#[derive(Debug)]
pub struct Face {
    pub v_indices: Vec<usize>,
    pub vt_indices: Vec<usize>,
    pub vn_indices: Vec<usize>,
}

pub fn parse_obj(path: &str) -> Result<(Vec<Point3D>, Vec<Point2D>, Vec<Point3D>, Vec<Face>)> {
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

fn parse_face_vertex(s: &str) -> Result<(usize, Option<usize>, Option<usize>)> {
    let parts: Vec<&str> = s.split('/').collect();
    let v = parts.get(0).ok_or_else(|| anyhow!("Missing vertex index"))?.parse::<usize>()? - 1;
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

pub fn fan_triangulate_faces(faces: &[Face], vertices: &[Point3D], texture_coords: &[Point2D], vertex_normals: &[Point3D]) -> Vec<Triangle3D> {
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
