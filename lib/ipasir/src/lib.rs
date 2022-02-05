//! Reimplementation of <https://github.com/Robbepop/ipasir-rs>

pub use api::*;
pub use types::*;

pub mod ffi;
pub mod solver;

mod api;
mod types;

#[cfg(test)]
mod tests;
