//! Reimplementation of <https://github.com/Robbepop/ipasir-rs>

pub use self::ipasir::*;
pub use self::solver::*;
pub use self::types::*;

pub mod ffi {
    pub use ipasir_sys::*;
}

mod ipasir;
mod solver;
mod types;

#[cfg(test)]
mod tests;
