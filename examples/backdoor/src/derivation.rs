use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use itertools::Itertools;
use log::{debug, trace};

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

    let mut derived_clauses = Vec::new();

    // count :: [(pos,neg)]
    let n = hard[0].len();
    let mut count: Vec<(u64, u64)> = vec![(0, 0); n];

    let pb = ProgressBar::new(hard.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("units");
    for cube in hard.iter().progress_with(pb) {
        for (i, &lit) in cube.iter().enumerate() {
            if lit.negated() {
                count[i].1 += 1;
            } else {
                count[i].0 += 1;
            }
        }
    }

    // for i in 0..n {
    //     let var = hard[0][i].var();
    //     let (pos, neg) = count[i];
    //     debug!("Count (pos/neg) for {} is {} / {}", var, pos, neg);
    // }
    for i in 0..n {
        let var = hard[0][i].var();
        let (pos, neg) = count[i];
        if pos == 0 {
            debug!("variable {} is never positive", var);
            derived_clauses.push(vec![Lit::new(var, true)]);
        }
        if neg == 0 {
            debug!("variable {} is never negative", var);
            derived_clauses.push(vec![Lit::new(var, false)]);
        }
    }

    // count_pair :: [(+a+b, +a-b, -a+b, -a-b)]
    let n = hard[0].len();
    let mut count_pair: Vec<(u64, u64, u64, u64)> = vec![(0, 0, 0, 0); n * n];

    let pb = ProgressBar::new(hard.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("binary");
    for cube in hard.iter().progress_with(pb) {
        for i in 0..n {
            if count[i].0 == 0 || count[i].1 == 0 {
                // Skip units
                continue;
            }
            for j in (i + 1)..n {
                if count[j].0 == 0 || count[j].1 == 0 {
                    // Skip units
                    continue;
                }
                let e = &mut count_pair[i * n + j];
                match (cube[i].negated(), cube[j].negated()) {
                    (false, false) => (*e).0 += 1, // pos-pos
                    (false, true) => (*e).1 += 1,  // pos-neg
                    (true, false) => (*e).2 += 1,  // neg-pos
                    (true, true) => (*e).3 += 1,   // neg-neg
                };
            }
        }
    }

    // for i in 0..n {
    //     if count[i].0 == 0 || count[i].1 == 0 {
    //         // Skip units
    //         continue;
    //     }
    //     for j in (i + 1)..n {
    //         if count[j].0 == 0 || count[j].1 == 0 {
    //             // Skip units
    //             continue;
    //         }
    //         let (pp, pn, np, nn) = count_pair[i * n + j];
    //         let a = hard[0][i].var();
    //         let b = hard[0][j].var();
    //         debug!("Count (pp/pn/np/nn) for {}-{} is {} / {} / {} / {}", a, b, pp, pn, np, nn);
    //     }
    // }
    for i in 0..n {
        if count[i].0 == 0 || count[i].1 == 0 {
            // Skip units
            continue;
        }
        for j in (i + 1)..n {
            if count[j].0 == 0 || count[j].1 == 0 {
                // Skip units
                continue;
            }
            let (pp, pn, np, nn) = count_pair[i * n + j];
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
    }

    derived_clauses
}
