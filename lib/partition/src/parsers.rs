use std::fmt::{Display, Formatter};

pub fn parse_input_variables(input: &str) -> Vec<usize> {
    let mut result = Vec::new();
    for part in input.split(',') {
        match parse_interval(part) {
            Interval::Single(x) => {
                result.push(x);
            }
            Interval::Interval(low, high) => {
                if low <= high {
                    result.extend(low..=high);
                } else {
                    result.extend((high..=low).rev());
                }
            }
        }
    }
    result
}

pub fn parse_intervals(input: &str) -> Vec<usize> {
    let mut result = Vec::new();
    for part in input.split(',') {
        match parse_interval(part) {
            Interval::Single(x) => {
                result.push(x);
            }
            Interval::Interval(low, high) => {
                if low <= high {
                    result.extend(low..=high);
                } else {
                    result.extend((high..=low).rev());
                }
            }
        }
    }
    result.sort();
    result.dedup();
    result
}

pub enum Interval<T> {
    Single(T),
    Interval(T, T),
}

impl<T> Display for Interval<T>
where
    for<'a> &'a T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Interval::Single(x) => write!(f, "{}", x),
            Interval::Interval(low, high) => write!(f, "{}-{}", low, high),
        }
    }
}

pub fn parse_interval(input: &str) -> Interval<usize> {
    let range_parts: Vec<&str> = input.splitn(2, "-").collect();
    if range_parts.len() == 2 {
        let start = range_parts[0].parse().unwrap();
        let end = range_parts[1].parse().unwrap();
        Interval::Interval(start, end)
    } else {
        let single = input.parse().unwrap();
        Interval::Single(single)
    }
}

pub fn parse_integer_maybe_power(input: &str) -> usize {
    let pow_parts: Vec<&str> = input.splitn(2, '^').collect();
    if pow_parts.len() == 2 {
        let base: usize = pow_parts[0].parse().unwrap();
        let power = pow_parts[1].parse().unwrap();
        base.pow(power)
    } else {
        input.parse().unwrap()
    }
}
