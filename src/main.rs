mod prelude;
use prelude::*;

fn main() {
    // Load the mesh in from disk
    let mesh = match Mesh::load("Plane.off") {
        Ok(m) => m,
        Err(e) => {
            std::eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
}
