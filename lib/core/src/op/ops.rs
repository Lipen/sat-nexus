use std::array;

use itertools::chain;

use crate::lit::Lit;

pub trait AddClause {
    fn add_clause<I>(&mut self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<Lit>;
}

impl<S> Ops for S where S: AddClause {}

pub trait Ops: AddClause {
    // ==========
    // basic ops
    // ==========

    /// `lhs => rhs`
    fn imply(&mut self, lhs: Lit, rhs: Lit) {
        self.add_clause([-lhs, rhs]);
    }

    /// `lhs <=> rhs`
    fn iff(&mut self, lhs: Lit, rhs: Lit) {
        self.imply(lhs, rhs);
        self.imply(rhs, lhs);
    }

    /// `ITE(cond, a, b)`
    fn ite(&mut self, cond: Lit, a: Lit, b: Lit) {
        self.imply(cond, a);
        self.imply(-cond, b);
        // Optional clause:
        self.add_clause([a, b]);
    }

    // ========
    // imply-*
    // ========

    /// `lhs => AND(rhs)`
    fn imply_and<I>(&mut self, lhs: Lit, rhs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        for x in rhs {
            self.imply(lhs, x);
        }
    }

    /// `lhs => OR(rhs)`
    fn imply_or<I>(&mut self, lhs: Lit, rhs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        self.add_clause(chain([-lhs], rhs));

        // let rhs = rhs.into_iter();
        // let mut v = Vec::with_capacity(1 + rhs.size_hint().0);
        // v.push(-lhs);
        // v.extend(rhs);
        // self.add_clause(v);

        // self.add_clause_lit(-lhs);
        // for x in rhs.into_iter() {
        //     self.add_clause_lit(x);
        // }
        // self.finalize_clause();
    }

    /// `x1 => (x2 => x3)`
    fn imply_imply(&mut self, x1: Lit, x2: Lit, x3: Lit) {
        self.add_clause([-x1, -x2, x3]);
    }

    /// `x1 => (x2 <=> x3)`
    fn imply_iff(&mut self, x1: Lit, x2: Lit, x3: Lit) {
        self.imply_imply(x1, x2, x3);
        self.imply_imply(x1, x3, x2);
    }

    /// `lhs => ITE(cond, a, b)`
    fn imply_ite(&mut self, lhs: Lit, cond: Lit, a: Lit, b: Lit) {
        self.add_clause([-lhs, -cond, a]);
        self.add_clause([-lhs, cond, b]);
        // Optional clause:
        self.add_clause([-lhs, a, b]);
    }

    // ==============
    // imply-imply-*
    // ==============

    /// `x1 => (x2 => AND(xs))`
    fn imply_imply_and<I>(&mut self, x1: Lit, x2: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        for x in xs {
            self.imply_imply(x1, x2, x);
        }
    }

    /// `x1 => (x2 => OR(xs))`
    fn imply_imply_or<I>(&mut self, x1: Lit, x2: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        self.add_clause(chain([-x1, -x2], xs));

        // let xs = xs.into_iter();
        // let mut v = Vec::with_capacity(2 + xs.size_hint().0);
        // v.push(-x1);
        // v.push(-x2);
        // v.extend(xs);
        // self.add_clause(v);
    }

    /// `x1 => (x2 => (x3 => x4))`
    fn imply_imply_imply(&mut self, x1: Lit, x2: Lit, x3: Lit, x4: Lit) {
        self.add_clause([-x1, -x2, -x3, x4]);
    }

    /// `x1 => (x2 => (x3 <=> x4))`
    fn imply_imply_iff(&mut self, x1: Lit, x2: Lit, x3: Lit, x4: Lit) {
        self.imply_imply_imply(x1, x2, x3, x4);
        self.imply_imply_imply(x1, x2, x4, x3);
    }

    /// `x1 => (x2 => ITE(cond, a, b))`
    fn imply_imply_ite(&mut self, x1: Lit, x2: Lit, cond: Lit, a: Lit, b: Lit) {
        self.imply_imply_imply(x1, x2, cond, a);
        self.imply_imply_imply(x1, x2, -cond, b);
        // TODO: Optional clauses
    }

    // ====================
    // imply-imply-imply-*
    // ====================

    /// `x1 => (x2 => (x3 => AND(xs)))`
    fn imply_imply_imply_and<I>(&mut self, x1: Lit, x2: Lit, x3: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        for x in xs {
            self.imply_imply_imply(x1, x2, x3, x);
        }
    }

    /// `x1 => (x2 => OR(xs))`
    fn imply_imply_imply_or<I>(&mut self, x1: Lit, x2: Lit, x3: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        self.add_clause(chain([-x1, -x2, -x3], xs));

        // let xs = xs.into_iter();
        // let mut v = Vec::with_capacity(3 + xs.size_hint().0);
        // v.push(-x1);
        // v.push(-x2);
        // v.push(-x3);
        // v.extend(xs);
        // self.add_clause(v);
    }

