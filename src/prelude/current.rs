use super::{Edge, Mesh, Chain};
use nalgebra::Vector3;
use std::path::Path;
use std::rc::Rc;
use std::fs::File;
use std::io::{prelude::*, BufReader};

pub struct Current {
    pub mesh: Rc<Mesh>,
    pub path: Vec<usize>,
}

impl Current {
    pub fn new(mesh: Rc<Mesh>) -> Current {
        Current {
            mesh,
            path: Vec::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(mesh: Rc<Mesh>, path: P) -> Result<Current, String> {
        let file = File::open(path).map_err(|e| format!("error opening file: {}", e))?;
        
        let mut path = Vec::new();

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            if line.chars().nth(0) == Some('#') { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();

            let x = parts[0].parse::<f64>().map_err(|_| "invalid 'x' coordinate".to_string())?;
            let y = parts[1].parse::<f64>().map_err(|_| "invalid 'y' coordinate".to_string())?;
            let z = parts[2].parse::<f64>().map_err(|_| "invalid 'z' coordinate".to_string())?;
            let pt = Vector3::new(x, y, z);

            // Find the closest vertex in the mesh.
            let mut closest = None;
            let mut min_dist = 0.0;
            for idx in 0..mesh.vertices.len() {
                let delta = pt - mesh.vertices[idx];
                let d2 = delta.norm_squared();
                if closest.is_none() || d2 < min_dist {
                    closest = Some(idx);
                    min_dist = d2;
                    if min_dist < 0.0000001 { break; } // We seem to have hit it.
                }
            }

            // This should never be the case.
            if closest.is_none() { return Err(format!("Failed to find vertex close to {:?}", pt));  }

            path.push(closest.unwrap());
        }

        Ok(Current {
            mesh,
            path,
        })
    }

    pub fn as_chain(&self) -> Result<Chain, String> {
        let mut chain = Chain::zero(self.mesh.clone());

        for i in 1..self.path.len() {
            let a = self.path[i-1];
            let b = self.path[i];

            let mut found = false;
            for idx in 0..self.mesh.edges.len() {
                if self.mesh.edges[idx] == Edge(a,b) {
                    chain.coeff[idx] = 1.0;
                    found = true;
                    break;
                }
                else if self.mesh.edges[idx] == Edge(b,a) {
                    chain.coeff[idx] = 1.0;
                    found = true;
                    break;
                }
            }

            if !found { return Err(format!("unknown edge {:?}", (a,b))); }
        }

        Ok(chain)
    }
}
