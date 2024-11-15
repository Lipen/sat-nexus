use std::ops::Index;

#[derive(Debug)]
pub struct DomainVar<K, V> {
    pub keys: Vec<K>,
    pub values: Vec<V>,
}

impl<K, V> DomainVar<K, V> {
    pub fn new(keys: Vec<K>, values: Vec<V>) -> Self {
        assert_eq!(keys.len(), values.len());
        Self { keys, values }
    }
}

impl<K, V> Default for DomainVar<K, V> {
    fn default() -> Self {
        Self::new(Vec::new(), Vec::new())
    }
}

impl<K, V> DomainVar<K, V> {
    pub fn add(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn get(&self, key: &K) -> &V
    where
        K: Eq,
        K: std::fmt::Debug,
    {
        let i = self
            .keys
            .iter()
            .position(|k| k == key)
            .unwrap_or_else(|| panic!("key '{:?}' not found in {:?}", key, self.keys));
        &self.values[i]
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys.iter().zip(self.values.iter())
    }
}

impl<K, V> Index<K> for DomainVar<K, V>
where
    K: Eq,
    K: std::fmt::Debug,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(&index)
    }
}
