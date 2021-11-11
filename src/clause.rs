use itertools::Itertools;

use crate::lit::Lit;

pub struct Clause {
    pub lits: Vec<Lit>,
}

impl Clause {
    pub fn new(lits: Vec<Lit>) -> Self {
        Clause { lits }
    }
}

impl<L> FromIterator<L> for Clause
where
    L: Into<Lit>,
{
    fn from_iter<T: IntoIterator<Item = L>>(iter: T) -> Self {
        Self::new(iter.into_iter().map_into::<Lit>().collect())
    }
}

impl IntoIterator for Clause {
    type Item = Lit;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.lits.into_iter()
    }
}
