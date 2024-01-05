use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::{Duration, Instant};

use flate2::read::GzDecoder;
use itertools::{join, Itertools};
use tracing::trace;

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

pub fn parse_dimacs<P>(path: P) -> impl Iterator<Item = Vec<Lit>>
where
    P: AsRef<Path>,
{
    read_maybe_gzip(path).unwrap().lines().flatten().filter_map(|line| {
        if line.is_empty() {
            trace!("Skipping empty line");
            None
        } else if line.starts_with('c') {
            trace!("Skipping comment '{}'", line);
            None
        } else if line.starts_with('p') {
            trace!("Skipping header '{}'", line);
            None
        } else {
            let lits = parse_dimacs_clause(&line);
            Some(lits)
        }
    })
}

pub fn parse_dimacs_clause(s: &str) -> Vec<Lit> {
    let clause = s
        .split_whitespace()
        .map(|x| x.parse::<i32>().expect("could not parse lit in clause"))
        .collect_vec();
    let (&last, lits) = clause.split_last().unwrap();
    assert_eq!(last, 0, "last lit in clause must be 0");
    lits.iter().map(|&lit| Lit::from_external(lit)).collect()
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

pub fn cmp_f64(a: f64, b: f64) -> Ordering {
    PartialOrd::partial_cmp(&a, &b).unwrap()
}

pub struct DisplaySlice<'a, T>(pub &'a [T])
where
    &'a T: Display;

impl<'a, T> Display for DisplaySlice<'a, T>
where
    &'a T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", join(self.0, ", "))
    }
}

pub struct DisplayIter<I>(pub I);

impl<I> Display for DisplayIter<I>
where
    I: IntoIterator + Copy,
    I::Item: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", join(self.0, ", "))
    }
}
