mod mesh;
mod chain;
mod current;
mod msp;

pub use mesh::*;
pub use chain::*;
pub use current::*;
pub use msp::*;

use std::rc::Rc;

pub struct MedianShape {
    mesh: Rc<Mesh>,
    lambda: f64,
    mu: f64,
    chains: Vec<Rc<Chain>>,
    alpha: Vec<f64>
}

impl MedianShape {
    pub fn new(mesh: Rc<Mesh>, mu: f64, lambda: f64) -> Self {
        MedianShape {
            mesh,
            lambda,
            mu,
            chains: Vec::new(),
            alpha: Vec::new(),
        }
    }

    pub fn add_chain(mut self, chain: Rc<Chain>, weight: f64) -> Self {
        self.chains.push(chain);
        self.alpha.push(weight);
        self
    }

    pub fn solve(self) -> Result<MSPResult, String> {
        median_shape(self.mesh, self.chains, self.alpha, self.mu, self.lambda)
    }
}
