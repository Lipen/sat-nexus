#![allow(dead_code)]

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::slice;

use crate::lit::Lit;
use crate::var::Var;

pub type VarMap<V> = IdxMap<Var, V>;
pub type LitMap<V> = IdxMap<Lit, V>;
pub type VarVec<V> = IdxVec<Var, V>;
pub type LitVec<V> = IdxVec<Lit, V>;
pub type VarHeap = IdxHeap<Var>;

// ==========================================

pub trait Idx {
    fn idx(&self) -> usize;
}

impl Idx for Var {
    fn idx(&self) -> usize {
        self.0 as usize
    }
}

impl Idx for Lit {
    fn idx(&self) -> usize {
        self.0 as usize
    }
}

impl<I> Idx for I
where
    I: num_traits::NumCast,
{
    fn idx(&self) -> usize {
        self.to_usize().unwrap()
    }
}

// ==========================================

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

impl<K: Idx, V> Debug for IdxMap<K, V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.map.iter()).finish()
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

// map[key] = (value)
impl<K: Idx, V> IndexMut<K> for IdxMap<K, V> {
    fn index_mut(&mut self, k: K) -> &mut Self::Output {
        self.map.index_mut(k.idx())
    }
}
// map[&key] = (value)
impl<K: Idx, V> IndexMut<&K> for IdxMap<K, V> {
    fn index_mut(&mut self, k: &K) -> &mut Self::Output {
        self.map.index_mut(k.idx())
    }
}

// ==========================================

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

// ==========================================

#[derive(Debug)]
pub struct IdxHeap<K: Idx> {
    heap: Vec<K>,
    index: IdxVec<K, usize>,
}

impl<K: Idx> IdxHeap<K> {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            index: IdxVec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn contains(&self, key: &K) -> bool {
        // Note: `usize::MAX` is a sentinel value to represent the absence of an element.
        self.index.contains_key(key) && self.index[key] != usize::MAX
    }

    pub fn clear(&mut self) {
        self.heap.clear();
        self.index.clear();
    }

    pub fn parent(&self, i: usize) -> usize {
        (i - 1) >> 1
    }
    pub fn left(&self, i: usize) -> usize {
        2 * i + 1
    }
    pub fn right(&self, i: usize) -> usize {
        2 * i + 2
    }

    /// Peek the top item in the heap.
    pub fn peek(&self) -> Option<&K> {
        self.heap.get(0)
    }
}

impl<K: Idx + Ord> IdxHeap<K> {
    fn ord_cmp(a: &K, b: &K) -> bool {
        // Max-heap
        a.cmp(b) == Ordering::Greater
        // Min-heap
        // a.cmp(b) == Ordering::Less
    }

    pub fn insert(&mut self, key: K) -> bool {
        self.insert_by(key, Self::ord_cmp)
    }
    pub fn pop(&mut self) -> Option<K> {
        self.pop_by(Self::ord_cmp)
    }
    pub fn update(&mut self, key: K) {
        self.update_by(key, Self::ord_cmp)
    }
    fn sift_up(&mut self, i: usize) {
        self.sift_up_by(i, Self::ord_cmp)
    }
    fn sift_down(&mut self, i: usize) {
        self.sift_down_by(i, Self::ord_cmp)
    }

    fn into_sorted_iter(self) -> IdxHeapIntoSortedIter<K, fn(&K, &K) -> bool> {
        self.into_sorted_iter_by(Self::ord_cmp)
    }
    fn into_sorted_vec(self) -> Vec<K> {
        self.into_sorted_vec_by(Self::ord_cmp)
    }
}

impl<K: Idx> IdxHeap<K> {
    /// Given a vector of keys, create a new heap with those keys, using
    /// the given comparison function to determine the order of the keys.
    ///
    /// The first thing we do is create a new heap with the given keys.
    /// We then iterate over the keys in reverse order, calling `sift_down_by`
    /// on each key. This is the same as calling `sift_down` on each key,
    /// except that we pass in the comparison function as an argument.
    ///
    /// **Arguments:**
    ///
    /// * `from`: The vector to heapify.
    /// * `cmp`: A function that takes two elements of the heap and
    /// returns true if the first element is less than the second element.
    ///
    /// **Returns:**
    ///
    /// * A min-heap, if the comparator (`cmp`) is "less-than".
    /// * A max-heap, if the comparator (`cmp`) is "greater-than".
    pub fn heapify_by<F>(from: Vec<K>, cmp: F) -> Self
    where
        F: Fn(&K, &K) -> bool,
    {
        let mut this = Self {
            heap: from,
            index: IdxVec::new(),
        };
        for i in (0..this.len()).rev() {
            this.sift_down_by(i, &cmp);
        }
        this
    }

