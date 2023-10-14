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
        for clause in encode_geq(&x[index + 1..], &a[index + 1..]) {
            let mut new_clause = vec![x[index]];
            new_clause.extend(clause);
            clauses.push(new_clause);
        }

        for clause in encode_leq(&x[index + 1..], &b[index + 1..]) {
            let mut new_clause = vec![-x[index]];
            new_clause.extend(clause);
            clauses.push(new_clause);
        }
    }

    clauses
}

pub fn encode_gt(x: &[i32], a: &[bool]) -> Vec<Vec<i32>> {
    let n = x.len();
    assert_eq!(n, a.len());

    if let Some(a1) = bits_increment(&a) {
        encode_geq(x, &a1)
    } else {
        // Empty clause:
        vec![vec![]]
    }
}

pub fn encode_lt(x: &[i32], b: &[bool]) -> Vec<Vec<i32>> {
    let n = x.len();
    assert_eq!(n, b.len());

    if let Some(b1) = bits_decrement(&b) {
        encode_leq(x, &b1)
    } else {
        // Empty clause:
        vec![vec![]]
    }
}

/// `t <=> (x >= a)`
pub fn encode_geq_reified(t: i32, x: &[i32], a: &[bool]) -> Vec<Vec<i32>> {
    let n = x.len();
    assert_eq!(n, a.len());

    if x.is_empty() {
        return Vec::new();
    }

    let mut clauses = Vec::new();

    if a[0] {
        // (~t \/ ~x[0] \/ (x[1..] >= a[1..]))   // prefix1 + GEQ
        // /\ (t \/ ~x[0] \/ (x[1..] < a[1..]))  // prefix2 + LT
        // /\ (~t \/ x[0])                       // binary clause

        // binary clause:
        clauses.push(vec![-t, x[0]]);

        // prefix1 + GEQ:
        for clause in encode_geq(&x[1..], &a[1..]) {
            let mut new_clause = vec![-t, -x[0]];
            new_clause.extend(&clause);
            clauses.push(new_clause);
        }

        // prefix2 + LT:
        for clause in encode_lt(&x[1..], &a[1..]) {
            let mut new_clause = vec![t, -x[0]];
            new_clause.extend(&clause);
            clauses.push(new_clause);
        }
    } else {
        // (~t \/ x[0] \/ (x[1..] >= a[1..])   // prefix1 + GEQ
        // /\ (t \/ x[0] \/ (x[1..] < a[1..])  // prefix2 + LT
        // /\ (t \/ ~x[0])                     // binary clause

        // binary clause:
        clauses.push(vec![t, -x[0]]);

        // prefix1 + GEQ:
        for clause in encode_geq(&x[1..], &a[1..]) {
            let mut new_clause = vec![-t, x[0]];
            new_clause.extend(&clause);
            clauses.push(new_clause);
        }

        // prefix2 + LT:
        for clause in encode_lt(&x[1..], &a[1..]) {
            let mut new_clause = vec![t, x[0]];
            new_clause.extend(&clause);
            clauses.push(new_clause);
        }
    }

    clauses
}

