use std::time::Instant;

use log::debug;

use sat_nexus_core::op::rel::{encode_both, encode_geq_reified, encode_leq_reified};
use sat_nexus_core::solver::{SolveResponse, Solver};

use crate::utils::num2bits;

pub fn encode_interval(lits: &[i32], low: usize, high: usize) -> Vec<Vec<i32>> {
    // info!("Encoding interval [{}, {}] for {} variables...", low, high, lits.len());
    let n = lits.len();
    let a = num2bits(low, n);
    let b = num2bits(high, n);
    encode_both(lits, &a, &b)
}

pub fn get_bounds(interval_index: usize, interval_size: usize) -> (usize, usize) {
    let low = interval_index * interval_size;
    let high = low + interval_size - 1;
    (low, high)
}

pub fn solve_interval(solver: &mut impl Solver, input_variables: &[usize], interval_size: usize, interval_index: usize) -> SolveResponse {
    let lits: Vec<i32> = input_variables.iter().map(|&x| x as i32).collect();
    let (low, high) = get_bounds(interval_index, interval_size);
    let clauses = encode_interval(&lits, low, high);

    debug!(
        "Adding {} clauses encoding the interval #{} [{}, {}] of size {} to the solver...",
        clauses.len(),
        interval_index,
        low,
        high,
        interval_size
    );
    for clause in clauses.iter() {
        solver.add_clause(clause)
    }

    debug!("Solving...");
    let time_start_solve = Instant::now();
    let result = solver.solve();
    let time_solve = time_start_solve.elapsed();
    debug!(
        "Result for interval #{} [{}, {}] of size {}: {} in {:.3}s",
        interval_index,
        low,
        high,
        interval_size,
        result,
        time_solve.as_secs_f64()
    );

    result
}

pub fn encode_interval_reified(lits: &[i32], low: usize, high: usize, t_geq: i32, t_leq: i32, t_both: i32) -> Vec<Vec<i32>> {
    // info!("Encoding interval [{}, {}] for {} variables...", low, high, lits.len());
    let n = lits.len();
    let a = num2bits(low, n);
    let b = num2bits(high, n);
    // encode_both(lits, &a, &b)
    let mut clauses = Vec::new();
    clauses.extend(encode_geq_reified(t_geq, lits, &a));
    clauses.extend(encode_leq_reified(t_leq, lits, &b));
    clauses.push(vec![-t_both, t_geq]);
    clauses.push(vec![-t_both, t_leq]);
    clauses.push(vec![t_both, -t_geq, -t_leq]);
    clauses
}

pub fn solve_interval_reified(
    solver: &mut impl Solver,
    input_variables: &[usize],
    interval_size: usize,
    interval_index: usize,
) -> SolveResponse {
    let lits: Vec<i32> = input_variables.iter().map(|&x| x as i32).collect();
    let low = interval_size * interval_index;
    let high = low + interval_size - 1;
    let t_geq = solver.new_var().get();
    let t_leq = solver.new_var().get();
    let t_both = solver.new_var().get();
    let clauses = encode_interval_reified(&lits, low, high, t_geq, t_leq, t_both);

    debug!(
        "Adding {} clauses encoding (reified to {}) the interval #{} [{}, {}] of size {} to the solver...",
        clauses.len(),
        t_both,
        interval_index,
        low,
        high,
        interval_size
    );
    for clause in clauses.iter() {
        solver.add_clause(clause)
    }

    debug!("Assuming {}", t_both);
    solver.assume(t_both);

    debug!("Solving...");
    let time_start_solve = Instant::now();
    let result = solver.solve();
    let time_solve = time_start_solve.elapsed();
    debug!(
        "Result for interval #{} [{}, {}] of size {}: {} in {:.3}s",
        interval_index,
        low,
        high,
        interval_size,
        result,
        time_solve.as_secs_f64()
    );

    result
}
