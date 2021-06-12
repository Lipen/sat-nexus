use lazy_static::lazy_static;

pub use cadical_sys::*;

pub fn load_cadical(filename: &str) -> CadicalFFI {
    eprintln!("Loading cadical shared library `{}`...", filename);
    unsafe { CadicalFFI::new(filename) }.expect("Could not load shared library")
}

lazy_static! {
    pub static ref CADICAL: CadicalFFI = load_cadical("cadical");
}
