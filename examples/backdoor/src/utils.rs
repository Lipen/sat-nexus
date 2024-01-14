use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter};
use std::iter::zip;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;

use cadical::statik::Cadical;
use cadical::SolveResponse;
use itertools::{Itertools, MultiProduct};
use log::{debug, info};
use ordered_float::OrderedFloat;

use crate::solvers::SatSolver;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::utils::{parse_dimacs, DisplaySlice};
use simple_sat::var::Var;

pub fn parse_multiple_comma_separated_intervals_from<P: AsRef<Path>>(path: P) -> Vec<Vec<usize>> {
    let path = path.as_ref();
    debug!("Reading '{}'", path.display());
    let f = File::open(path).unwrap_or_else(|_| panic!("Could not open '{}'", path.display()));
    let f = BufReader::new(f);
    let mut result = Vec::new();
    for line in f.lines().flatten() {
        result.push(parse_comma_separated_intervals(&line));
    }
    result
}

pub fn parse_multiple_comma_separated_intervals(input: &str) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    for part in input.split(':') {
        result.push(parse_comma_separated_intervals(part));
    }
    result
}

pub fn parse_comma_separated_intervals(input: &str) -> Vec<usize> {
    let mut result = Vec::new();
    for part in input.split(',') {
        let range_parts: Vec<&str> = part.splitn(2, '-').collect();
        if range_parts.len() == 2 {
            let start: usize = range_parts[0].parse().unwrap();
            let end: usize = range_parts[1].parse().unwrap();
            if start <= end {
                result.extend(start..=end);
            } else {
                result.extend((end..=start).rev());
            }
        } else {
            let single: usize = part.parse().unwrap();
            result.push(single);
        }
    }
    result
}

pub fn get_hard_tasks(variables: &[Var], solver: &mut SatSolver) -> Vec<Vec<Lit>> {
    match solver {
        SatSolver::SimpleSat(solver) => {
            let (hard, _easy) = partition_tasks(variables, solver);
            hard
        }
        SatSolver::Cadical(solver) => {
            let vars_external: Vec<i32> = variables.iter().map(|var| var.to_external() as i32).collect();
            let valid = solver.propcheck_all_tree_valid(&vars_external);
            valid
                .into_iter()
                .map(|cube| cube.into_iter().map(|i| Lit::from_external(i)).collect())
                .collect()
        }
    }
}

pub fn partition_tasks(variables: &[Var], solver: &mut Solver) -> (Vec<Vec<Lit>>, Vec<Vec<Lit>>) {
    partition_tasks_with(variables, |cube| solver.propcheck(cube))
}

pub fn partition_tasks_cadical(variables: &[Var], solver: &Cadical) -> (Vec<Vec<Lit>>, Vec<Vec<Lit>>) {
    partition_tasks_with(variables, |cube| {
        let cube: Vec<i32> = cube.iter().map(|lit| lit.to_external()).collect();
        solver.propcheck(&cube)
    })
}

pub fn partition_tasks_cadical_emulated(variables: &[Var], solver: &Cadical) -> (Vec<Vec<Lit>>, Vec<Vec<Lit>>) {
    partition_tasks_with(variables, |cube| {
        let cube: Vec<i32> = cube.iter().map(|lit| lit.to_external()).collect();
        for &lit in cube.iter() {
            solver.assume(lit).unwrap();
        }
        solver.limit("conflicts", 1);
        match solver.solve().unwrap() {
            SolveResponse::Sat => unreachable!(),
            SolveResponse::Unsat => {
                // log::info!("UNSAT on cube {}", DisplaySlice(&cube));
                false
            }
            SolveResponse::Interrupted => {
                // log::info!("UNKNOWN on cube {}", DisplaySlice(&cube));
                true
            }
        }
    })
}

pub fn partition_tasks_with<F>(variables: &[Var], mut propcheck: F) -> (Vec<Vec<Lit>>, Vec<Vec<Lit>>)
where
    F: FnMut(&[Lit]) -> bool,
{
    let mut hard = Vec::new();
    let mut easy = Vec::new();

    for cube in product_repeat([true, false].into_iter(), variables.len()) {
        let assumptions = zip(variables, cube).map(|(&v, s)| Lit::new(v, s)).collect_vec();
        let result = propcheck(&assumptions);
        if result {
            hard.push(assumptions);
        } else {
            easy.push(assumptions);
        }
    }

    (hard, easy)
}

