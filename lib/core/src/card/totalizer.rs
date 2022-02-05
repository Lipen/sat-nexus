//! Totalizer from [[`1`]].
//!
//! [[`1`]] O. Bailleux and Y. Boufkhad, "Efficient CNF encoding of Boolean cardinality constraints," in Principles and Practice of Constraint Programming, 2003, pp. 108–122.
//!
//! [`1`]: https://doi.org/10.1007/978-3-540-45193-8_8

use std::collections::VecDeque;

use itertools::Itertools;

use crate::lit::Lit;
use crate::solver::Solver;

pub struct Totalizer {
    output_vars: Vec<Lit>,
    declared_lower_bound: Option<usize>,
    declared_upper_bound: Option<usize>,
}

impl Totalizer {
    pub fn new(output_vars: Vec<Lit>) -> Self {
        Self {
            output_vars,
            declared_lower_bound: None,
            declared_upper_bound: None,
        }
    }

    pub fn declare<S>(solver: &mut S, input_vars: &[Lit]) -> Self
    where
        S: Solver,
    {
        let mut queue = VecDeque::new();

        for &e in input_vars {
            queue.push_back(vec![e]);
        }

        while queue.len() != 1 {
            let a = queue.pop_front().unwrap();
            let b = queue.pop_front().unwrap();

            let m1 = a.len();
            let m2 = b.len();
            let m = m1 + m2;

            let r = (0..m).map(|_| solver.new_var()).collect_vec();

            for alpha in 0..=m1 {
                for beta in 0..=m2 {
                    let sigma = alpha + beta;
                    let c1 = if sigma == 0 {
                        None
                    } else if alpha == 0 {
                        Some(vec![-b[beta - 1], r[sigma - 1]])
                    } else if beta == 0 {
                        Some(vec![-a[alpha - 1], r[sigma - 1]])
                    } else {
                        Some(vec![-a[alpha - 1], -b[beta - 1], r[sigma - 1]])
                    };
                    let c2 = if sigma == m {
                        None
                    } else if alpha == m1 {
                        Some(vec![b[beta], -r[sigma]])
                    } else if beta == m2 {
                        Some(vec![a[alpha], -r[sigma]])
                    } else {
                        Some(vec![a[alpha], b[beta], -r[sigma]])
                    };

                    if let Some(c) = c1 {
                        solver.add_clause(c);
                    }
                    if let Some(c) = c2 {
                        solver.add_clause(c);
                    }
                }
            }

            queue.push_back(r);
        }

        let output_vars = queue.pop_front().unwrap();
        Totalizer::new(output_vars)
    }

    pub fn declare_upper_bound_less_than<S>(&mut self, solver: &mut S, new_ub: usize)
    where
        S: Solver,
    {
        if let Some(cur_ub) = self.declared_upper_bound {
            assert!(
                new_ub < cur_ub,
                "New upper bound must be less than the current one (new_ub = {}, cur_ub = {})",
                new_ub,
                cur_ub
            );
        }

        self.declare_comparator_less_than(solver, new_ub);
    }

    pub fn declare_upper_bound_less_than_or_equal<S>(&mut self, solver: &mut S, new_ub: usize)
    where
        S: Solver,
    {
        self.declare_upper_bound_less_than(solver, new_ub + 1);
    }

    pub fn declare_lower_bound_greater_than<S>(&mut self, solver: &mut S, new_lb: usize)
    where
        S: Solver,
    {
        self.declare_lower_bound_greater_than_or_equal(solver, new_lb + 1);
    }

    pub fn declare_lower_bound_greater_than_or_equal<S>(&mut self, solver: &mut S, new_lb: usize)
    where
        S: Solver,
    {
        if let Some(cur_lb) = self.declared_lower_bound {
            assert!(
                new_lb >= cur_lb,
                "New lower bound must be greater or equal to the current one (new_lb = {}, cur_lb = {})",
                new_lb,
                cur_lb
            );
        }

        self.declare_comparator_greater_than_or_equal(solver, new_lb);
    }

    fn declare_comparator_less_than<S>(&mut self, solver: &mut S, ub: usize)
    where
        S: Solver,
    {
        assert!(ub <= self.output_vars.len());

        let max = self
            .declared_upper_bound
            .replace(ub)
            .unwrap_or(self.output_vars.len());
        for i in (ub..=max).rev() {
            // Note: totalizer is 0-based, but all params are naturally 1-based
            solver.add_clause([-self.output_vars[i - 1]]);
        }
    }

    fn declare_comparator_greater_than_or_equal<S>(&mut self, solver: &mut S, lb: usize)
    where
        S: Solver,
    {
        assert!(lb >= 1);

        let min = self.declared_lower_bound.replace(lb).unwrap_or(1);
        for i in min..=lb {
            // Note: totalizer is 0-based, but all params are naturally 1-based
            solver.add_clause([self.output_vars[i - 1]]);
        }
    }
}
