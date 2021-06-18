use itertools::Itertools;

use crate::ipasir::SolveResponse;
use crate::solver::GenericSolver;
use crate::types::Lit;

impl<S> AllSat for S where S: GenericSolver + ?Sized {}

pub trait AllSat: GenericSolver {
    fn all_sat<T, F>(&mut self, f: F) -> AllSolutionsIter<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut Self) -> T,
    {
        // If no essential vars were passed, then *all* variables are essential!
        let essential = (1..=self.num_vars()).map_into::<Lit>().collect();
        self.all_sat_essential(essential, f)
    }

    fn all_sat_essential<T, F>(&mut self, essential: Vec<Lit>, f: F) -> AllSolutionsIter<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut Self) -> T,
    {
        AllSolutionsIter::new(self, f, essential)
    }
}

pub struct AllSolutionsIter<'a, S, F>
where
    S: GenericSolver,
{
    solver: &'a mut S,
    callback: F,
    essential: Vec<Lit>,
    refutation: Option<Vec<Lit>>,
}

impl<'a, S, F> AllSolutionsIter<'a, S, F>
where
    S: GenericSolver,
{
    fn new(solver: &'a mut S, callback: F, essential: Vec<Lit>) -> Self {
        Self {
            solver,
            callback,
            essential,
            refutation: None,
        }
    }
}

impl<'a, T, S, F> Iterator for AllSolutionsIter<'a, S, F>
where
    S: GenericSolver,
    F: FnMut(&mut S) -> T,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(refutation) = self.refutation.take() {
            // Ban the solution
            self.solver.add_clause(refutation);
        }

        if let SolveResponse::Sat = self.solver.solve() {
            // Build the refutation
            self.refutation = Some(
                self.essential
                    .iter()
                    .map(|&x| if self.solver.val(x).bool() { -x } else { x })
                    .collect_vec(),
            );

            // Call the callback in the SAT state
            Some((self.callback)(self.solver))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::solver::wrap::WrappedIpasirSolver;

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
