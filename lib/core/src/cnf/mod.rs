use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use std::slice::Iter;

use clause::Clause;

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
        parsing::parse_cnf(path)
    }
}

impl Default for Cnf {
    fn default() -> Self {
        Self::new()
    }
}

impl<I> From<I> for Cnf
where
    I: IntoIterator,
    I::Item: Into<Clause>,
{
    fn from(iter: I) -> Self {
        let mut cnf = Self::new();
        for clause in iter.into_iter() {
            cnf.add_clause(clause)
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
    pub fn add_clause(&mut self, clause: impl Into<Clause>) {
        let clause = clause.into();
        self.max_var = self.max_var.max(clause.lits.iter().map(|lit| lit.var() as usize).max().unwrap());
        self.clauses.push(clause);
    }
}
