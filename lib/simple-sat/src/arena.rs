use std::ops::{Index, IndexMut};
use std::vec::IntoIter;

pub type Id = u32;

#[derive(Debug)]
pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn alloc(&mut self, item: T) -> Id {
        self.alloc_with_id(|_| item)
    }

    pub fn alloc_with_id<F>(&mut self, f: F) -> Id
    where
        F: FnOnce(Id) -> T,
    {
        let id = self.next_id();
        self.items.push(f(id));
        id
    }

    pub fn next_id(&self) -> Id {
        self.items.len() as Id + 1
    }

    pub fn get(&self, index: Id) -> &T {
        assert_ne!(index, 0);
        &self.items[index as usize - 1]
    }
    pub fn get_mut(&mut self, index: Id) -> &mut T {
        assert_ne!(index, 0);
        &mut self.items[index as usize - 1]
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<Id> for Arena<T> {
    type Output = T;

    fn index(&self, index: Id) -> &Self::Output {
        self.get(index)
    }
}

impl<T> IndexMut<Id> for Arena<T> {
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        self.get_mut(index)
    }
}

impl<T> IntoIterator for Arena<T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
