use nalgebra::Vector3;
use std::fs::File;
use std::path::Path;
use std::io::{prelude::*, BufReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge(pub usize, pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle(pub usize, pub usize, pub usize);

pub struct Mesh {
    pub vertices: Vec<Vector3<f64>>,
    pub edges: Vec<Edge>,
    pub triangles: Vec<Triangle>
}

impl Edge {
    pub fn length(&self, mesh: &Mesh) -> f64 {
        let ab = mesh.vertices[self.0]-mesh.vertices[self.1];
        ab.norm() as f64
    }
}

impl Triangle {
    pub fn area(&self, mesh: &Mesh) -> f64 {
        let ab = mesh.vertices[self.0]-mesh.vertices[self.1];
        let ac = mesh.vertices[self.0]-mesh.vertices[self.2];
        let cross = ab.cross(&ac);
        (cross.norm() as f64) / 2.0
    }

    pub fn is_face(&self, edge: &Edge) -> bool {
             if self.0 == edge.0 && self.1 == edge.1 { true }
        else if self.0 == edge.0 && self.2 == edge.1 { true }
        else if self.1 == edge.0 && self.2 == edge.1 { true }
        else if self.0 == edge.1 && self.1 == edge.0 { true }
        else if self.0 == edge.1 && self.2 == edge.0 { true }
        else if self.1 == edge.1 && self.2 == edge.0 { true }
        else { false }
    }
}

impl Mesh {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Mesh, String> {
        let file = File::open(path).map_err(|e| format!("error opening file: {}", e))?;
        
        let mut num_vertices = 0;
        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut triangles = Vec::new();

        let mut lineno = 0;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            if line.chars().nth(0) == Some('#') { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();

            lineno += 1;
            if lineno == 1 {
                if parts[0] != "OFF" { return Err("not an OFF file: missing header".to_string()); }
            } 
            else if lineno == 2 {
                num_vertices = parts[0].parse::<usize>().map_err(|_| "invalid number of vertices provided".to_string())?;
            }
            else if lineno <= num_vertices + 2 {
                let x = parts[0].parse::<f64>().map_err(|_| "invalid 'x' coordinate".to_string())?;
                let y = parts[1].parse::<f64>().map_err(|_| "invalid 'y' coordinate".to_string())?;
                let z = parts[2].parse::<f64>().map_err(|_| "invalid 'z' coordinate".to_string())?;
                vertices.push(Vector3::new(x, y, z));
            } 
            else {
                let c = parts[0].parse::<usize>().map_err(|_| "invalid face".to_string())?;
                if c != 3 { return Err("invalid face: not a triangle".to_string()) }

                let i = parts[1].parse::<usize>().map_err(|_| "invalid face".to_string())?;
                let j = parts[2].parse::<usize>().map_err(|_| "invalid face".to_string())?;
                let k = parts[3].parse::<usize>().map_err(|_| "invalid face".to_string())?;

                if i >= vertices.len() || j >= vertices.len() || k >= vertices.len() { return Err("invalid face: index out of bounds.".to_string()); }
                triangles.push(Triangle(i, j, k));
                edges.push(Edge(i,j));
                edges.push(Edge(i,k));
                edges.push(Edge(j,k));
            }
        }

        Ok(Mesh {
            vertices,
            edges,
            triangles,
        })
    }
}
