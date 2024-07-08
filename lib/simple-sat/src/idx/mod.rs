use self::idx_heap::IdxHeap;
use self::idx_map::IdxMap;
use self::idx_vec::IdxVec;

use crate::lit::Lit;
use crate::var::Var;

pub mod idx_heap;
pub mod idx_map;
pub mod idx_vec;

pub type VarMap<V> = IdxMap<Var, V>;
pub type LitMap<V> = IdxMap<Lit, V>;
pub type VarVec<V> = IdxVec<Var, V>;
pub type LitVec<V> = IdxVec<Lit, V>;
pub type VarHeap = IdxHeap<Var>;

pub trait Idx {
    fn idx(&self) -> usize;
}

impl Idx for Var {
    fn idx(&self) -> usize {
        self.inner() as usize
    }
}

impl Idx for Lit {
    fn idx(&self) -> usize {
        self.inner() as usize
    }
}
