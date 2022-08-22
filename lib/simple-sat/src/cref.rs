use crate::clause::Clause;
use std::ops::Index;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ClauseRef(pub(crate) usize);

impl Index<ClauseRef> for Vec<Clause> {
    type Output = Clause;

    fn index(&self, index: ClauseRef) -> &Self::Output {
        &self[index.0]
    }
}
