use std::array;

use crate::lit::Lit;
use crate::solver::Solver;

impl<S> Ops for S where S: Solver + ?Sized {}

pub trait Ops: Solver {
    #[deprecated = "old stuff"]
    fn declare<C, I, L>(&mut self, clauses: C)
    where
        C: IntoIterator<Item = I>,
        I: IntoIterator<Item = L>,
        L: Into<Lit>,
    {
        for clause in clauses.into_iter() {
            self.add_clause(clause);
        }
    }

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
    }

    // ========
    // imply-*
    // ========

    /// `lhs => AND(rhs)`
    fn imply_and<I>(&mut self, lhs: Lit, rhs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        for x in rhs.into_iter() {
            self.imply(lhs, x);
        }
    }

    /// `lhs => OR(rhs)`
    fn imply_or<I>(&mut self, lhs: Lit, rhs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        let rhs = rhs.into_iter();
        let mut v = Vec::with_capacity(1 + rhs.size_hint().0);
        v.push(-lhs);
        v.extend(rhs);
        self.add_clause(v);

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
        // FIXME: see [2009] (?) article for additional (redundant, but efficient!) clauses
        self.imply_imply(lhs, cond, a);
        self.imply_imply(lhs, -cond, b);
    }

    // ==============
    // imply-imply-*
    // ==============

    /// `x1 => (x2 => AND(xs))`
    fn imply_imply_and<I>(&mut self, x1: Lit, x2: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        for x in xs.into_iter() {
            self.imply_imply(x1, x2, x);
        }
    }

    /// `x1 => (x2 => OR(xs))`
    fn imply_imply_or<I>(&mut self, x1: Lit, x2: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        let xs = xs.into_iter();
        let mut v = Vec::with_capacity(2 + xs.size_hint().0);
        v.push(-x1);
        v.push(-x2);
        v.extend(xs);
        self.add_clause(v);
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
    }

    // ====================
    // imply-imply-imply-*
    // ====================

    /// `x1 => (x2 => (x3 => AND(xs)))`
    fn imply_imply_imply_and<I>(&mut self, x1: Lit, x2: Lit, x3: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        for x in xs.into_iter() {
            self.imply_imply_imply(x1, x2, x3, x);
        }
    }

    /// `x1 => (x2 => OR(xs))`
    fn imply_imply_imply_or<I>(&mut self, x1: Lit, x2: Lit, x3: Lit, xs: I)
    where
        I: IntoIterator<Item = Lit>,
    {
        let xs = xs.into_iter();
        let mut v = Vec::with_capacity(3 + xs.size_hint().0);
        v.push(-x1);
        v.push(-x2);
        v.push(-x3);
        v.extend(xs);
        self.add_clause(v);
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
        // TODO: check
        self.imply_ite(lhs, cond, a, b);
        self.imply_imply(cond, a, lhs);
        self.imply_imply(-cond, b, lhs);
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
