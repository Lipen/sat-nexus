//! Reimplementation of <https://github.com/Robbepop/ipasir-rs>

pub use interface::*;
pub use types::*;

pub mod ffi;
mod interface;
pub mod solver;
mod types;

#[cfg(test)]
mod tests;
