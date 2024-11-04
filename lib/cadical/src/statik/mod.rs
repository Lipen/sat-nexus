pub use self::cadical::*;

mod cadical;

pub mod ffi {
    pub use cadical_sys::statik::*;
}

#[cfg(test)]
mod tests;
