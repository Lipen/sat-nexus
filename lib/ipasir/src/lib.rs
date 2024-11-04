//! Reimplementation of <https://github.com/Robbepop/ipasir-rs>

pub use self::ipasir::*;
pub use self::types::*;

mod ipasir;
mod types;

pub mod ffi {
    pub use ipasir_sys::*;
}

#[cfg(test)]
mod tests;
