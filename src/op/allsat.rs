use itertools::Itertools;

use crate::core::lit::Lit;
use crate::core::solver::{SolveResponse, Solver};

impl<S> AllSat for S where S: Solver + ?Sized {}

pub trait AllSat: Solver {
    fn all_sat<T, F>(&mut self, f: F) -> AllSolutionsIter<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut Self) -> T,
    {
        // If no essential vars were passed, then *all* variables are essential!
        let essential = 1..=self.num_vars();
        self.all_sat_essential(essential, f)
    }

    fn all_sat_essential<I, L, T, F>(&mut self, essential: I, f: F) -> AllSolutionsIter<Self, F>
    where
        Self: Sized,
        I: IntoIterator<Item = L>,
        L: Into<Lit>,
        F: FnMut(&mut Self) -> T,
    {
        AllSolutionsIter::new(self, f, essential)
    }

    fn build_refutation(&self, essential: &[Lit]) -> Vec<Lit> {
        essential
            .iter()
            .map(|&x| if self.val(x).bool() { -x } else { x })
            .collect_vec()
    }
}

pub struct AllSolutionsIter<'s, S, F>
where
    S: Solver,
{
    solver: &'s mut S,
    callback: F,
    essential: Vec<Lit>,
    refutation: Option<Vec<Lit>>,
}

impl<'s, S, F> AllSolutionsIter<'s, S, F>
where
    S: Solver,
{
    fn new<I, L>(solver: &'s mut S, callback: F, essential: I) -> Self
    where
        I: IntoIterator<Item = L>,
        L: Into<Lit>,
    {
        let essential = essential.into_iter().map_into::<Lit>().collect_vec();
        Self {
            solver,
            callback,
            essential,
            refutation: None,
        }
    }
}

impl<'s, T, S, F> Iterator for AllSolutionsIter<'s, S, F>
where
    S: Solver,
    F: FnMut(&mut S) -> T,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(refutation) = self.refutation.take() {
            // Ban the solution
            self.solver.add_clause(refutation);
        }

        if matches!(self.solver.solve(), SolveResponse::Sat) {
            // Build the refutation
            self.refutation = Some(self.solver.build_refutation(&self.essential));

            // Call the callback in the SAT state
            Some((self.callback)(self.solver))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::solver::wrap_ipasir::WrappedIpasirSolver;

    use super::*;

    #[test]
    fn all_solutions_5vars() {
        let mut solver = WrappedIpasirSolver::new_cadical();

        let n = 5;
        let _lits = solver.new_var_vec(n);
        assert_eq!(solver.num_vars(), n);

        // Note: add the redundant clause `(x or -x)`, where x is the last used variable,
        //  in order to force the "allocation" of all variables inside the solver.
        solver.add_clause([Lit::from(n), -Lit::from(n)]);

        let num_solutions = solver.all_sat(|_| ()).count();
        assert_eq!(num_solutions, 32);
    }

    #[test]
    fn all_solutions_essential_3of5vars() {
        let mut solver = WrappedIpasirSolver::new_cadical();

        let n = 5;
        let lits = solver.new_var_vec(n);
        assert_eq!(solver.num_vars(), n);

        // Note: add the redundant clause `(x or -x)`, where x is the last used variable,
        //  in order to force the "allocation" of all variables inside the solver.
        solver.add_clause([Lit::from(n), -Lit::from(n)]);

        let k = 3;
        let essential = lits[0..k].to_vec();
        let num_solutions = solver.all_sat_essential(essential, |_| ()).count();
        assert_eq!(num_solutions, 8);
    }
}