pub fn filter_cubes(
    cubes_product: Vec<Vec<Lit>>,
    num_conflicts_budget: u64,
    num_conflicts_limit: u64,
    solver: &mut SatSolver,
) -> Vec<Vec<Lit>> {
    let variables = cubes_product[0].iter().map(|lit| lit.var()).sorted().collect_vec();
    let n = variables.len();

    debug!("Computing degree...");
    let time_compute_degree = Instant::now();
    let mut degree: HashMap<(Lit, Lit), u64> = HashMap::new();
    for (i, j) in (0..n).tuple_combinations() {
        for cube in cubes_product.iter() {
            assert_eq!(cube.len(), n);
            assert_eq!(cube[i].var(), variables[i]);
            assert_eq!(cube[j].var(), variables[j]);
            let a = cube[i];
            let b = cube[j];
            *degree.entry((a, b)).or_insert(0) += 1;
        }
    }
    // for (&a, &b) in variables.iter().tuple_combinations() {
    //     let pp = degree.get(&(Lit::new(a, false), Lit::new(b, false))).copied().unwrap_or(0);
    //     let pn = degree.get(&(Lit::new(a, false), Lit::new(b,  true))).copied().unwrap_or(0);
    //     let np = degree.get(&(Lit::new(a, true), Lit::new(b, false))).copied().unwrap_or(0);
    //     let nn = degree.get(&(Lit::new(a, true), Lit::new(b, true))).copied().unwrap_or(0);
    //     debug!("degrees for {}-{}: {} / {} / {} / {}", a, b, pp, pn, np, nn);
    // }
    let time_compute_degree = time_compute_degree.elapsed();
    debug!("Computed degree in {:.1}s", time_compute_degree.as_secs_f64());

    let mut cubes_product: Vec<Rc<Vec<Lit>>> = cubes_product.into_iter().map(|cube| Rc::new(cube)).collect();
    let mut indet_cubes: Vec<Vec<Lit>> = Vec::new();

    debug!("Computing neighbors...");
    let time_compute_neighbors = Instant::now();
    let mut neighbors: HashMap<(Lit, Lit), Vec<Rc<Vec<Lit>>>> = HashMap::new();
    for cube in cubes_product.iter() {
        for (&a, &b) in cube.iter().tuple_combinations() {
            neighbors.entry((a, b)).or_default().push(Rc::clone(cube));
        }
    }
    let time_compute_neighbors = time_compute_neighbors.elapsed();
    debug!("Computed neighbors in {:.1}s", time_compute_neighbors.as_secs_f64());

    let compute_cube_score = |cube: &Vec<Lit>, degree: &HashMap<(Lit, Lit), u64>| {
        let mut score: f64 = 0.0;
        for (&a, &b) in cube.iter().tuple_combinations() {
            if let Some(&d) = degree.get(&(a, b)) {
                if d != 0 {
                    score += 1.0 / d as f64;
                    if d == 1 {
                        score += 50.0;
                    }
                }
            }
        }
        score
    };

    debug!("Computing cube score...");
    let time_cube_scores = Instant::now();
    let mut cube_score: HashMap<Rc<Vec<Lit>>, f64> = HashMap::new();
    for cube in cubes_product.iter() {
        cube_score.insert(Rc::clone(cube), compute_cube_score(cube, &degree));
    }
    let time_cube_scores = time_cube_scores.elapsed();
    debug!("Computed cube scores in {:.1}s", time_cube_scores.as_secs_f64());

    while !cubes_product.is_empty() {
        let time_prepare = Instant::now();
        let num_conflicts = match solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => solver.conflicts() as u64,
        };
        if num_conflicts > num_conflicts_limit {
            info!("Budget exhausted");
            break;
        }

        debug!("Asserting...");
        let time_asserting = Instant::now();
        for cube in cubes_product.iter() {
            assert!(
                (compute_cube_score(cube, &degree) - cube_score[cube]).abs() <= 1e-6,
                "compute = {}, score = {}",
                compute_cube_score(cube, &degree),
                cube_score[cube]
            );
        }
        let time_asserting = time_asserting.elapsed();
        debug!("Asserted in {:.1}s", time_asserting.as_secs_f64());

        let pos = cubes_product
            .iter()
            .position_max_by_key(|cube| OrderedFloat(cube_score[*cube]))
            .unwrap();
        let best_cube = cubes_product.swap_remove(pos);
        let best_cube_score = cube_score[&best_cube];

        let time_prepare = time_prepare.elapsed();

        if best_cube_score > 0.0 {
            debug!(
                "Max score ({}) cube in {:.1}s: {}",
                best_cube_score,
                time_prepare.as_secs_f64(),
                DisplaySlice(&best_cube)
            );
            match solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    for &lit in best_cube.iter() {
                        solver.assume(lit.to_external()).unwrap();
                    }
                    solver.limit("conflicts", num_conflicts_budget as i32);
                    // debug!("Solving {}...", DisplaySlice(&best_cube));
                    let time_solve = Instant::now();
                    match solver.solve().unwrap() {
                        SolveResponse::Unsat => {
                            debug!(
                                "UNSAT in {:.1}s for {}",
                                time_solve.elapsed().as_secs_f64(),
                                DisplaySlice(&best_cube)
                            );
                            let time_rescore = Instant::now();
                            for (&a, &b) in best_cube.iter().tuple_combinations() {
                                let d = degree[&(a, b)];
                                if d == 0 {
                                    continue;
                                }
                                *degree.get_mut(&(a, b)).unwrap() -= 1;
                                let d = degree[&(a, b)];
                                if d == 0 {
                                    debug!("should derive {}", DisplaySlice(&[-a, -b]));
                                    assert_eq!(neighbors[&(a, b)].len(), 1);
                                    for cube in neighbors[&(a, b)].iter() {
                                        // score[cube] -= 50
                                        *cube_score.get_mut(cube.as_ref()).unwrap() -= 50.0;
                                    }
                                }
                                for cube in neighbors[&(a, b)].iter() {
                                    // score[cube] -= 1 / (d+1)
                                    *cube_score.get_mut(cube.as_ref()).unwrap() -= 1.0 / (d + 1) as f64;
                                    if d != 0 {
                                        // score[cube] += 1 / d
                                        *cube_score.get_mut(cube.as_ref()).unwrap() += 1.0 / d as f64;
                                        if d == 1 {
                                            // score[cube] += 50
                                            *cube_score.get_mut(cube.as_ref()).unwrap() += 50.0;
                                        }
                                    }
                                }
                                neighbors.get_mut(&(a, b)).unwrap().retain(|c| !Rc::ptr_eq(c, &best_cube));
                                // neighbors.get_mut(&(a, b)).unwrap().retain(|c| c != &best_cube);
                            }
                            let time_rescore = time_rescore.elapsed();
                            debug!("Rescored in {:.1}s", time_rescore.as_secs_f64());
                        }
                        SolveResponse::Interrupted => {
                            debug!(
                                "INDET in {:.1}s for {}",
                                time_solve.elapsed().as_secs_f64(),
                                DisplaySlice(&best_cube)
                            );
                            let time_rescore = Instant::now();
                            for (&a, &b) in best_cube.iter().tuple_combinations() {
                                let d = degree[&(a, b)];
                                for cube in neighbors[&(a, b)].iter() {
                                    // score[cube] -= 1 / d
                                    *cube_score.get_mut(cube.as_ref()).unwrap() -= 1.0 / d as f64;
                                }
                                degree.insert((a, b), 0);
                                neighbors.get_mut(&(a, b)).unwrap().clear();
                            }
                            let time_rescore = time_rescore.elapsed();
                            debug!("Rescored in {:.1}s", time_rescore.as_secs_f64());
                            indet_cubes.push((*best_cube).clone());
                        }
                        SolveResponse::Sat => panic!("Unexpected SAT"),
                    }
                }
            }
        } else {
            indet_cubes.push((*best_cube).clone());
            break;
        }
    }

    drop(neighbors);
    drop(cube_score);
    let mut cubes_product: Vec<Vec<Lit>> = cubes_product.into_iter().map(|cube| Rc::try_unwrap(cube).unwrap()).collect();
    cubes_product.extend(indet_cubes);
    cubes_product
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

