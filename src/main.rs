mod prelude;
use prelude::*;
use std::rc::Rc;

fn main() {
    // Load the mesh in from disk
    let mesh = match Mesh::load("Plane.off") {
        Ok(m) => Rc::new(m),
        Err(e) => {
            std::eprintln!("Error loading mesh: {}", e);
            std::process::exit(1);
        }
    };

    // Load the chains in from disk
    let c1 = match Chain::load(mesh.clone(), "Chain1.txt") {
        Ok(m) => m,
        Err(e) => {
            std::eprintln!("Error loading chain 1: {}", e);
            std::process::exit(1);
        }
    };

    let c2 = match Chain::load(mesh.clone(), "Chain2.txt") {
        Ok(m) => m,
        Err(e) => {
            std::eprintln!("Error loading chain 2: {}", e);
            std::process::exit(1);
        }
    };
}
