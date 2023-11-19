#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::style)]

include!(concat!(env!("OUT_DIR"), "/bindings-espresso-static.rs"));
// include!("../_bindings-espresso-static.rs");

#[cfg(test)]
mod tests {
    use std::fs::{File, OpenOptions};
    use std::os::windows::prelude::*;
    use std::ptr::null_mut;

    use super::*;

    #[test]
    fn test_version() {
        println!("VERSION = {:?}", std::ffi::CStr::from_bytes_with_nul(VERSION).unwrap())
    }

    #[test]
    fn test_run_espresso() {
        unsafe {
            // read_pla(fpla, TRUE as _, TRUE as _, FD_type as _, &mut pla);
            let mut pla = new_PLA();
            println!("pla = {:?}", pla);
            (*pla).pla_type = FD_type as _;
        }
    }
}