/// `t <=> (x <= b)`
pub fn encode_leq_reified(t: i32, x: &[i32], b: &[bool]) -> Vec<Vec<i32>> {
    let n = x.len();
    assert_eq!(n, b.len());

    if x.is_empty() {
        return Vec::new();
    }

    let mut clauses = Vec::new();

    if !b[0] {
        // (~t \/ x[0] \/ (x[1..] <= a[1..])   // prefix1 + LEQ
        // /\ (t \/ x[0] \/ (x[1..] > a[1..])  // prefix2 + GT
        // /\ (~t \/ ~x[0])                    // binary clause

        // binary clause:
        clauses.push(vec![-t, -x[0]]);

        // prefix1 + LEQ:
        for clause in encode_leq(&x[1..], &b[1..]) {
            let mut new_clause = vec![-t, x[0]];
            new_clause.extend(&clause);
            clauses.push(new_clause);
        }

        // prefix2 + GT:
        for clause in encode_gt(&x[1..], &b[1..]) {
            let mut new_clause = vec![t, x[0]];
            new_clause.extend(&clause);
            clauses.push(new_clause);
        }
    } else {
        // (~t \/ ~x[0] \/ (x[1..] <= a[1..]))   // prefix1 + LEQ
        // /\ (t \/ ~x[0] \/ (x[1..] > a[1..]))  // prefix2 + GT
        // /\ (t \/ x[0])                        // binary clause

        // binary clause:
        clauses.push(vec![t, x[0]]);

        // prefix1 + LEQ:
        for clause in encode_leq(&x[1..], &b[1..]) {
            let mut new_clause = vec![-t, -x[0]];
            new_clause.extend(&clause);
            clauses.push(new_clause);
        }

        // prefix2 + GT:
        for clause in encode_gt(&x[1..], &b[1..]) {
            let mut new_clause = vec![t, -x[0]];
            new_clause.extend(&clause);
            clauses.push(new_clause);
        }
    }

    clauses
}

// =============================================================

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

fn bits_increment(bits: &[bool]) -> Option<Vec<bool>> {
    let n = bits.len();
    if n == 0 {
        return None;
    }
    let mut result = vec![false; n];
    let mut carry = true;
    for i in (0..n).rev() {
        if carry {
            result[i] = !bits[i];
            carry = bits[i];
        } else {
            result[i] = bits[i];
        }
    }
    if carry {
        None
    } else {
        Some(result)
    }
}

