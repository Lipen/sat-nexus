pub use self::kissat::*;

mod kissat;

pub mod ffi {
    pub use kissat_sys::statik::*;
}

#[cfg(test)]
mod tests;
