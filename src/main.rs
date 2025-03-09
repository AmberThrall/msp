mod prelude;
use prelude::*;
use std::rc::Rc;

fn load_current(mesh: Rc<Mesh>, path: &str) -> Rc<Chain> {
    match Current::load(mesh.clone(), path) {
        Ok(m) => match m.as_chain() {
            Ok(c) => Rc::new(c),
            Err(e) => {
                std::eprintln!("Error generating chain: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            std::eprintln!("Error loading current: {}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    let mut mesh = Mesh::load("test.off").unwrap();
    mesh.orient().unwrap();
    println!("edges = {:?}", mesh.edges);
    println!("triangles = {:?}", mesh.triangles);

    // Load the mesh in from disk
    println!("Loading mesh...");
    let mesh = match Mesh::load("Sphere.off") {
        Ok(mut m) => {
            // Orient the mesh
            println!("Orienting mesh...");
            /*if m.orient2d().is_err() { 
                std::eprintln!("Error: mesh is not orientable.");
                std::process::exit(1);
            }*/
            Rc::new(m)
        },
        Err(e) => {
            std::eprintln!("Error loading mesh: {}", e);
            std::process::exit(1);
        }
    };

    // Load the chains in from disk
    println!("Loading currents...");
    let c1 = load_current(mesh.clone(), "SphereCurrent1.txt");
    let c2 = load_current(mesh.clone(), "SphereCurrent2.txt");
    //let c3 = load_current(mesh.clone(), "SphereCurrent3.txt");

    // Solve the problem
    println!("Solving LP...");
    let msp = MedianShape::new(mesh.clone(), 1e-5, 1e-7)
        .add_chain(c1.clone(), 0.5)
        //.add_chain(c2.clone(), 0.33)
        .add_chain(c2.clone(), 0.5);


    let median = match msp.solve() {
        Ok(m) => m,
        Err(e) => {
            std::eprintln!("Error solving LP: {}", e);
            std::process::exit(1);
        }
    };

    println!("Result: {}", median);
    c1.save("chain1.txt").expect("failed to save chain 1");
    c2.save("chain2.txt").expect("failed to save chain 2");
    //c3.save("chain3.txt").expect("failed to save chain 3");
    median.save("median.txt").expect("failed to save median chain");
}
