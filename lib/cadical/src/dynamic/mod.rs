pub use self::cadical::*;

mod cadical;

pub mod ffi {
    pub use cadical_sys::dynamic::*;
}

#[cfg(test)]
mod tests;
