use std::borrow::Borrow;

use itertools::Itertools;

use crate::parsers::Interval;

pub fn num2bits(x: usize, n: usize) -> Vec<bool> {
    assert!(x < (1 << n), "Number {} must be less than 2^{}={}", x, n, 1 << n);
    let mut bits = vec![false; n];

    for i in 0..n.min(32) {
        bits[n - i - 1] = (x >> i) & 1 != 0;
    }

    bits
}

pub fn bits2str(bits: &[bool]) -> String {
    bits.iter().map(|&bit| if bit { '1' } else { '0' }).collect()
}

pub fn bits2num(bits: &[bool]) -> u32 {
    assert!(bits.len() <= 32);

    let mut result = 0;
    let mut shift = 0;

    for &bit in bits.iter().rev() {
        if bit {
            result |= 1 << shift;
        }
        shift += 1;
    }

    result
}

pub fn to_dimacs(clause: &[i32]) -> String {
    let mut s = String::new();
    for &lit in clause {
        s += &format!("{} ", lit);
    }
    s += "0";
    s
}

pub fn is_power_of_two(x: usize) -> bool {
    x & (x - 1) == 0
}

pub fn mean(data: &[f64]) -> f64 {
    let count = data.len();
    if count > 0 {
        let sum = data.iter().sum::<f64>();
        sum / (count as f64)
    } else {
        f64::NAN
    }
}

pub fn mean_iter<I>(iter: I) -> f64
where
    I: IntoIterator,
    I::Item: Borrow<f64>,
{
    let mut count: usize = 0;
    let mut mean: f64 = 0.0;
    for x in iter.into_iter() {
        count += 1;
        mean += (x.borrow() - mean) / (count as f64);
    }
    if count > 0 {
        mean
    } else {
        f64::NAN
    }
}

pub fn median(data: &[f64]) -> f64 {
    let count = data.len();
    if count == 0 {
        return f64::NAN;
    }
    if count == 1 {
        return data[0];
    }
    if count == 2 {
        return (data[0] + data[1]) / 2.0;
    }
    let sorted = data.iter().copied().sorted_by(|x, y| x.partial_cmp(y).unwrap()).collect_vec();
    let mid = count / 2;
    if count % 2 == 0 {
        // Even length: median is the mean of two near-middle elements
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        // Odd length: median is the middle element
        sorted[mid]
    }
}

/// Sample variance using [Welford's online algorithm](https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Welford's_online_algorithm).
pub fn variance_iter<I>(iter: I) -> f64
where
    I: IntoIterator,
    I::Item: Borrow<f64>,
{
    let mut count: usize = 0;
    let mut mean: f64 = 0.0;
    let mut m2: f64 = 0.0;

    for x in iter.into_iter() {
        let x: f64 = *x.borrow();
        count += 1;
        let delta = x - mean;
        mean += delta / count as f64;
        let delta2 = x - mean;
        m2 += delta * delta2;
    }

    if count > 1 {
        // Bessel's correction for unbiased sample variance:
        m2 / (count - 1) as f64
    } else {
        f64::NAN
    }
}

pub fn std_deviation_iter<I>(iter: I) -> f64
where
    I: IntoIterator,
    I::Item: Borrow<f64>,
{
    variance_iter(iter).sqrt()
}

pub fn variance(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return f64::NAN;
    }
    let avg = mean(data);
    let sum = data
        .iter()
        .map(|&x| {
            let diff = x - avg;
            diff * diff
        })
        .sum::<f64>();
    // Bessel's correction for unbiased sample variance:
    sum / (data.len() - 1) as f64
}

pub fn std_deviation(data: &[f64]) -> f64 {
    variance(data).sqrt()
}

pub fn mean_absolute_deviation(data: &[f64]) -> f64 {
    let avg = mean(data);
    if avg.is_nan() {
        return f64::NAN;
    }
    let deviations = data.iter().map(|&x| (x - avg).abs()).collect_vec();
    mean(&deviations)
}

pub fn median_absolute_deviation(data: &[f64]) -> f64 {
    let med = median(data);
    if med.is_nan() {
        return f64::NAN;
    }
    let deviations = data.iter().map(|&x| (x - med).abs()).collect_vec();
    median(&deviations)
}

pub fn zscore(data: &[f64]) -> Vec<f64> {
    if data.is_empty() {
        return Vec::new();
    }
    let avg = mean(data);
    let sd = std_deviation(data);
    data.iter().map(|&x| (x - avg) / sd).collect_vec()
}

pub fn extract_intervals(data: &[usize]) -> Vec<Interval<usize>> {
    let n = data.len();
    let mut result = Vec::new();
    let mut i = 0;
    while i < n {
        let low = data[i];
        while i < n - 1 && data[i] + 1 == data[i + 1] {
            i += 1;
        }
        let high = data[i];
        if high - low >= 2 {
            result.push(Interval::Interval(low, high));
        } else if high - low == 1 {
            result.push(Interval::Single(low));
            result.push(Interval::Single(high));
        } else {
            result.push(Interval::Single(low));
        }
        i += 1;
    }
    result
}
