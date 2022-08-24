use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::slice;

use super::Idx;

pub struct IdxVec<K: Idx, V> {
    vec: Vec<V>,
    phantom: PhantomData<K>,
}

impl<K: Idx, V> IdxVec<K, V> {
    pub const fn new() -> Self {
        Self {
            vec: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<K: Idx, V> Default for IdxVec<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Idx, V> From<Vec<V>> for IdxVec<K, V> {
    fn from(vec: Vec<V>) -> Self {
        Self { vec, phantom: PhantomData }
    }
}

impl<K: Idx, V> Debug for IdxVec<K, V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.vec.iter()).finish()
    }
}

impl<K: Idx, V> IdxVec<K, V> {
    pub fn init(&mut self, k: &K)
    where
        V: Default,
    {
        self.init_by(k, Default::default)
    }

    pub fn init_by<F>(&mut self, k: &K, f: F)
    where
        F: FnMut() -> V,
    {
        let new_len = k.idx() + 1;
        if new_len > self.vec.len() {
            self.vec.resize_with(new_len, f);
        }
    }

    pub fn contains_key(&self, k: &K) -> bool {
        k.idx() < self.vec.len()
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.vec.get(k.idx())
    }
    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.vec.get_mut(k.idx())
    }

    pub fn push(&mut self, v: V) {
        self.vec.push(v);
    }

    pub fn iter(&self) -> slice::Iter<V> {
        self.vec.iter()
    }
    pub fn iter_mut(&mut self) -> slice::IterMut<V> {
        self.vec.iter_mut()
    }
}

// vec[key]
impl<K: Idx, V> Index<K> for IdxVec<K, V> {
    type Output = V;

    fn index(&self, k: K) -> &Self::Output {
        self.vec.index(k.idx())
    }
}

// vec[&key]
impl<K: Idx, V> Index<&K> for IdxVec<K, V> {
    type Output = V;

    fn index(&self, k: &K) -> &Self::Output {
        self.vec.index(k.idx())
    }
}

// vec[key] = (value)
impl<K: Idx, V> IndexMut<K> for IdxVec<K, V> {
    fn index_mut(&mut self, k: K) -> &mut Self::Output {
        self.vec.index_mut(k.idx())
    }
}

// vec[&key] = (value)
impl<K: Idx, V> IndexMut<&K> for IdxVec<K, V> {
    fn index_mut(&mut self, k: &K) -> &mut Self::Output {
        self.vec.index_mut(k.idx())
    }
}
