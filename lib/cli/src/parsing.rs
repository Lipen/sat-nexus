#![allow(dead_code)]

use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

use itertools::Itertools;

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn parse_dimacs_clause(s: &str) -> Vec<i32> {
    let clause = s
        .split_whitespace()
        .map(|x| {
            x.parse::<i32>()
                .unwrap_or_else(|e| panic!("Could not parse lit from line '{}': {}", x, e))
        })
        .collect_vec();
    let (&last, lits) = clause.split_last().unwrap();
    debug_assert_eq!(last, 0, "last lit in clause must be 0");
    lits.to_vec()
}
