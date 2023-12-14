use std::collections::HashSet;

use itertools::{Itertools, MultiProduct};

use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::var::Var;

pub fn parse_comma_separated_intervals(input: &str) -> Vec<usize> {
    let mut result = Vec::new();
    for part in input.split(',') {
        let range_parts: Vec<&str> = part.splitn(2, "-").collect();
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

pub fn concat_cubes(a: Vec<Lit>, b: Vec<Lit>) -> Vec<Lit> {
    let mut r = HashSet::new();
    r.extend(a);
    r.extend(b);
    let mut r = r.into_iter().collect_vec();
    r.sort_by_key(|lit| lit.var().0);
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

#[cfg(test)]
mod tests {
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
}
