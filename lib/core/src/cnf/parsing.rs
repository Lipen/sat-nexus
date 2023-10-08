use std::fs::File;
use std::io::Read;
use std::path::Path;

use dimacs::{parse_dimacs, Instance, Sign};

use crate::cnf::Cnf;

pub fn parse_cnf<P: AsRef<Path>>(path: P) -> Cnf {
    let mut f = File::open(path).expect("Could not open file");
    let mut cnf = String::new();
    f.read_to_string(&mut cnf).expect("Could not read CNF");
    let instance = parse_dimacs(&cnf).expect("Could not parse DIMACS");
    match instance {
        Instance::Cnf { num_vars, clauses } => Cnf {
            max_var: num_vars as usize,
            clauses: clauses
                .iter()
                .map(|c| {
                    c.lits()
                        .iter()
                        .map(|x| {
                            let v = x.var().to_u64() as i32;
                            match x.sign() {
                                Sign::Pos => v,
                                Sign::Neg => -v,
                            }
                        })
                        .collect()
                })
                .collect(),
        },
        _ => panic!("Bad instance"),
    }
}
