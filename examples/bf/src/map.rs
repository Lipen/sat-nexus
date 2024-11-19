use std::iter::zip;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Map<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K, V> Map<K, V> {
    /// Creates a new `Map` from the given keys and values.
    pub fn new(keys: Vec<K>, values: Vec<V>) -> Self {
        assert_eq!(keys.len(), values.len());
        Self { keys, values }
    }
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self::new(Vec::new(), Vec::new())
    }
}

impl<K, V> Map<K, V> {
    /// Adds a new key-value pair to the map.
    pub fn add(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.values.push(value);
    }

    /// Return the size of the map.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns `true` if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Returns a reference to the keys in the map.
    pub fn keys(&self) -> &[K] {
        &self.keys
    }

    /// Returns a reference to the values in the map.
    pub fn values(&self) -> &[V] {
        &self.values
    }

    /// Returns an iterator over the key-value pairs in the map.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        zip(&self.keys, &self.values)
    }
}

impl<K, V> Map<K, V>
where
    K: Eq,
    K: std::fmt::Debug,
{
    fn position(&self, key: &K) -> Option<usize> {
        self.keys.iter().position(|k| k == key)
    }

    fn index_of(&self, key: &K) -> usize {
        self.position(key)
            .unwrap_or_else(|| panic!("key '{:?}' not found in {:?}", key, self.keys))
    }

    /// Retrieves an immutable reference to the value associated with the given key.
    pub fn get(&self, key: &K) -> &V {
        let i = self.index_of(key);
        &self.values[i]
    }

    /// Retrieves a mutable reference to the value associated with the given key.
    pub fn get_mut(&mut self, key: &K) -> &mut V {
        let i = self.index_of(key);
        &mut self.values[i]
    }
}

impl<K, V> Index<K> for Map<K, V>
where
    K: Eq,
    K: std::fmt::Debug,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(&index)
    }
}

impl<K, V> IndexMut<K> for Map<K, V>
where
    K: Eq,
    K: std::fmt::Debug,
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(&index)
    }
}
