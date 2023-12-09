use std::collections::HashMap;

use itertools::Itertools;
use log::debug;

pub use _pyeda::*;
use simple_sat::lit::Lit;
use simple_sat::utils::DisplaySlice;
use simple_sat::var::Var;

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

    debug!("derive_clauses(hard = [{}])", hard.iter().map(|c| DisplaySlice(c)).join(", "));

    let mut derived_clauses = Vec::new();

    // count :: {Var: (pos, neg)}
    let mut count = HashMap::<Var, (u64, u64)>::new();
    for cube in hard.iter() {
        for &lit in cube.iter() {
            let e = count.entry(lit.var()).or_default();
            if lit.negated() {
                (*e).1 += 1;
            } else {
                (*e).0 += 1;
            }
        }
    }
    for (&var, &(pos, neg)) in count.iter() {
        debug!("Count (pos/neg) for {} is {} / {}", var, pos, neg);
    }
    for (&var, &(pos, neg)) in count.iter() {
        if pos == 0 {
            debug!("variable {} is never positive", var);
            derived_clauses.push(vec![Lit::new(var, true)]);
        }
        if neg == 0 {
            debug!("variable {} is never negative", var);
            derived_clauses.push(vec![Lit::new(var, false)]);
        }
    }

    // count_pair :: {(a, b): (+a+b, +a-b, -a+b, -a-b)}
    let mut count_pair = HashMap::<(Var, Var), (u64, u64, u64, u64)>::new();
    for cube in hard.iter() {
        for i in 0..cube.len() {
            let a = cube[i];
            if count[&a.var()].0 == 0 || count[&a.var()].1 == 0 {
                continue;
            }
            for j in (i + 1)..cube.len() {
                let b = cube[j];
                if count[&b.var()].0 == 0 || count[&b.var()].1 == 0 {
                    continue;
                }
                let (a, b) = if a.index() > b.index() { (b, a) } else { (a, b) };
                let e = count_pair.entry((a.var(), b.var())).or_default();
                match (a.negated(), b.negated()) {
                    (false, false) => (*e).0 += 1, // pos-pos
                    (false, true) => (*e).1 += 1,  // pos-neg
                    (true, false) => (*e).2 += 1,  // neg-pos
                    (true, true) => (*e).3 += 1,   // neg-neg
                }
            }
        }
    }
    for (&(a, b), &(pp, pn, np, nn)) in count_pair.iter() {
        debug!("Count (pp/pn/np/nn) for {}-{} is {} / {} / {} / {}", a, b, pp, pn, np, nn);
    }
    for (&(a, b), &(pp, pn, np, nn)) in count_pair.iter() {
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

    derived_clauses
}
