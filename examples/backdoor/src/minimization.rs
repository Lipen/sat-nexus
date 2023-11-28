use itertools::Itertools;

use pyo3_example::backdoor_to_clauses;
use simple_sat::lit::Lit;

pub fn minimize_backdoor(cubes: &[Vec<Lit>]) -> Vec<Vec<Lit>> {
    // Convert to i32-cubes (`Vec<i32>`):
    let cubes = cubes
        .iter()
        .map(|cube| cube.iter().map(|lit| lit.to_external()).collect_vec())
        .collect_vec();

    // Minimize:
    let mut clauses = backdoor_to_clauses(cubes);

    // Sort:
    for c in clauses.iter_mut() {
        c.sort_by_key(|lit| lit.unsigned_abs());
    }
    clauses.sort_by_key(|c| (c.len(), c.iter().map(|lit| lit.unsigned_abs()).collect_vec()));

    // Convert to Lit-clauses (`Vec<Lit>`):
    clauses
        .into_iter()
        .map(|c| c.into_iter().map(|lit| Lit::from_external(lit)).collect_vec())
        .collect_vec()
}