pub fn concat_cubes(a: Vec<Lit>, b: Vec<Lit>) -> Vec<Lit> {
    let mut r = HashSet::new();
    r.extend(a);
    r.extend(b);
    let mut r = r.into_iter().collect_vec();
    r.sort_by_key(|lit| lit.inner());
    r
    // let mut r = Vec::new();
    // r.extend(a);
    // for x in b {
    //     if !r.contains(&x) {
    //         r.push(x);
    //     }
    // }
    // r
}

pub fn bits_to_number(bits: &[bool]) -> u32 {
    let mut result: u32 = 0;
    for &bit in bits.iter() {
        result <<= 1;
        result |= bit as u32;
    }
    result
}

pub fn gray_to_index(bits: &[bool]) -> u32 {
    let mut num = bits_to_number(bits);
    num ^= num >> 16;
    num ^= num >> 8;
    num ^= num >> 4;
    num ^= num >> 2;
    num ^= num >> 1;
    num
}

pub fn mask(base: &[Lit], data: &[Lit]) -> Vec<bool> {
    let base = base.iter().map(|lit| lit.var()).collect::<HashSet<_>>();
    data.iter().map(|lit| !base.contains(&lit.var())).collect()
}

pub fn determine_vars_pool<P: AsRef<Path>>(path: P, allowed_vars: &Option<String>, banned_vars: &Option<String>) -> Vec<Var> {
    // Determine the set of variables encountered in CNF:
    let mut encountered_vars = HashSet::new();
    for clause in parse_dimacs(path) {
        for lit in clause {
            encountered_vars.insert(lit.var());
        }
    }

    // Ban some variables:
    if let Some(banned_vars) = banned_vars {
        let chunks = if banned_vars.starts_with('@') {
            parse_multiple_comma_separated_intervals_from(&banned_vars[1..])
        } else {
            parse_multiple_comma_separated_intervals(&banned_vars)
        };
        let banned_vars: HashSet<Var> = chunks.into_iter().flatten().map(|i| Var::from_external(i as u32)).collect();
        encountered_vars.retain(|v| !banned_vars.contains(v));
    }

    // Allow only some variables:
    if let Some(allowed_vars) = allowed_vars {
        let chunks = if allowed_vars.starts_with('@') {
            parse_multiple_comma_separated_intervals_from(&allowed_vars[1..])
        } else {
            parse_multiple_comma_separated_intervals(&allowed_vars)
        };
        let allowed_vars: HashSet<Var> = chunks.into_iter().flatten().map(|i| Var::from_external(i as u32)).collect();
        encountered_vars.retain(|v| allowed_vars.contains(v));
    }

    // Create the pool of variables:
    let pool: Vec<Var> = encountered_vars.into_iter().sorted().collect();
    pool
}

