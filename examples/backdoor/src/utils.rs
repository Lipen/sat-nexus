use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter};
use std::iter::zip;
use std::path::Path;

use cadical::statik::Cadical;
use itertools::{Itertools, MultiProduct};
use log::debug;

use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
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

pub fn partition_tasks(variables: &[Var], solver: &mut Solver) -> (Vec<Vec<Lit>>, Vec<Vec<Lit>>) {
    partition_tasks_with(variables, |cube| solver.propcheck(cube))
}

pub fn partition_tasks_cadical(variables: &[Var], solver: &Cadical) -> (Vec<Vec<Lit>>, Vec<Vec<Lit>>) {
    partition_tasks_with(variables, |cube| {
        let cube: Vec<i32> = cube.iter().map(|lit| lit.to_external()).collect();
        solver.propcheck(&cube)
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

pub fn determine_vars_pool(solver: &Solver, allowed_vars: &Option<String>, banned_vars: &Option<String>) -> Vec<Var> {
    // Determine the set of variables encountered in CNF:
    let mut encountered_vars = HashSet::new();
    // TODO: note that `clauses_iter` does not contain units!
    for clause in solver.clauses_iter() {
        for lit in clause.iter() {
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
