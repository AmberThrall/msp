use nalgebra::Vector3;
use std::collections::VecDeque;
use std::fs::File;
use std::path::Path;
use std::io::{prelude::*, BufReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    CCW,
    CW
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge(pub usize, pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle(pub usize, pub usize, pub usize);

/// Represents a mesh in 3D. Assumes that the y-axis is the upwards direction.
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

    pub fn induce_orientation(&mut self, tri: &Triangle) {
        let mut o = Vec::new();
        let mut odd_idx = false;
        for (i, v) in [tri.0, tri.1, tri.2].iter().enumerate() {
            if self.0 != *v && self.1 != *v {
                odd_idx = i % 2 == 1;
            } else {
                o.push(*v);
            }
        }

        self.0 = o[0];
        self.1 = o[1];
        if odd_idx { self.swap_orientation(); }
    }

    pub fn orientation(&self) -> Orientation {
        if self.0 < self.1 { Orientation::CCW }
        else { Orientation::CW }
    }
}

impl Triangle {
    pub fn signed_area(&self, mesh: &Mesh) -> f64 {
        let a = mesh.vertices[self.0];
        let b = mesh.vertices[self.1];
        let c = mesh.vertices[self.2];

        let ab = b - a;
        let ac = c - a;
        let cross = ab.cross(&ac);

        if cross.y > 0.0 {
            // Oriented CCW
            (cross.norm() as f64) / 2.0
        } 
        else {
            // Oriented CW
            -(cross.norm() as f64) / 2.0
        }
    }

    pub fn area(&self, mesh: &Mesh) -> f64 {
        self.signed_area(mesh).abs()    
    }

    pub fn is_face(&self, edge: &Edge) -> bool {
        let v = [self.0, self.1, self.2];
        v.contains(&edge.0) && v.contains(&edge.1)
    }

    pub fn swap_orientation(&mut self) {
        let tmp = self.0;
        self.0 = self.1;
        self.1 = tmp;
    }

    pub fn orient(&mut self, orientation: Orientation) {
        let mut v = [self.0, self.1, self.2];
        v.sort();
        self.0 = v[0]; self.1 = v[1]; self.2 = v[2];

        if orientation == Orientation::CW { 
            self.swap_orientation();    
        }
    }

    pub fn orientation(&self) -> Orientation {
             if self.0 < self.1 && self.1 < self.2 { Orientation::CCW } // [0,1,2]
        else if self.0 < self.2 && self.2 < self.1 { Orientation::CW  } // [0,2,1]
        else if self.1 < self.0 && self.0 < self.2 { Orientation::CW  } // [1,0,2]
        else if self.1 < self.2 && self.2 < self.0 { Orientation::CCW } // [1,2,0]
        else if self.2 < self.0 && self.0 < self.1 { Orientation::CCW } // [2,0,1]
        else if self.2 < self.1 && self.1 < self.2 { Orientation::CW  } // [2,1,0]
        else { Orientation::CW } // Degenerate triangle
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
                if !edges.contains(&Edge(i,j)) && !edges.contains(&Edge(j,i)) { edges.push(Edge(i.min(j),j.max(i))); }
                if !edges.contains(&Edge(i,k)) && !edges.contains(&Edge(k,i)) { edges.push(Edge(i.min(k),k.max(i))); }
                if !edges.contains(&Edge(j,k)) && !edges.contains(&Edge(k,j)) { edges.push(Edge(j.min(k),k.max(j))); }
            }
        }

        Ok(Mesh {
            vertices,
            edges,
            triangles,
        })
    }
    
    pub fn orient2d(&mut self) -> Result<(), String> {
        // Orient all edges lexicographically.
        for edge in self.edges.iter_mut() {
            let a = edge.0; let b = edge.1;
            edge.0 = a.min(b); edge.1 = a.max(b);
        }

        // Orient the triangles by signed area.
        for i in 0..self.triangles.len() {
            self.triangles[i].orient(Orientation::CCW); 

            if self.triangles[i].signed_area(self) < 0.0 {
                self.triangles[i].swap_orientation();
            }
        }

        // Induce orientation onto edges.
        let mut visited = vec![false; self.edges.len()];
        for tri in 0..self.triangles.len() {
            let edges = self._edges(tri);
            for edge in edges.iter() {
                if visited[*edge] { continue; }
                self.edges[*edge].induce_orientation(&self.triangles[tri]);
                visited[*edge] = true;
            }
        }

        Ok(())        
    }

    pub fn orient(&mut self) -> Result<(), String> {
        // Orient all edges lexicographically.
        for edge in self.edges.iter_mut() {
            let a = edge.0; let b = edge.1;
            edge.0 = a.min(b); edge.1 = a.max(b);
        }

        // Orient all triangles lexicographically (CCW) to start.
        for tri in self.triangles.iter_mut() { tri.orient(Orientation::CCW); }

        // Keep track of what triangles we have oriented.
        let mut visited = vec![false; self.triangles.len()]; 

        // Fix the first triangle's orientation and add it to the FIFO queue.
        visited[0] = true;
        let mut queue = VecDeque::new();
        queue.push_back(0);
        
        // Intermediate step: propogate the orientation of face tri to its neighbors.
        while let Some(tri) = queue.pop_front() {
            // Get tri's edges and what oriention tri induces onto each edge.
            let edges = self._edges(tri);
            let orientations: Vec<Orientation> = edges.iter()
                .map(|i| { 
                    let mut e = self.edges[*i].clone(); 
                    e.induce_orientation(&self.triangles[tri]); 
                    e.orientation() 
                })
                .collect();

            // Determine the orientation of each neighboring face.
            for nbhr in self._nbhrs(tri) {
                // For each shared edge (should be 1), ensure that tri and nbhr induce opposite
                // orientations.
                for i in 0..edges.len() {
                    if !self.triangles[nbhr].is_face(&self.edges[edges[i]]) { continue; }

                    let mut edge_copy = self.edges[edges[i]].clone();
                    edge_copy.induce_orientation(&self.triangles[nbhr]);

                    // If both triangles induce the same oriention, swap nbhr's orientation.
                    if orientations[i] == edge_copy.orientation() {
                        if visited[nbhr] { // We've already oriented this triangle and now we need
                                           // to change it's orientation!
                            return Err(format!("{:?} requires both CW and CCW orientation.", self.triangles[nbhr]));
                        }
                        self.triangles[nbhr].swap_orientation();
                    }
                }
                                
                // Add each unvisited neigbhor to our queue
                if !visited[nbhr] { 
                    visited[nbhr] = true;
                    queue.push_back(nbhr);
                }
            }
        }

        Ok(())
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
        let edges: Vec<&Edge> = self._edges(tri).iter().map(|i| &self.edges[*i]).collect();

        let mut nbhrs = Vec::new();
        for i in 0..self.triangles.len() {
            if i == tri { continue; }

            if self.triangles[i].is_face(edges[0])
                || self.triangles[i].is_face(edges[1])
                || self.triangles[i].is_face(edges[2]) {
                nbhrs.push(i);
            }
        }
        nbhrs
    }
}
