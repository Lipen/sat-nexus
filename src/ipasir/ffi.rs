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

// pub unsafe fn ipasir_signature() -> *const ::std::os::raw::c_char {
//     CADICAL.ipasir_signature()
// }
//
// pub unsafe fn ipasir_init() -> *mut ::std::os::raw::c_void {
//     CADICAL.ipasir_init()
// }
//
// pub unsafe fn ipasir_release(solver: *mut ::std::os::raw::c_void) -> () {
//     CADICAL.ipasir_release(solver)
// }
//
// pub unsafe fn ipasir_add(solver: *mut ::std::os::raw::c_void, lit_or_zero: i32) -> () {
//     CADICAL.ipasir_add(solver, lit_or_zero)
// }
//
// pub unsafe fn ipasir_assume(solver: *mut ::std::os::raw::c_void, lit: i32) -> () {
//     CADICAL.ipasir_assume(solver, lit)
// }
//
// pub unsafe fn ipasir_solve(solver: *mut ::std::os::raw::c_void) -> ::std::os::raw::c_int {
//     CADICAL.ipasir_solve(solver)
// }
//
// pub unsafe fn ipasir_val(solver: *mut ::std::os::raw::c_void, lit: i32) -> i32 {
//     CADICAL.ipasir_val(solver, lit)
// }
//
// pub unsafe fn ipasir_failed(
//     solver: *mut ::std::os::raw::c_void,
//     lit: i32,
// ) -> ::std::os::raw::c_int {
//     CADICAL.ipasir_failed(solver, lit)
// }
