use std::fmt::Write as _;

use crate::domainvar::DomainVar;

pub struct SatEncoder {
    pub num_vars: usize,
    pub clauses: Vec<Vec<i32>>,
}

impl SatEncoder {
    pub fn new(num_vars: usize) -> Self {
        Self {
            num_vars,
            clauses: Vec::new(),
        }
    }
}

impl Default for SatEncoder {
    fn default() -> Self {
        Self::new(0)
    }
}

impl SatEncoder {
    /// Generate a new unique variable ID for SAT encoding.
    pub fn new_var(&mut self) -> i32 {
        self.num_vars += 1;
        self.num_vars as i32
    }

    /// Add a clause to the SAT encoding in CNF format.
    pub fn add_clause(&mut self, clause: Vec<i32>) {
        assert!(!clause.is_empty(), "clause must not be empty");
        self.clauses.push(clause);
    }

    /// Convert the SAT encoding to DIMACS format for the SAT solver.
    pub fn to_dimacs(&self) -> String {
        let mut output = String::new();
        writeln!(output, "p cnf {} {}", self.num_vars, self.clauses.len()).unwrap();
        for clause in &self.clauses {
            for lit in clause {
                write!(output, "{} ", lit).unwrap();
            }
            writeln!(output, "0").unwrap();
        }
        output
    }
}

impl SatEncoder {
    pub fn exactly_one(&mut self, vars: &[i32]) {
        self.at_least_one(vars);
        self.at_most_one(vars);
    }

    pub fn at_least_one(&mut self, vars: &[i32]) {
        assert!(!vars.is_empty(), "vars must not be empty in AtLeastOne constraint");
        self.add_clause(vars.to_vec());
    }

    pub fn at_most_one(&mut self, vars: &[i32]) {
        for (i, &var1) in vars.iter().enumerate() {
            for &var2 in vars.iter().skip(i + 1) {
                self.add_clause(vec![-var1, -var2]);
            }
        }
    }
}

impl SatEncoder {
    pub fn new_direct<T>(&mut self, values: Vec<T>) -> DomainVar<T, i32> {
        let variables = values.iter().map(|_| self.new_var()).collect();
        DomainVar::new(values, variables)
    }
}
