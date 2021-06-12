#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub type IpasirFFI = bindings::ipasir;
pub type IpasirPtr = *mut ::std::os::raw::c_void;

pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings-ipasir.rs"));
}
