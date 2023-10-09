pub fn encode_geq(x: &[i32], a: &[bool]) -> Vec<Vec<i32>> {
    let n = x.len();
    assert_eq!(n, a.len());

    if x.is_empty() {
        return Vec::new();
    }

    let mut clauses = Vec::new();
    let mut prefix = Vec::new(); // x[i]'s corresponding to a[i]=0

    for i in 0..n {
        if a[i] {
            let mut new_clause = prefix.clone();
            new_clause.push(x[i]);
            clauses.push(new_clause);
        } else {
            prefix.push(x[i]);
        }
    }

    clauses
}

pub fn encode_leq(x: &[i32], b: &[bool]) -> Vec<Vec<i32>> {
    let n = x.len();
    assert_eq!(n, b.len());

    if x.is_empty() {
        return Vec::new();
    }

    let mut clauses = Vec::new();
    let mut prefix = Vec::new(); // x[i]'s corresponding to b[i]=1

    for i in 0..n {
        if !b[i] {
            let mut new_clause = prefix.clone();
            new_clause.push(-x[i]);
            clauses.push(new_clause);
        } else {
            prefix.push(-x[i]);
        }
    }

    clauses
}

pub fn encode_both(x: &[i32], a: &[bool], b: &[bool]) -> Vec<Vec<i32>> {
    let n = x.len();
    assert_eq!(n, a.len());
    assert_eq!(n, b.len());

    if x.is_empty() {
        return Vec::new();
    }

    let mut clauses = Vec::new();
    let mut index: Option<usize> = None;

    for i in 0..n {
        if a[i] {
            assert!(b[i]);
            clauses.push(vec![x[i]]);
        } else if !b[i] {
            assert!(!a[i]);
            clauses.push(vec![-x[i]]);
        } else {
            assert!(!a[i]);
            assert!(b[i]);
            index = Some(i);
            break;
        }
    }

    if let Some(index) = index {
        // Note: both for GEQ and LEQ, the encoding implementation is the same
        //       as in the corresponding `encode_geq`/`encode_leq` functions.
        // However, the `prefix` is not empty and different for each case!

        // GEQ
        let mut prefix = vec![x[index]]; // x[i]'s corresponding to a[i]=0
        for i in (index + 1)..n {
            if a[i] {
                let mut new_clause = prefix.clone();
                new_clause.push(x[i]);
                clauses.push(new_clause);
            } else {
                prefix.push(x[i]);
            }
        }

        // LEQ
        let mut prefix = vec![-x[index]]; // x[i]'s corresponding to bi=1
        for i in (index + 1)..n {
            if !b[i] {
                let mut new_clause = prefix.clone();
                new_clause.push(-x[i]);
                clauses.push(new_clause);
            } else {
                prefix.push(-x[i]);
            }
        }
    }

    clauses
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::Itertools;

    fn num2bits(x: u32, n: usize) -> Vec<bool> {
        let mut bits = vec![false; n];

        for i in 0..n.min(32) {
            bits[n - i - 1] = (x >> i) & 1 != 0;
        }

        bits
    }

    fn bits2str(bits: &[bool]) -> String {
        bits.iter().map(|&bit| if bit { '1' } else { '0' }).collect::<String>()
    }

    fn bits2num(bits: &[bool]) -> u32 {
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

    fn sort_clauses(clauses: &mut [Vec<i32>]) {
        // Sort each clause:
        for clause in clauses.iter_mut() {
            clause.sort_by_key(|&lit| lit.abs());
        }
        // Sort all clauses:
        clauses.sort_by_key(|c| c.iter().map(|&lit| lit.abs()).collect_vec());
    }

    #[test]
    fn test_geq() {
        let n = 8;
        let x: Vec<i32> = (1..=n as i32).collect();
        let a: Vec<bool> = num2bits(13, n);

        println!("n = {}", n);
        println!("x = {:?}", x);
        println!("a = {} = {}", bits2str(&a), bits2num(&a));

        let mut clauses_geq = encode_geq(&x, &a);
        sort_clauses(&mut clauses_geq);
        println!("GEQ clauses ({}): {:?}", clauses_geq.len(), clauses_geq);
        assert_eq!(clauses_geq, vec![vec![1, 2, 3, 4, 5], vec![1, 2, 3, 4, 6], vec![1, 2, 3, 4, 7, 8]]);
    }

    #[test]
    fn test_leq() {
        let n = 8;
        let x: Vec<i32> = (1..=n as i32).collect();
        let b: Vec<bool> = num2bits(42, n);

        println!("n = {}", n);
        println!("x = {:?}", x);
        println!("b = {} = {}", bits2str(&b), bits2num(&b));

        let mut clauses_leq = encode_leq(&x, &b);
        sort_clauses(&mut clauses_leq);
        println!("LEQ clauses ({}): {:?}", clauses_leq.len(), clauses_leq);
        assert_eq!(
            clauses_leq,
            vec![vec![-1], vec![-2], vec![-3, -4], vec![-3, -5, -6], vec![-3, -5, -7, -8]]
        );
    }

    #[test]
    fn test_both() {
        let n = 8;
        let x: Vec<i32> = (1..=n as i32).collect();
        let a: Vec<bool> = num2bits(13, n);
        let b: Vec<bool> = num2bits(42, n);

        println!("n = {}", n);
        println!("x = {:?}", x);
        println!("a = {} = {}", bits2str(&a), bits2num(&a));
        println!("b = {} = {}", bits2str(&b), bits2num(&b));

        let mut clauses_geq = encode_geq(&x, &a);
        sort_clauses(&mut clauses_geq);
        let mut clauses_leq = encode_leq(&x, &b);
        sort_clauses(&mut clauses_leq);
        let mut clauses_both = encode_both(&x, &a, &b);
        sort_clauses(&mut clauses_both);

        println!("GEQ clauses ({}): {:?}", clauses_geq.len(), clauses_geq);
        println!("LEQ clauses ({}): {:?}", clauses_leq.len(), clauses_leq);
        println!("Both clauses ({}): {:?}", clauses_both.len(), clauses_both);
        assert_eq!(
            clauses_both,
            vec![
                vec![-1],
                vec![-2],
                vec![-3, -4],
                vec![3, 4, 5],
                vec![3, 4, 6],
                vec![3, 4, 7, 8],
                vec![-3, -5, -6],
                vec![-3, -5, -7, -8]
            ]
        );
    }

    #[test]
    fn test_big() {
        let n = 100000;
        let x: Vec<i32> = (1..=n as i32).collect();
        let a: Vec<bool> = num2bits(13, n);
        let b: Vec<bool> = num2bits(42, n);

        println!("n = {}", n);

        let clauses_geq = encode_geq(&x, &a);
        let clauses_leq = encode_leq(&x, &b);
        let clauses_both = encode_both(&x, &a, &b);

        println!("GEQ clauses: ({})", clauses_geq.len());
        // println!("{:?}", geq_clauses);
        println!("LEQ clauses: ({})", clauses_leq.len());
        // println!("{:?}", leq_clauses);
        println!("Both clauses: ({})", clauses_both.len());
        // println!("{:?}", both_clauses);

        // No asserts, because we only check that it runs without errors
        //  and does not consume infinite amount of memory.
    }
}