    /// Insert the value (`key`) into the heap.
    ///
    /// - Min-heap, when "less-than" comparator (`cmp`) is used: `cmp = |&a, &b| a < b`.
    /// - Max-heap, when "greater-than" comparator (`cmp`) is used: `cmp = |&a, &b| a > b`.
    ///
    /// Returns `false` if `key` is already in the heap.
    pub fn insert_by<F>(&mut self, key: K, cmp: F) -> bool
    where
        F: Fn(&K, &K) -> bool,
    {
        if !self.contains(&key) {
            let i = self.heap.len();
            // Note: `usize::MAX` is a sentinel value to represent the absence of an element.
            self.index.init_by(&key, || usize::MAX);
            self.heap.push(key);
            self.sift_up_by(i, cmp);
            true
        } else {
            false
        }
    }

    /// Remove the top item from the heap.
    ///
    /// - If the comparator (`cmp`) if "less-than" (min-heap), the returned item is the **minimum**.
    /// - If the comparator (`cmp`) if "greater-than" (max-heap), the returned item is the **maximum**.
    ///
    /// Returns [`None`] if the heap is empty. Otherwise, returns [`Some(item)`][Some]
    /// with the top item with respect to the comparator (`cmp`).
    pub fn pop_by<F>(&mut self, cmp: F) -> Option<K>
    where
        F: Fn(&K, &K) -> bool,
    {
        if self.heap.is_empty() {
            None
        } else {
            let res = self.heap.swap_remove(0);
            self.index[&res] = usize::MAX;
            if !self.heap.is_empty() {
                self.index[&self.heap[0]] = 0;
                self.sift_down_by(0, cmp);
            }
            Some(res)
        }
    }

    /// Update the value (`key`) in the heap using the provided comparator (`cmp`).
    ///
    /// Panics if `key` is not present in the heap.
    pub fn update_by<F>(&mut self, key: K, cmp: F)
    where
        F: Fn(&K, &K) -> bool,
    {
        let i = self.index[&key];
        if i == usize::MAX {
            panic!("The key (key.idx() = {}) is missing from the heap", key.idx());
        }
        self.sift_down_by(i, &cmp);
        self.sift_up_by(i, cmp);
    }
    pub fn decrease_by<F>(&mut self, key: K, cmp: F)
    where
        F: Fn(&K, &K) -> bool,
    {
        let i = self.index[key];
        self.sift_up_by(i, cmp);
    }
    pub fn increase_by<F>(&mut self, key: K, cmp: F)
    where
        F: Fn(&K, &K) -> bool,
    {
        let i = self.index[key];
        self.sift_down_by(i, cmp);
    }

    fn sift_up_by<F>(&mut self, mut i: usize, cmp: F)
    where
        F: Fn(&K, &K) -> bool,
    {
        while i > 0 {
            let p = self.parent(i);
            if cmp(&self.heap[i], &self.heap[p]) {
                self.index[&self.heap[p]] = i;
                self.heap.swap(i, p);
                i = p;
            } else {
                break;
            }
        }
        self.index[&self.heap[i]] = i;
    }

    fn sift_down_by<F>(&mut self, mut i: usize, cmp: F)
    where
        F: Fn(&K, &K) -> bool,
    {
        loop {
            let l = self.left(i);
            if l >= self.heap.len() {
                break;
            }
            let r = self.right(i);
            let c = if r < self.heap.len() && cmp(&self.heap[r], &self.heap[l]) {
                r
            } else {
                l
            };

            if cmp(&self.heap[c], &self.heap[i]) {
                self.index[&self.heap[c]] = i;
                self.heap.swap(c, i);
                i = c;
            } else {
                break;
            }
        }
        self.index[&self.heap[i]] = i;
    }

