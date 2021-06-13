use lazy_static::lazy_static;

pub use ipasir_sys::*;

pub fn load_ipasir(filename: &str) -> IpasirFFI {
    eprintln!("Loading ipasir shared library `{}`...", filename);
    unsafe { IpasirFFI::new(filename) }.expect("Could not load shared library")
}

lazy_static! {
    pub static ref MINISAT: IpasirFFI = load_ipasir("minisat");
    pub static ref GLUCOSE: IpasirFFI = load_ipasir("glucose");
    pub static ref CADICAL: IpasirFFI = load_ipasir("cadical");
}
