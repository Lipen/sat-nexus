use std::fmt::Write as _;

use sat_nexus_core::map::Map;

pub struct CnfEncoder {
    pub num_vars: usize,
    pub clauses: Vec<Vec<i32>>,
}

impl CnfEncoder {
    pub fn new(num_vars: usize) -> Self {
        Self {
            num_vars,
            clauses: Vec::new(),
        }
    }
}

impl Default for CnfEncoder {
    fn default() -> Self {
        Self::new(0)
    }
}

impl CnfEncoder {
    pub fn new_var(&mut self) -> i32 {
        self.num_vars += 1;
        self.num_vars as i32
    }

    pub fn add_clause(&mut self, clause: Vec<i32>) {
        assert!(!clause.is_empty(), "clause must not be empty");
        self.clauses.push(clause);
    }

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

// Variables
impl CnfEncoder {
    pub fn new_direct<T>(&mut self, values: Vec<T>) -> Map<T, i32> {
        let variables = values.iter().map(|_| self.new_var()).collect();
        Map::new(values, variables)
    }
}

// Constraints
impl CnfEncoder {
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
