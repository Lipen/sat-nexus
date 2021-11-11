use crate::lit::Lit;

pub struct Clause {
    pub lits: Vec<Lit>,
}

impl Clause {
    pub fn new(lits: Vec<Lit>) -> Self {
        Clause { lits }
    }
}

impl From<Vec<Lit>> for Clause {
    fn from(value: Vec<Lit>) -> Self {
        Self::new(value)
    }
}

impl From<&[Lit]> for Clause {
    fn from(value: &[Lit]) -> Self {
        Self::new(value.to_vec())
    }
}

impl IntoIterator for Clause {
    type Item = Lit;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.lits.into_iter()
    }
}
