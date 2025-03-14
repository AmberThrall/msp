use super::*;
use std::rc::Rc;
use nalgebra::DMatrix;
use good_lp::{constraint, default_solver, variable, variables, Expression, Variable, SolverModel, Solution};

const EPSILON: f64 = 1e-6;

pub struct MSPResult {
    pub median: Chain,
    pub decomp: Vec<(Chain, Chain)>,
}

#[derive(Default)]
struct Variables {
    pub t_plus: Vec<Variable>,
    pub t_minus: Vec<Variable>,
    pub r_plus: Vec<Vec<Variable>>,
    pub r_minus: Vec<Vec<Variable>>,
    pub s_plus: Vec<Vec<Variable>>,
    pub s_minus: Vec<Vec<Variable>>,
}

pub fn median_shape(mesh: Rc<Mesh>, input: Vec<Rc<Chain>>, alpha: Vec<f64>, mu: f64, lambda: f64) -> Result<MSPResult, String> {
    if input.len() != alpha.len() {
        return Err(format!("invalid input, got {} chains and {} weights.", input.len(), alpha.len()));
    }


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
    // sum {h in 1..k} ( a[h]*(sum {i in 1..m} w[i]*(qip[h,i]+qim[h,i])) + Lambda*(sum {j in 1..n} v[j]*(rip[h,j]+rim[h,j]))) 
    //      + Mu*(sum {i in 1..m} w[i]*(tp[i]+tm[i]));
    let w: Vec<f64> = mesh.edges.iter().map(|e| e.length(&mesh)).collect();
    let v: Vec<f64> = mesh.triangles.iter().map(|t| t.area(&mesh)).collect();

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
    // subject to FlatDecomp {h in 1..k, i in 1..m}: tp[i]-tm[i] - Ti[h,i] = qip[h,i]-qim[h,i] + sum {j in 1..n} B[i,j]*(rip[h,j]-rim[h,j]);
    let mut constraints = Vec::new();
    let B = DMatrix::from_fn(m, n, |r, c| {
        let edge = mesh.edges[r];
        let tri = mesh.triangles[c];

        if !tri.is_face(&edge) { 0.0 }
        else {
            let mut ec = edge.clone();
            ec.induce_orientation(&tri);
            if ec.orientation() == tri.orientation() { 1.0 } else { -1.0 }
        }
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
        .map_err(|e| format!("{}", e))?;

    let mut res = MSPResult {
        median: Chain::zero(1, mesh.clone()),
        decomp: Vec::new(),
    };

    for _ in 0..N { res.decomp.push((Chain::zero(1, mesh.clone()), Chain::zero(2, mesh.clone()))); }

    for i in 0..m {
        let v = solution.value(vars.t_plus[i]) - solution.value(vars.t_minus[i]);
        if v.abs() > EPSILON {
            res.median.coeff[i] = v;
        }

        for h in 0..N {
            let v = solution.value(vars.r_plus[h][i]) - solution.value(vars.r_minus[h][i]);
            if v.abs() > EPSILON {
                res.decomp[h].0.coeff[i] = v;
            }
        }
    }

    for j in 0..n {
        for h in 0..N {
            let v = solution.value(vars.s_plus[h][j]) - solution.value(vars.s_minus[h][j]);
            if v.abs() > EPSILON {
                res.decomp[h].1.coeff[j] = v;
            }
        }
    }

    Ok(res)
}