    /// `x1 => (x2 => (x3 => (x4 => x5)))`
    fn imply_imply_imply_imply(&mut self, x1: Lit, x2: Lit, x3: Lit, x4: Lit, x5: Lit) {
        self.add_clause([-x1, -x2, -x3, -x4, x5]);
    }

    /// `x1 => (x2 => (x3 => (x4 <=> x5)))`
    fn imply_imply_imply_iff(&mut self, x1: Lit, x2: Lit, x3: Lit, x4: Lit, x5: Lit) {
        self.imply_imply_imply_imply(x1, x2, x3, x4, x5);
        self.imply_imply_imply_imply(x1, x2, x3, x5, x4);
    }

    /// `x1 => (x2 => (x3 => ITE(cond, a, b)))`
    fn imply_imply_imply_ite(&mut self, x1: Lit, x2: Lit, x3: Lit, cond: Lit, a: Lit, b: Lit) {
        self.imply_imply_imply_imply(x1, x2, x3, cond, a);
        self.imply_imply_imply_imply(x1, x2, x3, -cond, b);
        // TODO: Optional clauses
    }

    // ======
    // iff-*
    // ======

    /// `lhs <=> AND(xs)`
    fn iff_and<I>(&mut self, lhs: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        let xs = xs.into_iter();
        let mut v = Vec::with_capacity(1 + xs.size_hint().0);
        v.push(lhs);
        for x in xs {
            v.push(-x);
            self.imply(lhs, x);
        }
        // `v` is the clause `(lhs, -x1, -x2, ..., -xN)`
        self.add_clause(v);
    }

    /// `lhs <=> OR(xs)`
    fn iff_or<I>(&mut self, lhs: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        let xs = xs.into_iter();
        let mut v = Vec::with_capacity(1 + xs.size_hint().0);
        v.push(-lhs);
        for x in xs {
            v.push(x);
            self.imply(x, lhs);
        }
        // `v` is the clause `(-lhs, x1, x2, ..., xN)`
        self.add_clause(v);
    }

    /// `lhs <=> (x1 => x2)`
    fn iff_imply(&mut self, lhs: Lit, x1: Lit, x2: Lit) {
        self.imply_imply(lhs, x1, x2);
        self.add_clause([lhs, x1]);
        self.add_clause([lhs, -x2]);
    }

    /// `lhs <=> (x1 <=> x2)`
    fn iff_iff(&mut self, lhs: Lit, x1: Lit, x2: Lit) {
        self.imply_iff(lhs, x1, x2);
        self.add_clause([lhs, -x1, -x2]);
        self.add_clause([lhs, x1, x2]);
    }

    /// `lhs <=> ITE(cond, a, b)`
    fn iff_ite(&mut self, lhs: Lit, cond: Lit, a: Lit, b: Lit) {
        self.add_clause([lhs, -cond, -a]);
        self.add_clause([-lhs, -cond, a]);
        self.add_clause([lhs, cond, -b]);
        self.add_clause([-lhs, cond, b]);
        // Optional clauses:
        self.add_clause([lhs, -a, -b]);
        self.add_clause([-lhs, a, b]);
    }

    // ============
    // imply-iff-*
    // ============

    /// `x1 => (x2 <=> AND(xs))`
    fn imply_iff_and<I>(&mut self, x1: Lit, x2: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        let xs = xs.into_iter();
        let mut v = Vec::with_capacity(2 + xs.size_hint().0);
        v.push(-x1);
        v.push(x2);
        for x in xs {
            v.push(-x);
            self.imply_imply(x1, x2, x);
        }
        // `v` is the clause `(-x1, x2, -xs1, -xs2, ..., -xsN)`
        self.add_clause(v);
    }

    /// `x1 => (x2 <=> OR(xs))`
    fn imply_iff_or<I>(&mut self, x1: Lit, x2: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        let xs = xs.into_iter();
        let mut v = Vec::with_capacity(2 + xs.size_hint().0);
        v.push(-x1);
        v.push(-x2);
        for x in xs {
            v.push(x);
            self.imply_imply(x1, x, x2);
        }
        // `v` is the clause `(-x1, -x2, xs1, xs2, ..., xsN)`
        self.add_clause(v);
    }

    // TODO:
    //  imply_iff_imply
    //  imply_iff_iff
    //  imply_iff_ite

    // ==================
    // imply-imply-iff-*
    // ==================

    // TODO:
    //  imply_imply_iff_and
    //  imply_imply_iff_or
    //  imply_imply_iff_imply
    //  imply_imply_iff_iff
    //  imply_imply_iff_ite
}

#[deprecated = "`IntoIterator for [T;N]` was stabilized in Rust 1.53"]
#[allow(deprecated)]
pub trait OpsArray: Ops {
    fn imply_and_array<const N: usize>(&mut self, lhs: Lit, rhs: [Lit; N]) {
        self.imply_and(lhs, array::IntoIter::new(rhs));
    }

