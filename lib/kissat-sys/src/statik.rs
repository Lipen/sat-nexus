#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings-kissat-static.rs"));
// include!("../_bindings-kissat-static.rs");

#[cfg(test)]
mod tests {
    use super::*;

    use ffi_utils::cstr2str;

    #[test]
    fn test_signature() {
        unsafe {
            let s = cstr2str(kissat_signature());
            println!("signature = {:?}", s);
            assert!(s.starts_with("kissat"));
        }
    }

    #[test]
    fn test_init_and_release() {
        unsafe {
            let ptr = kissat_init();
            println!("ptr = {:?}", ptr);
            assert!(!ptr.is_null());
            kissat_release(ptr);
        }
    }
}
