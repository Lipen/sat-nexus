use std::any::type_name;
use std::borrow::Cow;
use std::collections::HashMap;

use snafu::Snafu;
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
    pub fn contains<T: 'static>(&self) -> bool {
        self.storage.contains::<T>()
    }

    pub fn entry<T: 'static>(&mut self) -> type_map::Entry<T> {
        self.storage.entry()
    }

    pub fn insert<T: 'static>(&mut self, value: T) -> Option<T> {
        self.storage.insert::<T>(value)
    }

    pub fn get<T: 'static>(&self) -> Result<&T> {
        self.storage.get::<T>().ok_or_else(|| ContextError::NoElementByType {
            type_name: type_name::<T>(),
        })
    }

    pub fn get_mut<T: 'static>(&mut self) -> Result<&mut T> {
        self.storage.get_mut::<T>().ok_or_else(|| ContextError::NoElementByType {
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

    pub fn get_named_mut<T: 'static, S>(&mut self, name: S) -> Result<&mut T>
    where
        S: Into<NamedStorageType>,
    {
        let map = self.get_mut::<NamedStorage<T>>()?;
        let name = name.into();
        map.get_mut(&name).ok_or(ContextError::NoElementByName { name })
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