    fn imply_or_array<const N: usize>(&mut self, lhs: Lit, rhs: [Lit; N]) {
        self.imply_or(lhs, array::IntoIter::new(rhs))
    }

    fn imply_imply_and_array<const N: usize>(&mut self, x1: Lit, x2: Lit, xs: [Lit; N]) {
        self.imply_imply_and(x1, x2, array::IntoIter::new(xs));
    }

    fn imply_imply_or_array<const N: usize>(&mut self, x1: Lit, x2: Lit, xs: [Lit; N]) {
        self.imply_imply_or(x1, x2, array::IntoIter::new(xs));
    }

    fn imply_imply_imply_and_array<const N: usize>(&mut self, x1: Lit, x2: Lit, x3: Lit, xs: [Lit; N]) {
        self.imply_imply_imply_and(x1, x2, x3, array::IntoIter::new(xs));
    }

    fn imply_imply_imply_or_array<const N: usize>(&mut self, x1: Lit, x2: Lit, x3: Lit, xs: [Lit; N]) {
        self.imply_imply_imply_or(x1, x2, x3, array::IntoIter::new(xs));
    }

    fn iff_and_array<const N: usize>(&mut self, lhs: Lit, xs: [Lit; N]) {
        self.iff_and(lhs, array::IntoIter::new(xs));
    }

    fn iff_or_array<const N: usize>(&mut self, lhs: Lit, xs: [Lit; N]) {
        self.iff_or(lhs, array::IntoIter::new(xs));
    }

    fn imply_iff_and_array<const N: usize>(&mut self, x1: Lit, x2: Lit, xs: [Lit; N]) {
        self.imply_iff_and(x1, x2, array::IntoIter::new(xs));
    }

    fn imply_iff_or_array<const N: usize>(&mut self, x1: Lit, x2: Lit, xs: [Lit; N]) {
        self.imply_iff_or(x1, x2, array::IntoIter::new(xs));
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};
    use std::fmt::Debug;
    use std::hash::{Hash, Hasher};

    use itertools::Itertools;

    use super::*;

    fn my_eq<T>(a: &[T], b: &[T]) -> bool
    where
        T: Eq + Hash,
    {
        let a: HashSet<_> = a.iter().collect();
        let b: HashSet<_> = b.iter().collect();
        a == b
    }

    #[derive(Debug, Clone, Eq)]
    struct Clause(Vec<Lit>);

    impl PartialEq for Clause {
        fn eq(&self, other: &Self) -> bool {
            my_eq(&self.0, &other.0)
        }
    }

    impl Hash for Clause {
        fn hash<H: Hasher>(&self, state: &mut H) {
            BTreeSet::from_iter(self.0.iter().copied()).hash(state);
        }
    }

    impl<I> From<I> for Clause
    where
        I: IntoIterator,
        I::Item: Into<Lit>,
    {
        fn from(iter: I) -> Self {
            Clause(iter.into_iter().map_into::<Lit>().collect_vec())
        }
    }

    #[derive(Debug, Clone, Eq)]
    struct Clauses(Vec<Clause>);

    impl Clauses {
        fn new() -> Self {
            Clauses(Vec::new())
        }
    }

    impl PartialEq for Clauses {
        fn eq(&self, other: &Self) -> bool {
            my_eq(&self.0, &other.0)
        }
    }

    impl AddClause for Clauses {
        fn add_clause<I>(&mut self, lits: I)
        where
            I: IntoIterator,
            I::Item: Into<Lit>,
        {
            self.0.push(Clause::from(lits));
        }
    }

    const X1: Lit = Lit::new(1);
    const X2: Lit = Lit::new(2);
    const X3: Lit = Lit::new(3);
    const X4: Lit = Lit::new(4);
    const X5: Lit = Lit::new(5);
    const X6: Lit = Lit::new(6);

    fn run<F, I>(f: F, expected: I)
    where
        F: FnOnce(&mut Clauses),
        I: IntoIterator,
        I::Item: Into<Clause>,
    {
        let mut clauses = Clauses::new();
        f(&mut clauses);
        let expected = expected.into_iter().map_into::<Clause>().collect_vec();
        assert_eq!(clauses, Clauses(expected));
    }

    #[test]
    fn test_imply() {
        run(|s| s.imply(X1, X2), [[-X1, X2]]);
        run(|s| s.imply(-X3, X1), [[X3, X1]]);
    }

    #[test]
    fn test_iff() {
        run(|s| s.iff(X1, X2), [[-X1, X2], [X1, -X2]]);
        run(|s| s.iff(-X3, -X1), [[X3, -X1], [-X3, X1]]);
    }

