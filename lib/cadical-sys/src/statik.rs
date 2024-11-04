#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::style)]

include!(concat!(env!("OUT_DIR"), "/bindings-ccadical-static.rs"));
// include!("../_bindings-ccadical-static.rs");

pub type CCadicalPtr = *mut CCaDiCaL;

#[cfg(test)]
mod tests {
    use super::*;

    use ffi_utils::cstr2str;

    #[test]
    fn test_signature() {
        let s = unsafe { cstr2str(ccadical_signature()) };
        println!("signature = {:?}", s);
        assert!(s.starts_with("cadical"));
    }

    #[test]
    fn test_init_and_release() {
        unsafe {
            let ptr = ccadical_init();
            println!("ptr = {:?}", ptr);
            assert!(!ptr.is_null());
            ccadical_release(ptr);
        }
    }
}
