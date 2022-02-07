use std::any::type_name;
use std::borrow::Cow;
use std::collections::HashMap;

use type_map::TypeMap;

type NamedStorageType = Cow<'static, str>;
type NamedStorage<T> = HashMap<NamedStorageType, T>;

#[derive(Debug)]
pub struct Context {
    storage: TypeMap,
}

impl Context {
    pub fn new() -> Self {
        Self {
            storage: TypeMap::new(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl From<TypeMap> for Context {
    fn from(map: TypeMap) -> Self {
        Context { storage: map }
    }
}

impl Context {
    pub fn insert<T: 'static>(&mut self, value: T) -> Option<T>
    where
        T: std::fmt::Debug,
    {
        self.storage.insert::<T>(value)
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.storage.get::<T>()
    }

    pub fn extract<T: 'static>(&self) -> &T {
        self.get::<T>()
            .unwrap_or_else(|| panic!("Could not extract {}", type_name::<T>()))
    }

    pub fn insert_named<T: 'static, S>(&mut self, name: S, value: T) -> Option<T>
    where
        S: Into<NamedStorageType>,
    {
        self.storage
            .entry::<NamedStorage<T>>()
            .or_insert_with(NamedStorage::new)
            .insert(name.into(), value)
    }

    pub fn get_named<T: 'static, S>(&self, name: S) -> Option<&T>
    where
        S: Into<NamedStorageType>,
    {
        let map = self.storage.get::<NamedStorage<T>>()?;
        map.get(&name.into())
    }

    pub fn extract_named<T: 'static, S>(&self, name: S) -> &T
    where
        S: Into<NamedStorageType>,
    {
        // Note: we have to `clone` here because of the second use of `name` in `unwrap_or_else`.
        let name = name.into();
        self.get_named(name.clone())
            .unwrap_or_else(|| panic!("Could not extract {} with name `{}`", type_name::<T>(), name))
    }
}

#[cfg(test)]
mod tests;
