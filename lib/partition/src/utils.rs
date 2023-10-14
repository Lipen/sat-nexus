use itertools::Itertools;
use ordered_float::OrderedFloat;

pub fn num2bits(x: usize, n: usize) -> Vec<bool> {
    let mut bits = vec![false; n];

    for i in 0..n.min(32) {
        bits[n - i - 1] = (x >> i) & 1 != 0;
    }

    bits
}

pub fn bits2str(bits: &[bool]) -> String {
    bits.iter().map(|&bit| if bit { '1' } else { '0' }).collect::<String>()
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
    s += &format!("0");
    s
}

pub fn is_power_of_two(x: usize) -> bool {
    x & (x - 1) == 0
}

pub fn mean(data: &[f64]) -> Option<f64> {
    let count = data.len();
    if count > 0 {
        let sum = data.iter().sum::<f64>();
        Some(sum / count as f64)
    } else {
        None
    }
}

pub fn median(data: &[f64]) -> Option<f64> {
    let count = data.len();
    if count == 0 {
        return None;
    }
    if count == 1 {
        return Some(data[0]);
    }
    if count == 2 {
        return Some((data[0] + data[1]) / 2.0);
    }
    let sorted = data.iter().copied().sorted_by_key(|&x| OrderedFloat(x)).collect_vec();
    let mid = count / 2;
    if count % 2 == 0 {
        // Even length: median is the mean of two near-middle elements
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        // Odd length: median is the middle element
        Some(sorted[mid])
    }
}

pub fn std_deviation(data: &[f64]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }
    let avg = mean(data)?;
    let variance = data
        .iter()
        .map(|&x| {
            let diff = x - avg;
            diff * diff
        })
        .sum::<f64>()
        / (data.len() - 1) as f64; // Bessel's correction for unbiased variance
    Some(variance.sqrt())
}

pub fn mean_absolute_deviation(data: &[f64]) -> Option<f64> {
    let avg = mean(data)?;
    let deviations = data.iter().map(|&x| (x - avg).abs()).collect_vec();
    mean(&deviations)
}

pub fn median_absolute_deviation(data: &[f64]) -> Option<f64> {
    let med = median(data)?;
    let deviations = data.iter().map(|&x| (x - med).abs()).collect_vec();
    median(&deviations)
}

pub fn zscore(data: &[f64]) -> Vec<f64> {
    if data.is_empty() {
        return Vec::new();
    }
    let avg = mean(data).unwrap();
    let sd = std_deviation(data).unwrap();
    data.iter().map(|&x| (x - avg) / sd).collect_vec()
}
