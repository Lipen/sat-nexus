use itertools::Itertools;

use crate::lit::Lit;
use crate::solver::{SolveResponse, Solver};

impl<S> AllSat for S where S: Solver {}

pub trait AllSat: Solver {
    fn all_sat<T, F>(&mut self, f: F) -> AllSolutionsIter<Self, F>
    where
        F: FnMut(&mut Self) -> T,
    {
        // If no essential vars were passed, then *all* variables are essential!
        let essential = 1..=self.num_vars();
        self.all_sat_essential(essential, f)
    }

    fn all_sat_essential<I, T, F>(&mut self, essential: I, f: F) -> AllSolutionsIter<Self, F>
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
        F: FnMut(&mut Self) -> T,
    {
        AllSolutionsIter::new(self, f, essential)
    }

    fn build_refutation(&self, essential: &[Lit]) -> Vec<Lit> {
        essential
            .iter()
            .map(|&x| if self.value(x).bool() { -x } else { x })
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
    fn new<I>(solver: &'s mut S, callback: F, essential: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
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
