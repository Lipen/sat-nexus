use super::Lit;

/// A clause from the IPASIR solver.
///
/// Note: last literal is 0.
pub struct Clause<'a> {
    /// The zero-ended literals.
    lits: &'a [Lit],
}

impl<'a> Clause<'a> {
    /// Returns the length of the clause.
    pub fn len(&self) -> usize {
        self.lits.len()
    }

    /// Returns `true` if the clause is empty.
    ///
    /// # Note
    ///
    /// Normally a clause should never be empty.
    pub fn is_empty(&self) -> bool {
        self.lits.len() == 0
    }

    /// Returns an iterator over the literals of the clause.
    pub fn iter(&self) -> impl Iterator<Item = &Lit> {
        self.lits.iter()
    }
}

impl<'a> From<&'a [Lit]> for Clause<'a> {
    fn from(lits: &'a [Lit]) -> Self {
        debug_assert!(!lits.is_empty());
        debug_assert_eq!(lits.last().map(|x| x.get()), Some(0));
        Self { lits }
    }
}

impl<'a, Idx> std::ops::Index<Idx> for Clause<'a>
where
    Idx: std::slice::SliceIndex<[Lit]>,
{
    type Output = <[Lit] as std::ops::Index<Idx>>::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.lits[index]
    }
}
