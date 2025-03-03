use super::Mesh;
use std::path::Path;
use std::rc::Rc;
use std::fs::File;
use std::io::{prelude::*, BufReader};

pub struct Chain {
    pub mesh: Rc<Mesh>,
    pub coeff: Vec<f64>,
}

impl Chain {
    pub fn zero(mesh: Rc<Mesh>) -> Chain {
        let coeff = vec![0.0; mesh.edges.len()];
        Chain {
            mesh,
            coeff,
        }
    }

    pub fn load<P: AsRef<Path>>(mesh: Rc<Mesh>, path: P) -> Result<Chain, String> {
        let file = File::open(path).map_err(|e| format!("error opening file: {}", e))?;
        
        let mut coeff = vec![0.0; mesh.edges.len()];

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            if line.chars().nth(0) == Some('#') { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();

            let i = parts[0].parse::<usize>().map_err(|_| "invalid edge")?;
            let j = parts[1].parse::<usize>().map_err(|_| "invalid edge")?;

            // Find the edge (i,j) in the mesh
            let mut found = false;
            for idx in 0..mesh.edges.len() {
                if mesh.edges[idx] == (i,j) {
                    coeff[idx] = 1.0;
                    found = true;
                    break;
                }
                else if mesh.edges[idx] == (j,i) {
                    coeff[idx] = -1.0;
                    found = true;
                    break;
                }
            }

            if !found { return Err(format!("unknown edge {:?}", (i,j))) }
        }

        Ok(Chain {
            mesh,
            coeff,
        })
    }
}
