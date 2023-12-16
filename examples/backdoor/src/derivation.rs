use std::collections::HashMap;

use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::{debug, trace};
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

pub use _pyeda::*;
use simple_sat::lit::Lit;
use simple_sat::utils::DisplaySlice;

#[cfg(feature = "pyeda")]
mod _pyeda {
    use itertools::Itertools;

    use pyeda::backdoor_to_clauses;
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
}

#[cfg(not(feature = "pyeda"))]
mod _pyeda {
    use simple_sat::lit::Lit;

    pub fn minimize_backdoor(_cubes: &[Vec<Lit>]) -> Vec<Vec<Lit>> {
        panic!("Use 'pyeda' feature!")
    }
}

pub fn derive_clauses(hard: &[Vec<Lit>]) -> Vec<Vec<Lit>> {
    // Note: currently, derives only units and binary clauses.

    trace!("derive_clauses(hard = [{}])", hard.iter().map(|c| DisplaySlice(c)).join(", "));

    // for cube in hard.iter() {
    //     assert_eq!(cube.len(), hard[0].len());
    //     assert!(std::iter::zip(cube, &hard[0]).all(|(a, b)| a.var() == b.var()));
    // }

    let n = hard[0].len();
    let mut derived_clauses = Vec::new();

    let pb = ProgressBar::new(n as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("units");
    // count_unit :: {i: (pos,neg)}
    let count_unit: HashMap<usize, (u64, u64)> = (0..n)
        .into_par_iter()
        .progress_with(pb)
        .map(|i| {
            let mut pos = 0;
            let mut neg = 0;
            for cube in hard.iter() {
                if cube[i].negated() {
                    neg += 1;
                } else {
                    pos += 1;
                }
            }
            (i, (pos, neg))
        })
        .collect();

    for (&i, &(pos, neg)) in count_unit.iter() {
        let var = hard[0][i].var();
        debug!("Count (pos/neg) for {} is {} / {}", var, pos, neg);
    }
    for (&i, &(pos, neg)) in count_unit.iter() {
        let var = hard[0][i].var();
        if pos == 0 {
            debug!("variable {} is never positive", var);
            derived_clauses.push(vec![Lit::new(var, true)]);
        }
        if neg == 0 {
            debug!("variable {} is never negative", var);
            derived_clauses.push(vec![Lit::new(var, false)]);
        }
    }

    let pb = ProgressBar::new((n * (n - 1) / 2) as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("binary");
    // count_binary :: {(i,j): (++, +-, -+, --)}
    let count_binary: HashMap<(usize, usize), (u64, u64, u64, u64)> = (0..n)
        .tuple_combinations()
        .par_bridge()
        .progress_with(pb)
        .filter_map(|(i, j)| {
            // Skip units:
            let (pos, neg) = count_unit[&i];
            if pos == 0 || neg == 0 {
                return None;
            }
            let (pos, neg) = count_unit[&j];
            if pos == 0 || neg == 0 {
                return None;
            }

            let (mut pp, mut pn, mut np, mut nn) = (0, 0, 0, 0);
            for cube in hard.iter() {
                match (cube[i].negated(), cube[j].negated()) {
                    (false, false) => pp += 1, // pos-pos
                    (false, true) => pn += 1,  // pos-neg
                    (true, false) => np += 1,  // neg-pos
                    (true, true) => nn += 1,   // neg-neg
                };
            }
            Some(((i, j), (pp, pn, np, nn)))
        })
        .collect();

    for (&(i, j), &(pp, pn, np, nn)) in count_binary.iter() {
        let a = hard[0][i].var();
        let b = hard[0][j].var();
        debug!("Count (pp/pn/np/nn) for {}-{} is {} / {} / {} / {}", a, b, pp, pn, np, nn);
    }
    for (&(i, j), &(pp, pn, np, nn)) in count_binary.iter() {
        let a = hard[0][i].var();
        let b = hard[0][j].var();
        if pp == 0 {
            debug!(
                "pair {}-{} is never pos-pos |= clause ({}, {})",
                a,
                b,
                Lit::new(a, true),
                Lit::new(b, true)
            );
            derived_clauses.push(vec![Lit::new(a, true), Lit::new(b, true)]);
        }
        if pn == 0 {
            debug!(
                "pair {}-{} is never pos-neg |= clause ({}, {})",
                a,
                b,
                Lit::new(a, true),
                Lit::new(b, false)
            );
            derived_clauses.push(vec![Lit::new(a, true), Lit::new(b, false)]);
        }
        if np == 0 {
            debug!(
                "pair {}-{} is never neg-pos |= clause ({}, {})",
                a,
                b,
                Lit::new(a, false),
                Lit::new(b, true)
            );
            derived_clauses.push(vec![Lit::new(a, false), Lit::new(b, true)]);
        }
        if nn == 0 {
            debug!(
                "pair {}-{} is never neg-neg |= clause ({}, {})",
                a,
                b,
                Lit::new(a, false),
                Lit::new(b, false)
            );
            derived_clauses.push(vec![Lit::new(a, false), Lit::new(b, false)]);
        }
    }

    // Sort each clause:
    for clause in derived_clauses.iter_mut() {
        clause.sort_by_key(|lit| lit.var().0);
    }
    // Sort all clauses:
    derived_clauses.sort_by_key(|clause| (clause.len(), clause.iter().map(|lit| lit.var().0).collect_vec()));

    derived_clauses
}
