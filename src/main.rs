mod prelude;
use prelude::*;
use std::rc::Rc;

fn main() {
    // Load the mesh in from disk
    println!("Loading mesh...");
    let mesh = match Mesh::load("Plane.off") {
        Ok(m) => Rc::new(m),
        Err(e) => {
            std::eprintln!("Error loading mesh: {}", e);
            std::process::exit(1);
        }
    };

    // Load the chains in from disk
    println!("Loading chains...");
    let c1 = match Chain::load(mesh.clone(), "Chain1.txt") {
        Ok(m) => Rc::new(m),
        Err(e) => {
            std::eprintln!("Error loading chain 1: {}", e);
            std::process::exit(1);
        }
    };

    let c2 = match Chain::load(mesh.clone(), "Chain2.txt") {
        Ok(m) => Rc::new(m),
        Err(e) => {
            std::eprintln!("Error loading chain 2: {}", e);
            std::process::exit(1);
        }
    };

    // Solve the problem
    println!("Solving LP...");
    let msp = MedianShape::new(mesh.clone(), 1e-5, 1e-5)
        .add_chain(c1, 0.4)
        .add_chain(c2, 0.6);

    let median = match msp.solve() {
        Ok(m) => m,
        Err(e) => {
            std::eprintln!("Error solving LP: {}", e);
            std::process::exit(1);
        }
    };

    println!("Result: {}", median);
    median.save("median.txt").expect("failed to save median chain");
}