fn bits_decrement(bits: &[bool]) -> Option<Vec<bool>> {
    let n = bits.len();
    if n == 0 {
        return None;
    }
    let mut result = vec![false; n];
    let mut carry = true;
    for i in (0..n).rev() {
        if carry {
            result[i] = !bits[i];
            carry = !bits[i];
        } else {
            result[i] = bits[i];
        }
    }
    if carry {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    fn sort_clauses(clauses: &mut [Vec<i32>]) {
        // Sort each clause:
        for clause in clauses.iter_mut() {
            clause.sort_by_key(|&lit| lit.abs());
        }
        // Sort all clauses:
        clauses.sort_by_key(|c| c.iter().map(|&lit| lit.abs()).collect_vec());
    }

    fn is_satisfied(clause: &[i32], cube: &[bool]) -> bool {
        for &lit in clause {
            if (lit < 0) ^ cube[(lit.abs() - 1) as usize] {
                return true;
            }
        }
        false
    }

    #[test]
    fn test_is_satisfied() {
        let clause = vec![1, 2, 3, 4];
        assert!(!is_satisfied(&clause, &num2bits(0, 4)));
        for x in 1..(1 << 4) {
            assert!(is_satisfied(&clause, &num2bits(x, 4)));
        }

        let clause = vec![-1, -2, -3, -4];
        for x in 0..(1 << 4) - 1 {
            assert!(is_satisfied(&clause, &num2bits(x, 4)));
        }
        assert!(!is_satisfied(&clause, &num2bits((1 << 4) - 1, 4)));

        let clause = vec![1, 3];
        assert!(!is_satisfied(&clause, &num2bits(0b0000, 4)));
        assert!(!is_satisfied(&clause, &num2bits(0b0001, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b0010, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b0011, 4)));
        assert!(!is_satisfied(&clause, &num2bits(0b0100, 4)));
        assert!(!is_satisfied(&clause, &num2bits(0b0101, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b0110, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b0111, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b1000, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b1000, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b1001, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b1010, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b1011, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b1100, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b1101, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b1110, 4)));
        assert!(is_satisfied(&clause, &num2bits(0b1111, 4)));
    }

    #[test]
    fn test_manual() {
        let n = 4;
        let x: Vec<i32> = (1..=n as i32).collect();
        let a: Vec<bool> = num2bits(0b0010, n);

        println!("n = {}", n);
        println!("x = {:?}", x);
        println!("a = {} = {}", bits2str(&a), bits2num(&a));

        let mut clauses_geq = encode_geq(&x, &a);
        sort_clauses(&mut clauses_geq);
        println!("clauses ({}): {:?}", clauses_geq.len(), clauses_geq);
    }

    #[test]
    fn test_geq() {
        let n = 8;
        let x: Vec<i32> = (1..=n as i32).collect();
        let low = 13;
        let a: Vec<bool> = num2bits(low, n);

        println!("n = {}", n);
        println!("x = {:?}", x);
        println!("a = {} = {}", bits2str(&a), bits2num(&a));

        let mut clauses_geq = encode_geq(&x, &a);
        sort_clauses(&mut clauses_geq);
        println!("GEQ clauses ({}): {:?}", clauses_geq.len(), clauses_geq);
        assert_eq!(clauses_geq, vec![vec![1, 2, 3, 4, 5], vec![1, 2, 3, 4, 6], vec![1, 2, 3, 4, 7, 8]]);

        for y in 0..(1 << n) {
            let cube = num2bits(y, n);
            println!();
            println!("y = {} = {}", bits2str(&cube), y);
            for clause in clauses_geq.iter() {
                println!("is_satisfied({:?}): {}", clause, is_satisfied(clause, &cube));
            }
            if y >= low {
                assert!(
                    clauses_geq.iter().all(|c| is_satisfied(c, &cube)),
                    "All clauses must be satisfied for (y>={}): y = {} = {}",
                    low,
                    bits2str(&cube),
                    y
                );
            } else {
                assert!(
                    clauses_geq.iter().any(|c| !is_satisfied(c, &cube)),
                    "At least one clause must be non-satisfied for !(y>={}): y = {} = {}",
                    low,
                    bits2str(&cube),
                    y
                );
            }
        }
    }

    #[test]
    fn test_gt() {
        let n = 4;
        let x: Vec<i32> = (1..=n as i32).collect();
        let low = 5;
        let a: Vec<bool> = num2bits(low, n);

        println!("n = {}", n);
        println!("x = {:?}", x);
        println!("a = {} = {}", bits2str(&a), bits2num(&a));

        let mut clauses_gt = encode_gt(&x, &a);
        sort_clauses(&mut clauses_gt);
        println!("GT clauses ({}): {:?}", clauses_gt.len(), clauses_gt);
        assert_eq!(clauses_gt, vec![vec![1, 2], vec![1, 3]]);

        for y in 0..(1 << n) {
            let cube = num2bits(y, n);
            println!();
            println!("y = {} = {}", bits2str(&cube), y);
            for clause in clauses_gt.iter() {
                println!("is_satisfied({:?}): {}", clause, is_satisfied(clause, &cube));
            }
            if y > low {
                assert!(
                    clauses_gt.iter().all(|c| is_satisfied(c, &cube)),
                    "All clauses must be satisfied for (y>{}): y = {} = {}",
                    low,
                    bits2str(&cube),
                    y
                );
            } else {
                assert!(
                    clauses_gt.iter().any(|c| !is_satisfied(c, &cube)),
                    "At least one clause must be non-satisfied for !(y>{}): y = {} = {}",
                    low,
                    bits2str(&cube),
                    y
                );
            }
        }
    }

    #[test]
    fn test_geq_reified() {
        let n = 8;
        let t = (n + 1) as i32;
        let x: Vec<i32> = (1..=n as i32).collect();
        let low = 5;
        let a: Vec<bool> = num2bits(low, n);

        println!("n = {}", n);
        println!("t = {}", t);
        println!("x = {:?}", x);
        println!("a = {} = {}", bits2str(&a), bits2num(&a));

        let mut clauses_geq_reified = encode_geq_reified(t, &x, &a);
        sort_clauses(&mut clauses_geq_reified);
        println!("GEQ_REIFY clauses ({}): {:?}", clauses_geq_reified.len(), clauses_geq_reified);
        // assert_eq!(clauses_geq_reified, vec![vec![1, 2, 3, 4, 5, 7]]);
    }

    #[test]
    fn test_leq() {
        let n = 8;
        let x: Vec<i32> = (1..=n as i32).collect();
        let high = 42;
        let b: Vec<bool> = num2bits(high, n);

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

        for y in 0..(1 << n) {
            let cube = num2bits(y, n);
            println!();
            println!("y = {} = {}", bits2str(&cube), y);
            for clause in clauses_leq.iter() {
                println!("is_satisfied({:?}): {}", clause, is_satisfied(clause, &cube));
            }
            if y <= high {
                assert!(
                    clauses_leq.iter().all(|c| is_satisfied(c, &cube)),
                    "All clauses must be satisfied for (y<={}): y = {} = {}",
                    high,
                    bits2str(&cube),
                    y
                );
            } else {
                assert!(
                    clauses_leq.iter().any(|c| !is_satisfied(c, &cube)),
                    "At least one clause must be non-satisfied for !(y<={}): y = {} = {}",
                    high,
                    bits2str(&cube),
                    y
                );
            }
        }
    }

    #[test]
    fn test_lt() {
        let n = 4;
        let x: Vec<i32> = (1..=n as i32).collect();
        let high = 5;
        let b: Vec<bool> = num2bits(high, n);

        println!("n = {}", n);
        println!("x = {:?}", x);
        println!("b = {} = {}", bits2str(&b), bits2num(&b));

        let mut clauses_lt = encode_lt(&x, &b);
        sort_clauses(&mut clauses_lt);
        println!("LT clauses ({}): {:?}", clauses_lt.len(), clauses_lt);
        assert_eq!(clauses_lt, vec![vec![-1], vec![-2, -3], vec![-2, -4]]);

        for y in 0..(1 << n) {
            let cube = num2bits(y, n);
            println!();
            println!("y = {} = {}", bits2str(&cube), y);
            for clause in clauses_lt.iter() {
                println!("is_satisfied({:?}): {}", clause, is_satisfied(clause, &cube));
            }
            if y < high {
                assert!(
                    clauses_lt.iter().all(|c| is_satisfied(c, &cube)),
                    "All clauses must be satisfied for (y<{}): y = {} = {}",
                    high,
                    bits2str(&cube),
                    y
                );
            } else {
                assert!(
                    clauses_lt.iter().any(|c| !is_satisfied(c, &cube)),
                    "At least one clause must be non-satisfied for !(y<{}): y = {} = {}",
                    high,
                    bits2str(&cube),
                    y
                );
            }
        }
    }

    #[test]
    fn test_both() {
        let n = 8;
        let x: Vec<i32> = (1..=n as i32).collect();
        let low = 13;
        let high = 42;
        let a: Vec<bool> = num2bits(low, n);
        let b: Vec<bool> = num2bits(high, n);

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
        let n = 10000;
        let x: Vec<i32> = (1..=n as i32).collect();
        let low = 13;
        let high = 42;
        let a: Vec<bool> = num2bits(low, n);
        let b: Vec<bool> = num2bits(high, n);

        println!("n = {}", n);

        let clauses_geq = encode_geq(&x, &a);
        let clauses_leq = encode_leq(&x, &b);
        let clauses_both = encode_both(&x, &a, &b);

        println!("GEQ clauses: ({})", clauses_geq.len());
        // println!("{:?}", clauses_geq);
        println!("LEQ clauses: ({})", clauses_leq.len());
        // println!("{:?}", clauses_leq);
        println!("Both clauses: ({})", clauses_both.len());
        // println!("{:?}", clauses_both);

        // No asserts, because we only check that it runs without errors
        //  and does not consume infinite amount of memory.
    }
}
