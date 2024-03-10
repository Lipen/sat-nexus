use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::path::Path;
use std::time::Instant;

use itertools::{zip_eq, Itertools, MultiProduct};
use log::{debug, info, trace};
use ordered_float::OrderedFloat;

use cadical::statik::Cadical;
use cadical::SolveResponse;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::trie::Trie;
use simple_sat::utils::{parse_dimacs, DisplaySlice};
use simple_sat::var::Var;

use crate::solvers::SatSolver;

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
    partition_tasks_with(variables, |cube| solver.propcheck(cube, None, None))
}

pub fn partition_tasks_cadical(variables: &[Var], solver: &Cadical) -> (Vec<Vec<Lit>>, Vec<Vec<Lit>>) {
    partition_tasks_with(variables, |cube| {
        let cube: Vec<i32> = cube.iter().map(|lit| lit.to_external()).collect();
        solver.propcheck(&cube, true)
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
        let assumptions = zip_eq(variables, cube).map(|(&v, s)| Lit::new(v, s)).collect_vec();
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
    cubes: Vec<Vec<Lit>>,
    num_conflicts_budget: u64,
    num_conflicts_limit: u64,
    solver: &mut SatSolver,
    mysolver: &mut Solver,
    all_clauses: &mut HashSet<Vec<Lit>>,
    all_derived_clauses: &mut Vec<Vec<Lit>>,
    file_derived_clauses: &mut Option<LineWriter<File>>,
) -> Vec<Vec<Lit>> {
    debug!("Computing neighbors...");
    let time_compute_neighbors = Instant::now();
    let mut neighbors: HashMap<(Lit, Lit), Vec<usize>> = HashMap::new();
    for (i, cube) in cubes.iter().enumerate() {
        for (&a, &b) in cube.iter().tuple_combinations() {
            neighbors.entry((a, b)).or_default().push(i);
        }
    }
    let time_compute_neighbors = time_compute_neighbors.elapsed();
    debug!(
        "Computed neighbors (size={}, cubes={}) in {:.1}s",
        neighbors.len(),
        neighbors.values().map(|vs| vs.len()).sum::<usize>(),
        time_compute_neighbors.as_secs_f64()
    );

    let compute_cube_score = |cube: &[Lit], neighbors: &HashMap<(Lit, Lit), Vec<usize>>| {
        let mut score: f64 = 0.0;
        for (&a, &b) in cube.iter().tuple_combinations() {
            if let Some(neighbors) = neighbors.get(&(a, b)) {
                let d = neighbors.len();
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
    let mut cube_score: Vec<f64> = cubes.iter().map(|cube| compute_cube_score(cube, &neighbors)).collect();
    let time_cube_scores = time_cube_scores.elapsed();
    debug!(
        "Computed cube scores (size={}) in {:.1}s",
        cube_score.len(),
        time_cube_scores.as_secs_f64()
    );

    let mut remaining_cubes: Vec<usize> = (0..cubes.len()).collect();
    let mut indet_cubes: Vec<usize> = Vec::new();

    let verb = false;

    while !remaining_cubes.is_empty() {
        let num_conflicts = match solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => solver.conflicts() as u64,
        };
        if num_conflicts > num_conflicts_limit {
            info!("Budget exhausted");
            break;
        }

        if false {
            // debug!("Asserting...");
            let time_asserting = Instant::now();
            for &i in remaining_cubes.iter() {
                assert!(
                    (compute_cube_score(&cubes[i], &neighbors) - cube_score[i]).abs() <= 1e-6,
                    "compute = {}, score = {}",
                    compute_cube_score(&cubes[i], &neighbors),
                    cube_score[i]
                );
            }
            let time_asserting = time_asserting.elapsed();
            debug!("Asserted in {:.1}s", time_asserting.as_secs_f64());
        }

        let best_cube_position = remaining_cubes
            .iter()
            .position_max_by_key(|&&i| OrderedFloat(cube_score[i]))
            .unwrap();
        let best_cube = remaining_cubes.swap_remove(best_cube_position);
        let best_cube_score = cube_score[best_cube];

        if best_cube_score > 0.0 {
            // debug!(
            //     "Max score ({}) cube: {}",
            //     best_cube_score,
            //     DisplaySlice(&cubes[best_cube])
            // );
            match solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    for &lit in cubes[best_cube].iter() {
                        solver.assume(lit.to_external()).unwrap();
                    }
                    solver.limit("conflicts", num_conflicts_budget as i32);
                    // debug!("Solving {}...", DisplaySlice(&best_cube));
                    let time_solve = Instant::now();
                    match solver.solve().unwrap() {
                        SolveResponse::Unsat => {
                            if verb {
                                debug!(
                                    "UNSAT in {:.1}s for cube with score {}: {}",
                                    time_solve.elapsed().as_secs_f64(),
                                    best_cube_score,
                                    DisplaySlice(&cubes[best_cube])
                                );
                            }
                            let time_rescore = Instant::now();
                            for (&a, &b) in cubes[best_cube].iter().tuple_combinations() {
                                let d = neighbors[&(a, b)].len();
                                if d == 0 {
                                    continue;
                                } else if d == 1 {
                                    // debug!("should derive {}", DisplaySlice(&[-a, -b]));
                                    assert_eq!(neighbors[&(a, b)][0], best_cube);
                                    cube_score[best_cube] = 0.0;
                                } else {
                                    for &i in neighbors[&(a, b)].iter() {
                                        cube_score[i] -= 1.0 / d as f64;
                                        cube_score[i] += 1.0 / (d - 1) as f64;
                                        if d - 1 == 1 {
                                            cube_score[i] += 50.0;
                                        }
                                    }
                                }
                                neighbors.get_mut(&(a, b)).unwrap().retain(|&i| i != best_cube);
                            }
                            let time_rescore = time_rescore.elapsed();
                            if verb || time_rescore.as_secs_f64() > 0.1 {
                                debug!("Rescored in {:.1}s", time_rescore.as_secs_f64());
                            }

                            if true {
                                let mut lemma = Vec::new();
                                for &lit in cubes[best_cube].iter() {
                                    if solver.failed(lit.to_external()).unwrap() {
                                        lemma.push(-lit);
                                    }
                                }
                                // debug!("UNSAT for cube = {}, lemma = {}", DisplaySlice(&cube), DisplaySlice(&lemma));
                                lemma.sort_by_key(|lit| lit.inner());
                                if lemma.len() <= 5 && all_clauses.insert(lemma.clone()) {
                                    debug!("new lemma from unsat core: {}", DisplaySlice(&lemma));
                                    if let Some(f) = file_derived_clauses {
                                        for lit in lemma.iter() {
                                            write!(f, "{} ", lit).unwrap();
                                        }
                                        writeln!(f, "0").unwrap();
                                    }
                                    solver.add_clause(clause_to_external(&lemma));
                                    mysolver.add_clause(&lemma);
                                    all_derived_clauses.push(lemma);
                                }
                            }
                        }
                        SolveResponse::Interrupted => {
                            if verb {
                                debug!(
                                    "INDET in {:.1}s for cube with score {}: {}",
                                    time_solve.elapsed().as_secs_f64(),
                                    best_cube_score,
                                    DisplaySlice(&cubes[best_cube])
                                );
                            }
                            let time_rescore = Instant::now();
                            for (&a, &b) in cubes[best_cube].iter().tuple_combinations() {
                                let ns = neighbors.get_mut(&(a, b)).unwrap();
                                let d = ns.len();
                                for i in ns.drain(..) {
                                    // score[cube] -= 1 / d
                                    cube_score[i] -= 1.0 / d as f64;
                                }
                                assert_eq!(neighbors[&(a, b)].len(), 0);
                            }
                            let time_rescore = time_rescore.elapsed();
                            if verb {
                                debug!("Rescored in {:.1}s", time_rescore.as_secs_f64());
                            }
                            indet_cubes.push(best_cube);
                        }
                        SolveResponse::Sat => panic!("Unexpected SAT"),
                    }
                }
            }
        } else {
            indet_cubes.push(best_cube);
            break;
        }
    }

    cubes
        .into_iter()
        .enumerate()
        .filter_map(|(i, cube)| {
            if remaining_cubes.contains(&i) || indet_cubes.contains(&i) {
                Some(cube)
            } else {
                None
            }
        })
        .collect()
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

pub fn propcheck_all_trie_via_internal(
    solver: &Cadical,
    vars: &[Var],
    trie: &Trie,
    limit: u64,
    mut out_valid: Option<&mut Vec<Vec<Lit>>>,
) -> u64 {
    assert!(vars.len() < 30);

    // TODO:
    // if (internal->unsat || internal->unsat_constraint) {
    //     std::cout << "Already unsat" << std::endl;
    //     return 0;
    // }

    // Trivial case:
    if vars.is_empty() || trie.is_empty() {
        return 0;
    }

    // Backtrack to 0 level before prop-checking:
    if solver.internal_level() > 0 {
        trace!("Backtracking from level {} to 0", solver.internal_level());
        solver.internal_backtrack(0);
    }

    // Propagate everything that needs to be propagated:
    if !solver.internal_propagate() {
        debug!("Conflict during pre-propagation");
        solver.internal_reset_conflict();
        return 0;
    }

    let mut cube = vec![false; vars.len()];
    let mut total_checked = 0u64;
    let mut total_count = 0u64;

    #[derive(Debug)]
    enum State {
        Descending,
        Ascending,
        Propagating,
    }
    let mut state = State::Descending;

    loop {
        let level = solver.internal_level();
        assert!(level <= vars.len());

        match state {
            State::Descending => {
                if level == vars.len() {
                    if let Some(valid) = &mut out_valid {
                        valid.push(zip_eq(vars, &cube).take(level).map(|(&v, &s)| Lit::new(v, s)).collect_vec());
                    }
                    total_count += 1;
                    if limit > 0 && total_count >= limit {
                        trace!("reached the limit: {} >= {}", total_count, limit);
                        break;
                    }
                    state = State::Ascending;
                } else {
                    if trie.search(&cube[..=level]) == 0 {
                        // Dummy level:
                        solver.internal_assume_decision(0);
                        state = State::Ascending;
                    } else {
                        let v = vars[level].to_external() as i32;
                        let lit = if cube[level] { -v } else { v };
                        let b = solver.internal_val(lit);
                        if b > 0 {
                            // Dummy level:
                            solver.internal_assume_decision(0);
                            state = State::Descending;
                        } else if b < 0 {
                            // Dummy level:
                            solver.internal_assume_decision(0);
                            state = State::Ascending;
                        } else {
                            // Enqueue the literal:
                            solver.internal_assume_decision(lit);
                            state = State::Propagating;
                        }
                    }
                }
            }

            State::Ascending => {
                assert!(level > 0);

                // Find the 1-based index of the last 'false' value in 'cube':
                let mut i = level; // 1-based
                while i > 0 && cube[i - 1] {
                    i -= 1;
                }
                if i == 0 {
                    break;
                }

                // Increment the 'cube':
                assert!(!cube[i - 1]);
                cube[i - 1] = true;
                for j in i..vars.len() {
                    cube[j] = false;
                }

                // Backtrack to the level before `i`:
                solver.internal_backtrack(i - 1);

                // Switch state to descending:
                state = State::Descending;
            }

            State::Propagating => {
                total_checked += 1;
                if !solver.internal_propagate() {
                    // Conflict.
                    solver.internal_reset_conflict();
                    state = State::Ascending;
                } else {
                    // No conflict.
                    state = State::Descending;
                }
            }
        }
    }

    // Post-backtrack to zero level:
    solver.internal_backtrack(0);

    trace!("Checked {} cubes, found {} valid", total_checked, total_count);
    total_count
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;
    use test_log::test;

    use simple_sat::trie::build_trie;

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

    #[test]
    fn test_cadical_propcheck() {
        let mut solver = Cadical::new();

        let x1 = 1;
        let x2 = 2;
        let g1 = 3;
        info!("x1 = {}, x2 = {}, g1 = {}", x1, x2, g1);

        // g1 <=> x1 AND x2
        solver.add_clause([g1, -x1, -x2]);
        solver.add_clause([-g1, x1]);
        solver.add_clause([-g1, x2]);

        // Forbid (x1 AND ~x2)
        solver.add_clause([-x1, x2]);

        info!("vars = {}, clauses = {}", solver.vars(), solver.all_clauses_iter().count());

        // Problem is satisfiable.
        // let res = solver.solve();
        // assert_eq!(res, SolveResult::Sat);

        let vars = vec![x1, x2];

        info!("----------------------");
        let count_tree = solver.propcheck_all_tree(&vars, 0);
        info!("count_tree = {}", count_tree);

        info!("----------------------");
        let count_tree_internal = solver.propcheck_all_tree_via_internal(&vars, 0);
        info!("count_tree_internal = {}", count_tree_internal);

        assert_eq!(count_tree, count_tree_internal);

        info!("----------------------");
        let variables = vars.iter().map(|&v| Var::from_external(v as u32)).collect::<Vec<_>>();
        let cubes = vec![vec![false, false], vec![true, true], vec![true, false], vec![false, true]];
        let trie = build_trie(&cubes);
        let mut valid = Vec::new();
        let count_trie_internal = propcheck_all_trie_via_internal(&solver, &variables, &trie, 0, Some(&mut valid));
        info!("count_trie_internal = {}", count_trie_internal);

        assert_eq!(count_tree_internal, count_trie_internal);
    }
}
