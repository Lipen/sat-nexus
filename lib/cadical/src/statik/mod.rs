pub use cadical_sys::statik as ffi;

pub use self::cadical::*;

mod cadical;

#[cfg(test)]
mod tests;
