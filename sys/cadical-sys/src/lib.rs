#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub type CadicalFFI = bindings::cadical;
pub type CadicalPtr = *mut bindings::CCaDiCaL;

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings-ccadical.rs"));
}