    #[test]
    fn test_ite() {
        run(|s| s.ite(X1, X2, X3), [[-X1, X2], [X1, X3], [X2, X3]]);
    }

    #[test]
    fn test_imply_and() {
        run(|s| s.imply_and(X1, [X2, X3, X4]), [[-X1, X2], [-X1, X3], [-X1, X4]]);
    }

    #[test]
    fn test_imply_or() {
        run(|s| s.imply_or(X1, [X2, X3, X4]), [[-X1, X2, X3, X4]]);
    }

    #[test]
    fn test_imply_imply() {
        run(|s| s.imply_imply(X1, X2, X3), [[-X1, -X2, X3]]);
    }

    #[test]
    fn test_imply_iff() {
        run(|s| s.imply_iff(X1, X2, X3), [[-X1, -X2, X3], [-X1, X2, -X3]]);
    }

    #[test]
    fn test_imply_ite() {
        run(|s| s.imply_ite(X1, X2, X3, X4), [[-X1, -X2, X3], [-X1, X2, X4], [-X1, X3, X4]]);
    }

    #[test]
    fn test_imply_imply_and() {
        run(
            |s| s.imply_imply_and(X1, X2, [X3, X4, X5]),
            [[-X1, -X2, X3], [-X1, -X2, X4], [-X1, -X2, X5]],
        );
    }

    #[test]
    fn test_imply_imply_or() {
        run(|s| s.imply_imply_or(X1, X2, [X3, X4, X5]), [[-X1, -X2, X3, X4, X5]]);
    }

    #[test]
    fn test_imply_imply_imply() {
        run(|s| s.imply_imply_imply(X1, X2, X3, X4), [[-X1, -X2, -X3, X4]]);
    }

    #[test]
    fn test_imply_imply_iff() {
        run(|s| s.imply_imply_iff(X1, X2, X3, X4), [[-X1, -X2, -X3, X4], [-X1, -X2, X3, -X4]]);
    }

    #[test]
    fn test_imply_imply_ite() {
        run(|s| s.imply_imply_ite(X1, X2, X3, X4, X5), [[-X1, -X2, -X3, X4], [-X1, -X2, X3, X5]]);
    }

    #[test]
    fn test_imply_imply_imply_and() {
        run(
            |s| s.imply_imply_imply_and(X1, X2, X3, [X4, X5, X6]),
            [[-X1, -X2, -X3, X4], [-X1, -X2, -X3, X5], [-X1, -X2, -X3, X6]],
        );
    }

    #[test]
    fn test_imply_imply_imply_or() {
        run(|s| s.imply_imply_imply_or(X1, X2, X3, [X4, X5, X6]), [[-X1, -X2, -X3, X4, X5, X6]]);
    }

    #[test]
    fn test_imply_imply_imply_imply() {
        run(|s| s.imply_imply_imply_imply(X1, X2, X3, X4, X5), [[-X1, -X2, -X3, -X4, X5]]);
    }

    #[test]
    fn test_imply_imply_imply_iff() {
        run(
            |s| s.imply_imply_imply_iff(X1, X2, X3, X4, X5),
            [[-X1, -X2, -X3, -X4, X5], [-X1, -X2, -X3, X4, -X5]],
        );
    }

    #[test]
    fn test_imply_imply_imply_ite() {
        run(
            |s| s.imply_imply_imply_ite(X1, X2, X3, X4, X5, X6),
            [[-X1, -X2, -X3, -X4, X5], [-X1, -X2, -X3, X4, X6]],
        );
    }

    #[test]
    fn test_iff_and() {
        run(
            |s| s.iff_and(X1, [X2, X3, X4]),
            [vec![X1, -X2, -X3, -X4], vec![-X1, X2], vec![-X1, X3], vec![-X1, X4]],
        );
    }

    #[test]
    fn test_iff_or() {
        run(
            |s| s.iff_or(X1, [X2, X3, X4]),
            [vec![-X1, X2, X3, X4], vec![X1, -X2], vec![X1, -X3], vec![X1, -X4]],
        );
    }

    #[test]
    fn test_imply_iff_and() {
        run(
            |s| s.imply_iff_and(X1, X2, [X3, X4, X5]),
            [
                vec![-X1, X2, -X3, -X4, -X5],
                vec![-X1, -X2, X3],
                vec![-X1, -X2, X4],
                vec![-X1, -X2, X5],
            ],
        );
    }

    #[test]
    fn test_imply_iff_or() {
        run(
            |s| s.imply_iff_or(X1, X2, [X3, X4, X5]),
            [
                vec![-X1, -X2, X3, X4, X5],
                vec![-X1, X2, -X3],
                vec![-X1, X2, -X4],
                vec![-X1, X2, -X5],
            ],
        );
    }
}
