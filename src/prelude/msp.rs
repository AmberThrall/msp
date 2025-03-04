use super::*;
use std::rc::Rc;
use nalgebra::DMatrix;
use good_lp::{constraint, default_solver, variable, variables, Expression, Variable, SolverModel, Solution};

const EPSILON: f64 = 1e-6;

#[derive(Default)]
struct Variables {
    pub t_plus: Vec<Variable>,
    pub t_minus: Vec<Variable>,
    pub r_plus: Vec<Vec<Variable>>,
    pub r_minus: Vec<Vec<Variable>>,
    pub s_plus: Vec<Vec<Variable>>,
    pub s_minus: Vec<Vec<Variable>>,
}

fn is_face(edge: Edge, tri: Triangle) -> bool {
         if tri.0 == edge.0 && tri.1 == edge.1 { true }
    else if tri.0 == edge.0 && tri.2 == edge.1 { true }
    else if tri.1 == edge.0 && tri.2 == edge.1 { true }
    else if tri.0 == edge.1 && tri.1 == edge.0 { true }
    else if tri.0 == edge.1 && tri.2 == edge.0 { true }
    else if tri.1 == edge.1 && tri.2 == edge.0 { true }
    else { false }
}

fn edge_length(mesh: &Mesh, edge: &Edge) -> f64 {
    let ab = mesh.vertices[edge.0]-mesh.vertices[edge.1];
    ab.norm() as f64
}

fn area(mesh: &Mesh, tri: &Triangle) -> f64 {
    let ab = mesh.vertices[tri.0]-mesh.vertices[tri.1];
    let ac = mesh.vertices[tri.0]-mesh.vertices[tri.2];
    let cross = ab.cross(&ac);
    (cross.norm() as f64) / 2.0
}

pub fn median_shape(mesh: Rc<Mesh>, input: Vec<Chain>, alpha: Vec<f64>, mu: f64, lambda: f64) -> Chain {
    ///////////////////////////
    // Construct the problem //
    ///////////////////////////
    let m = mesh.edges.len();
    let n = mesh.triangles.len();
    let N = input.len();
    let mut problem = variables!();

    //  - Decision variables
    let mut vars = Variables::default(); 
    for _ in 0..m {
        vars.t_plus.push(problem.add(variable().min(0.0)));
        vars.t_minus.push(problem.add(variable().min(0.0)));
    }

    for h in 0..N {
        vars.r_plus.push(vec![]);
        vars.r_minus.push(vec![]);
        for _ in 0..m {
            vars.r_plus[h].push(problem.add(variable().min(0.0)));
            vars.r_minus[h].push(problem.add(variable().min(0.0)));
        }

        vars.s_plus.push(vec![]);
        vars.s_minus.push(vec![]);
        for _ in 0..n {
            vars.s_plus[h].push(problem.add(variable().min(0.0)));
            vars.s_minus[h].push(problem.add(variable().min(0.0)));
        }
    }

    //  - Objective Function
    let w: Vec<f64> = mesh.edges.iter().map(|e| edge_length(&mesh, e)).collect();
    let v: Vec<f64> = mesh.triangles.iter().map(|t| area(&mesh, t)).collect();

    let mut objective: Expression = 0.into();
    for i in 0..m {
        objective += mu * w[i] * (vars.t_plus[i] + vars.t_minus[i]);
    }

    for h in 0..N {
        for i in 0..m {
            objective += alpha[h] * w[i] * (vars.r_plus[h][i] + vars.r_minus[h][i]);
        }

        for j in 0..n {
            objective += alpha[h] * lambda * v[j] * (vars.s_plus[h][j] + vars.s_minus[h][j]);
        }
    }

    //  - Constraints
    let mut constraints = Vec::new();
    let B = DMatrix::from_fn(m, n, |r, c| {
        let sigma = mesh.edges[r];
        let tau = mesh.triangles[c];
        if is_face(sigma, tau) { 1.0 } else { 0.0 }
    });
    for h in 0..N {
        for i in 0..m {
            let lhs = vars.t_plus[i] - vars.t_minus[i] - input[h].coeff[i];
            let mut rhs = vars.r_plus[h][i] - vars.r_minus[h][i];

            for j in 0..n {
                if B[(i,j)] != 0.0 {
                    rhs += B[(i,j)] * (vars.s_plus[h][j] - vars.s_minus[h][j]);
                }
            }

            //constraints.push(constraint!(lhs.clone() - rhs.clone() <= EPSILON));
            //constraints.push(constraint!(rhs - lhs <= EPSILON));
            constraints.push(constraint!(lhs == rhs));
        }
    }

    ///////////////////
    // Solve the LP  //
    ///////////////////
    let solution = problem.minimise(objective)
        .using(default_solver)
        .with_all(constraints)
        .solve()
        .unwrap();

    let mut res = Chain::zero(mesh.clone());
    for i in 0..m {
        let v = solution.value(vars.t_plus[i]) - solution.value(vars.t_minus[i]);
        if v.abs() > EPSILON {
            res.coeff[i] = v;
        }
    }

    res
}