    pub fn sorted_iter_by<F>(&mut self, cmp: F) -> IdxHeapSortedIter<K, F>
    where
        F: Fn(&K, &K) -> bool,
    {
        IdxHeapSortedIter { heap: self, cmp }
    }

    pub fn into_sorted_iter_by<F>(self, cmp: F) -> IdxHeapIntoSortedIter<K, F>
    where
        F: Fn(&K, &K) -> bool,
    {
        IdxHeapIntoSortedIter { heap: self, cmp }
    }

    pub fn into_sorted_vec_by<F>(mut self, cmp: F) -> Vec<K>
    where
        F: Fn(&K, &K) -> bool,
    {
        let mut res = Vec::with_capacity(self.len());
        while let Some(k) = self.pop_by(&cmp) {
            res.push(k);
        }
        res
    }
}

impl<K: Idx> Index<usize> for IdxHeap<K> {
    type Output = K;

    fn index(&self, i: usize) -> &Self::Output {
        self.heap.index(i)
    }
}

// ==========================================

pub struct IdxHeapSortedIter<'a, K: Idx, F>
where
    F: Fn(&K, &K) -> bool,
{
    heap: &'a mut IdxHeap<K>,
    cmp: F,
}

impl<'a, K: Idx, F> Iterator for IdxHeapSortedIter<'a, K, F>
where
    F: Fn(&K, &K) -> bool,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.heap.pop_by(&self.cmp)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.heap.len(), Some(self.heap.len()))
    }
}

impl<'a, K: Idx, F> ExactSizeIterator for IdxHeapSortedIter<'a, K, F>
where
    F: Fn(&K, &K) -> bool,
{
    fn len(&self) -> usize {
        self.heap.len()
    }
}

// ==========================================

pub struct IdxHeapIntoSortedIter<K: Idx, F>
where
    F: Fn(&K, &K) -> bool,
{
    heap: IdxHeap<K>,
    cmp: F,
}

impl<K: Idx, F> Iterator for IdxHeapIntoSortedIter<K, F>
where
    F: Fn(&K, &K) -> bool,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.heap.pop_by(&self.cmp)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.heap.len(), Some(self.heap.len()))
    }
}

impl<K: Idx, F> ExactSizeIterator for IdxHeapIntoSortedIter<K, F>
where
    F: Fn(&K, &K) -> bool,
{
    fn len(&self) -> usize {
        self.heap.len()
    }
}

// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_heap_insert_by() {
        let mut heap = IdxHeap::<u32>::new();
        let cmp = |&a: &u32, &b: &u32| a < b;
        heap.insert_by(3, &cmp);
        heap.insert_by(1, &cmp);
        heap.insert_by(5, &cmp);
        heap.insert_by(2, &cmp);
        heap.insert_by(4, &cmp);
        println!("heap = {:?}", heap);
        assert_eq!(heap.heap, vec![1, 2, 5, 3, 4]);

        let sorted = heap.into_sorted_iter_by(&cmp).collect::<Vec<_>>();
        println!("sorted = {:?}", sorted);
        assert_eq!(sorted, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_max_heap_insert_by() {
        let mut heap = IdxHeap::<u32>::new();
        let cmp = |&a: &u32, &b: &u32| a > b;
        heap.insert_by(3, &cmp);
        heap.insert_by(1, &cmp);
        heap.insert_by(5, &cmp);
        heap.insert_by(2, &cmp);
        heap.insert_by(4, &cmp);
        println!("heap = {:?}", heap);
        assert_eq!(heap.heap, vec![5, 4, 3, 1, 2]);

        let sorted = heap.into_sorted_iter_by(&cmp).collect::<Vec<_>>();
        println!("sorted = {:?}", sorted);
        assert_eq!(sorted, vec![5, 4, 3, 2, 1]);
    }

    #[test]
    fn test_heap_insert() {
        let mut heap = IdxHeap::<u32>::new();
        heap.insert(3);
        heap.insert(1);
        heap.insert(5);
        heap.insert(2);
        heap.insert(4);
        println!("heap = {:?}", heap);
        assert_eq!(heap.heap, vec![5, 4, 3, 1, 2]);

        let sorted = heap.into_sorted_vec();
        println!("sorted = {:?}", sorted);
        assert_eq!(sorted, vec![5, 4, 3, 2, 1]);
    }
}
