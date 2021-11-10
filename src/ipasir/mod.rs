//! Reimplementation of <https://github.com/Robbepop/ipasir-rs>

pub use interface::*;
pub use types::*;

pub mod ffi;
pub mod solver;

mod interface;
mod types;

#[cfg(test)]
mod tests;
