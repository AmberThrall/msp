use super::{Edge, Mesh};
use std::path::Path;
use std::rc::Rc;
use std::fs::File;
use std::fmt;
use std::io::{prelude::*, BufReader};

pub struct Chain {
    pub dim: u32,
    pub mesh: Rc<Mesh>,
    pub coeff: Vec<f64>,
}

impl Chain {
    pub fn zero(dim: u32, mesh: Rc<Mesh>) -> Chain {
        let coeff = match dim {
            1 => vec![0.0; mesh.edges.len()],
            2 => vec![0.0; mesh.triangles.len()],
            _ => panic!("Unsupported dimension.")
        };
        Chain {
            dim,
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
                if mesh.edges[idx] == Edge(i,j)
                    || mesh.edges[idx] == Edge(j,i) {
                    coeff[idx] = 1.0;
                    found = true;
                    break;
                }
            }

            if !found { return Err(format!("unknown edge {:?}", (i,j))) }
        }

        Ok(Chain {
            dim: 1,
            mesh,
            coeff,
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        for i in 0..self.coeff.len() {
            if self.coeff[i] != 0.0 {
                match self.dim {
                    1 => write!(file, "{} {}\n", self.mesh.edges[i].0, self.mesh.edges[i].1)?,
                    2 => write!(file, "{} {} {}\n", self.mesh.triangles[i].0, self.mesh.triangles[i].1, self.mesh.triangles[i].2)?,
                    _ => panic!("unsupported dimension.")
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.mesh.edges.len() {
            if self.coeff[i] == 0.0 { continue; }
            if self.coeff[i] < 0.0 { write!(f, "- ")?; }
            if self.coeff[i] > 0.0 { write!(f, "+ ")?; }
            if self.coeff[i].abs() != 1.0 { write!(f, "{}*{:?} ", self.coeff[i].abs(), self.mesh.edges[i])?; }
            else { write!(f, "{:?} ", self.mesh.edges[i])?; }
        }
        Ok(())
    }
}