pub fn create_line_writer<P: AsRef<Path>>(path: P) -> LineWriter<File> {
    let path = path.as_ref();
    let f = File::create(path).unwrap_or_else(|_| panic!("Could not create '{}'", path.display()));
    let f = LineWriter::new(f);
    f
}

pub fn maybe_create<P: AsRef<Path>>(path: &Option<P>) -> Option<LineWriter<File>> {
    path.as_ref().map(create_line_writer)
}

pub fn clause_to_external<'a, I>(lits: I) -> impl Iterator<Item = i32> + 'a
where
    I: IntoIterator<Item = &'a Lit>,
    <I as IntoIterator>::IntoIter: 'a,
{
    lits.into_iter().map(|lit| lit.to_external())
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    #[test]
    fn test_gray_to_number() {
        assert_eq!(0, gray_to_index(&[false, false, false, false]));
        assert_eq!(1, gray_to_index(&[false, false, false, true]));
        assert_eq!(2, gray_to_index(&[false, false, true, true]));
        assert_eq!(3, gray_to_index(&[false, false, true, false]));
        assert_eq!(4, gray_to_index(&[false, true, true, false]));
        assert_eq!(5, gray_to_index(&[false, true, true, true]));
        assert_eq!(6, gray_to_index(&[false, true, false, true]));
        assert_eq!(7, gray_to_index(&[false, true, false, false]));
        assert_eq!(8, gray_to_index(&[true, true, false, false]));
        assert_eq!(9, gray_to_index(&[true, true, false, true]));
        assert_eq!(10, gray_to_index(&[true, true, true, true]));
        assert_eq!(11, gray_to_index(&[true, true, true, false]));
        assert_eq!(12, gray_to_index(&[true, false, true, false]));
        assert_eq!(13, gray_to_index(&[true, false, true, true]));
        assert_eq!(14, gray_to_index(&[true, false, false, true]));
        assert_eq!(15, gray_to_index(&[true, false, false, false]));
    }

    #[test]
    fn test_mask() {
        let x1 = Lit::from_external(1);
        let x2 = Lit::from_external(2);
        let x3 = Lit::from_external(3);
        let x5 = Lit::from_external(5);
        let x7 = Lit::from_external(7);
        let x8 = Lit::from_external(8);
        let x9 = Lit::from_external(9);
        assert_equal(
            mask(&[x1, x2, x3, x7, x9], &[x2, x3, x5, x7, x8]),
            [false, false, true, false, true],
        )
    }
}
