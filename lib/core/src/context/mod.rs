use snafu::Snafu;
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
        Self { storage: TypeMap::new() }
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
    pub fn insert<T: 'static>(&mut self, value: T) -> Option<T> {
        self.storage.insert::<T>(value)
    }

    pub fn get<T: 'static>(&self) -> Result<&T> {
        self.storage.get::<T>().ok_or_else(|| ContextError::NoElementByType {
            type_name: type_name::<T>(),
        })
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

    pub fn get_named<T: 'static, S>(&self, name: S) -> Result<&T>
    where
        S: Into<NamedStorageType>,
    {
        let map = self.get::<NamedStorage<T>>()?;
        let name = name.into();
        map.get(&name).ok_or(ContextError::NoElementByName { name })
    }
}

pub type Result<T, E = ContextError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[allow(clippy::enum_variant_names)]
pub enum ContextError {
    #[snafu(display("No element of type {}", type_name))]
    NoElementByType { type_name: &'static str },

    #[snafu(display("No element with name {}", name))]
    NoElementByName { name: NamedStorageType },
}

#[cfg(test)]
mod tests;
