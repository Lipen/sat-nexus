use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use super::Idx;

pub struct IdxMap<K: Idx, V> {
    map: vec_map::VecMap<V>,
    phantom: PhantomData<K>,
}

impl<K: Idx, V> IdxMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: vec_map::VecMap::new(),
            phantom: PhantomData,
        }
    }
}

impl<K: Idx, V> Default for IdxMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Idx, V> Debug for IdxMap<K, V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.map.iter()).finish()
    }
}

impl<K: Idx, V> IdxMap<K, V> {
    pub fn insert(&mut self, k: impl Borrow<K>, v: V) -> Option<V> {
        self.map.insert(k.borrow().idx(), v)
    }

    pub fn remove(&mut self, k: impl Borrow<K>) -> Option<V> {
        self.map.remove(k.borrow().idx())
    }

    pub fn get(&self, k: impl Borrow<K>) -> Option<&V> {
        self.map.get(k.borrow().idx())
    }
    pub fn get_mut(&mut self, k: impl Borrow<K>) -> Option<&mut V> {
        self.map.get_mut(k.borrow().idx())
    }

    pub fn iter(&self) -> vec_map::Iter<'_, V> {
        self.map.iter()
    }
    pub fn iter_mut(&mut self) -> vec_map::IterMut<V> {
        self.map.iter_mut()
    }
}

// map[key]
impl<K: Idx, V> Index<K> for IdxMap<K, V> {
    type Output = V;

    fn index(&self, k: K) -> &Self::Output {
        self.map.index(k.idx())
    }
}

// map[&key]
impl<K: Idx, V> Index<&K> for IdxMap<K, V> {
    type Output = V;

    fn index(&self, k: &K) -> &Self::Output {
        self.map.index(k.idx())
    }
}

// &mut map[key]
impl<K: Idx, V> IndexMut<K> for IdxMap<K, V> {
    fn index_mut(&mut self, k: K) -> &mut Self::Output {
        self.map.index_mut(k.idx())
    }
}

// &mut map[&key]
impl<K: Idx, V> IndexMut<&K> for IdxMap<K, V> {
    fn index_mut(&mut self, k: &K) -> &mut Self::Output {
        self.map.index_mut(k.idx())
    }
}
