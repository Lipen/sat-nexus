#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(deref_nullptr)] // see https://github.com/rust-lang/rust-bindgen/issues/1651
#![allow(clippy::style)]

include!(concat!(env!("OUT_DIR"), "/bindings-minisat.rs"));
// include!("../_bindings-minisat.rs");

// `minisat.h` contains the following declaration:
//   typedef opaque(int) minisat_bool;
// However, in the implementation (`minisat.cc`) it is just a plain c-bool.
// Hence, we blocklist `minisat_bool` in bindgen and declare its Rust counterpart manually.
pub type minisat_bool = bool;
