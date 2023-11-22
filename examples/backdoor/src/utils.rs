use itertools::{Itertools, MultiProduct};

use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::var::Var;

pub fn partition_tasks(variables: &[Var], solver: &mut Solver) -> (Vec<Vec<Lit>>, Vec<Vec<Lit>>) {
    let mut hard = Vec::new();
    let mut easy = Vec::new();

    for cube in product_repeat([true, false].into_iter(), variables.len()) {
        let assumptions = variables.iter().zip(cube.iter()).map(|(&v, &s)| Lit::new(v, s)).collect_vec();
        let result = solver.propcheck(&assumptions);
        if result {
            hard.push(assumptions);
        } else {
            easy.push(assumptions);
        }
    }

    (hard, easy)
}

/// Rust version of Python's `itertools.product()`.
/// It returns the cartesian product of the input iterables, and it is
/// semantically equivalent to `repeat` nested for loops.
///
/// # Arguments
///
/// * `it` - An iterator over a cloneable data structure
/// * `repeat` - Number of repetitions of the given iterator
///
/// See https://stackoverflow.com/a/68231315/3592218
pub fn product_repeat<I>(it: I, repeat: usize) -> MultiProduct<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    std::iter::repeat(it).take(repeat).multi_cartesian_product()
}
