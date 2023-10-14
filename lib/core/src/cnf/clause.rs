use std::fmt::{Display, Formatter};
use std::slice::Iter;
use std::vec::IntoIter;

use itertools::Itertools;

use crate::lit::Lit;

#[derive(Debug, Clone)]
pub struct Clause {
    pub lits: Vec<Lit>,
}

impl Clause {
    pub fn iter(&self) -> Iter<'_, Lit> {
        self.lits.iter()
    }
}

impl Clause {
    pub fn new(lits: Vec<Lit>) -> Self {
        debug_assert!(!lits.is_empty(), "Clause must be non-empty");
        debug_assert!(
            itertools::equal(lits.iter(), lits.iter().unique()),
            "Clause must have unique literals"
        );
        Clause { lits }
    }
}

impl<L> FromIterator<L> for Clause
where
    L: Into<Lit>,
{
    fn from_iter<T: IntoIterator<Item = L>>(iter: T) -> Self {
        let lits = iter.into_iter().map_into::<Lit>().collect();
        Self::new(lits)
    }
}

impl IntoIterator for Clause {
    type Item = Lit;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.lits.into_iter()
    }
}

impl<'a> IntoIterator for &'a Clause {
    type Item = &'a Lit;
    type IntoIter = Iter<'a, Lit>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Display for Clause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        for lit in self.lits.iter() {
            list.entry(&format_args!("{}", lit));
        }
        list.finish()
    }
}

impl PartialEq for Clause {
    fn eq(&self, other: &Self) -> bool {
        if self.lits.len() != other.lits.len() {
            return false;
        }
        let lhs = self.lits.iter().copied().sorted_unstable();
        let rhs = other.lits.iter().copied().sorted_unstable();
        itertools::equal(lhs, rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_display_clause() {
        let lits = vec![Lit::new(1), -Lit::new(2), Lit::new(3)];
        let clause = Clause::new(lits);
        assert_eq!("[1, -2, 3]", &format!("{}", clause))
    }
}
