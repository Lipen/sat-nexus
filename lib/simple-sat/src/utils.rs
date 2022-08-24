use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

use flate2::read::GzDecoder;
use itertools::Itertools;

use crate::lit::Lit;

pub fn get_extension(path: &Path) -> Option<&str> {
    path.extension().and_then(OsStr::to_str)
}

pub fn read_maybe_gzip<P>(path: P) -> io::Result<Box<dyn BufRead>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let file = File::open(path)?;
    let capacity = 128 * 1024;
    if get_extension(path).unwrap() == "gz" {
        Ok(Box::new(BufReader::with_capacity(capacity, GzDecoder::new(file))))
    } else {
        Ok(Box::new(BufReader::with_capacity(capacity, file)))
    }
}

pub fn parse_dimacs_clause(s: &str) -> Vec<Lit> {
    let clause = s
        .split_whitespace()
        .map(|x| x.parse::<i32>().expect("could not parse lit in clause"))
        .collect_vec();
    let (&last, lits) = clause.split_last().unwrap();
    debug_assert_eq!(last, 0, "last lit in clause must be 0");
    lits.iter().map(|&lit| Lit::from_lit(lit)).collect()
}

pub fn luby(y: f64, mut x: u32) -> f64 {
    // Find the finite subsequence that contains index 'x',
    // and the size of that subsequence:
    let mut size = 1;
    let mut seq = 0;

    while size < x + 1 {
        seq += 1;
        size = 2 * size + 1;
    }

    while size - 1 != x {
        size = (size - 1) >> 1;
        seq -= 1;
        x %= size;
    }

    y.powi(seq)
}

pub fn measure_time<T, F>(f: F) -> (Duration, T)
where
    F: FnOnce() -> T,
{
    let time_start = Instant::now();
    let result = f();
    (time_start.elapsed(), result)
}
