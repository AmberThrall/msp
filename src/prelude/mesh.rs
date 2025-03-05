use nalgebra::Vector3;
use std::fs::File;
use std::path::Path;
use std::io::{prelude::*, BufReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Even,
    Odd
}

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

    pub fn swap_orientation(&mut self) {
        let tmp = self.0;
        self.0 = self.1;
        self.1 = tmp;
    }

    pub fn orient(&mut self, orientation: Orientation) {
        if self.orientation() != orientation { 
            self.swap_orientation();
        }
    }

    pub fn induce_orientation(&mut self, tri: &Triangle) {
        let a = self.0;
        let b = self.1;
        if tri.0 == a && tri.1 == b { self.0 = a; self.1 = b; }
        if tri.0 == a && tri.2 == b { self.0 = b; self.1 = a; }
        if tri.1 == a && tri.2 == b { self.0 = a; self.1 = b; }
        if tri.0 == b && tri.1 == a { self.0 = b; self.1 = a; }
        if tri.0 == b && tri.2 == a { self.0 = a; self.1 = b; }
        if tri.1 == b && tri.2 == a { self.0 = b; self.1 = a; }
    }

    pub fn orientation(&self) -> Orientation {
        if self.0 < self.1 { Orientation::Even }
        else { Orientation::Odd }
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

    pub fn swap_orientation(&mut self) {
        let tmp = self.0;
        self.0 = self.1;
        self.1 = tmp;
    }

    pub fn orient(&mut self, orientation: Orientation) {
        if self.orientation() != orientation { 
            self.swap_orientation();    
        }
    }

    pub fn orientation(&self) -> Orientation {
             if self.0 < self.1 && self.1 < self.2 { Orientation::Even }
        else if self.0 < self.2 && self.2 < self.1 { Orientation::Odd  }
        else if self.1 < self.0 && self.0 < self.2 { Orientation::Odd  }
        else if self.1 < self.2 && self.2 < self.0 { Orientation::Even }
        else if self.2 < self.1 && self.1 < self.0 { Orientation::Odd  }
        else if self.2 < self.0 && self.0 < self.1 { Orientation::Even }
        else { Orientation::Odd }
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
                if !edges.contains(&Edge(i,j)) && !edges.contains(&Edge(j,i)) { edges.push(Edge(i,j)); }
                if !edges.contains(&Edge(i,k)) && !edges.contains(&Edge(k,i)) { edges.push(Edge(i,k)); }
                if !edges.contains(&Edge(j,k)) && !edges.contains(&Edge(k,j)) { edges.push(Edge(j,k)); }
            }
        }

        Ok(Mesh {
            vertices,
            edges,
            triangles,
        })
    }

    pub fn orient(&mut self) -> Result<(), String> {
        // 1. Set the orientation of a random triangle and induce its oriention onto its faces.
        //    Then get its neighbors.
        let mut visited_edges = vec![false; self.edges.len()];
        let mut visited_tris = vec![false; self.triangles.len()];
        self.triangles[0].orient(Orientation::Even);
        visited_tris[0] = true;
        self._orient_edges(0, &mut visited_edges);
        let mut nbhrs = self._nbhrs(0); 

        // 2. For each neighbor:
        //   a. Set the orientation such that the induced oriention on its faces is opposite their
        //      currrent orientation.
        //   b. Induce orientation onto non-oriented faces.
        //   c. Add each unoriented neighbor to the neighbors list.
        while let Some(nbhr) = nbhrs.pop() {
            let mut orientations = Vec::new();
            visited_tris[nbhr] = true;

            // Determine the needed orientations
            let edges = self._edges(nbhr);
            for i in 0..3 {
                if !visited_edges[edges[i]] { continue; }

                let edge_o = self.edges[edges[i]].orientation();
                let mut edge_clone = self.edges[edges[i]].clone();

                self.triangles[nbhr].orient(Orientation::Even);
                edge_clone.induce_orientation(&self.triangles[nbhr]);
                if edge_clone.orientation() == edge_o {
                    self.triangles[nbhr].swap_orientation();
                }

                orientations.push(self.triangles[nbhr].orientation());
            }

            // Check that we only need one orienation
            if orientations.len() > 1 {
                for i in 0..orientations.len()-1 {
                    if orientations[i] != orientations[i+1] {
                        return Err("unorientable.".to_string());
                    }
                }
            }

            // Induce orientation onto faces and get neighbors
            self._orient_edges(nbhr, &mut visited_edges);
            for nbhr_of_nbhr in self._nbhrs(nbhr) {
                if !visited_tris[nbhr_of_nbhr] && !nbhrs.contains(&nbhr_of_nbhr) {
                    nbhrs.push(nbhr_of_nbhr);
                }
            }
        }

        Ok(())
    }

    fn _orient_edges(&mut self, tri: usize, visited_edges: &mut Vec<bool>) {
        let edges = self._edges(tri);
        for i in 0..3 {
            if !visited_edges[edges[i]] {
                self.edges[edges[i]].induce_orientation(&self.triangles[tri]);
                visited_edges[edges[i]] = true; 
            }
        }
    }

    fn _edges(&self, tri: usize) -> [usize; 3] {
        let mut edges = [0; 3];
        let mut idx = 0;
        for i in 0..self.edges.len() {
            if self.triangles[tri].is_face(&self.edges[i]) {
                edges[idx] = i;
                idx += 1;
            }
        }

        edges
    }

    fn _nbhrs(&self, tri: usize) -> Vec<usize> {
        let mut nbhrs = Vec::new();
        let edges = self._edges(tri);
        for i in 0..self.triangles.len() {
            if i == tri { continue; }
            if self.triangles[i].is_face(&self.edges[edges[0]])
                || self.triangles[i].is_face(&self.edges[edges[1]])
                || self.triangles[i].is_face(&self.edges[edges[2]]) {
                nbhrs.push(i);
            }
        }

        nbhrs
    }
}
