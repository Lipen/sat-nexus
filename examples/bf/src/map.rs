use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Map<K, V> {
    pub keys: Vec<K>,
    pub values: Vec<V>,
}

impl<K, V> Map<K, V> {
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
    pub fn add(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys.iter().zip(self.values.iter())
    }
}

impl<K, V> Map<K, V>
where
    K: Eq,
    K: std::fmt::Debug,
{
    fn index_of(&self, key: &K) -> usize {
        self.keys
            .iter()
            .position(|k| k == key)
            .unwrap_or_else(|| panic!("key '{:?}' not found in {:?}", key, self.keys))
    }

    pub fn get(&self, key: &K) -> &V {
        let i = self.index_of(key);
        &self.values[i]
    }

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
