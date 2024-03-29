use log::debug;
use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::slice::Iter;

use clause::Clause;

use crate::lit::Lit;

pub mod clause;
mod parsing;

#[derive(Debug)]
pub struct Cnf {
    pub max_var: usize,
    pub clauses: Vec<Clause>,
}

impl Cnf {
    pub fn iter(&self) -> Iter<'_, Clause> {
        self.clauses.iter()
    }
}

impl Cnf {
    pub fn new() -> Self {
        Self {
            max_var: 0,
            clauses: Vec::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        debug!("Reading CNF from '{}'...", path.as_ref().display());
        parsing::parse_cnf(path)
    }
}

impl Default for Cnf {
    fn default() -> Self {
        Self::new()
    }
}

impl<I> FromIterator<I> for Cnf
where
    I: IntoIterator,
    I::Item: Into<Lit>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut cnf = Self::new();
        for lits in iter.into_iter() {
            let clause = Clause::from_iter(lits);
            cnf.add(clause);
        }
        cnf
    }
}

impl Display for Cnf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        for clause in self.clauses.iter() {
            list.entry(&format_args!("{}", clause));
        }
        list.finish()
    }
}

impl Cnf {
    pub fn add(&mut self, clause: Clause) {
        self.max_var = self.max_var.max(clause.lits.iter().map(|lit| lit.var() as usize).max().unwrap());
        self.clauses.push(clause);
    }
}

impl Extend<Clause> for Cnf {
    fn extend<T: IntoIterator<Item = Clause>>(&mut self, iter: T) {
        for clause in iter {
            self.add(clause);
        }
    }
}

impl crate::op::ops::AddClause for Cnf {
    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        let clause = Clause::from_iter(lits);
        self.add(clause)
    }
}

#[cfg(test)]
mod tests {
    use crate::op::ops::Ops;

    use super::*;

    #[test]
    fn test_cnf() {
        let mut cnf = Cnf::new();
        let a = Lit::from(1i32);
        let b = Lit::from(2i32);
        let c = Lit::from(3i32);
        cnf.imply_and(a, [b, c]);
        assert_eq!(cnf.clauses, [Clause::from_iter([-1, 2]), Clause::from_iter([-1, 3])]);
    }
}
