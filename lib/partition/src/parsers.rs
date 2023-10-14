use std::fmt::Debug;
use std::ops::RangeInclusive;
use std::str::FromStr;

pub fn parse_input_variables(input: &str) -> Vec<u32> {
    let mut result = Vec::new();
    for part in input.split(',') {
        result.extend(parse_interval::<u32>(part));
    }
    result
}

pub fn parse_interval<T>(input: &str) -> RangeInclusive<T>
where
    T: FromStr + Copy,
    <T as FromStr>::Err: Debug,
{
    let range_parts: Vec<&str> = input.splitn(2, "-").collect();
    if range_parts.len() == 2 {
        let start = range_parts[0].parse().unwrap();
        let end = range_parts[1].parse().unwrap();
        start..=end
    } else {
        let single = input.parse().unwrap();
        single..=single
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
