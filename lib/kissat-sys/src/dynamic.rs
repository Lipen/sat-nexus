pub mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clippy::all)]

    include!(concat!(env!("OUT_DIR"), "/bindings-kissat-dynamic.rs"));
    // include!("../_bindings-kissat-dynamic.rs");
}

pub type KissatFFI = bindings::kissat_ffi;
pub type KissatPtr = *mut bindings::kissat;

impl KissatFFI {
    pub fn load(name: &str) -> Self {
        unsafe { Self::new(libloading::library_filename(name)) }
            .unwrap_or_else(|e| panic!("Could not load shared library '{}': {}: {:?}", name, e, e))
    }

    pub fn instance() -> &'static Self {
        use ::std::sync::OnceLock;
        static INSTANCE: OnceLock<KissatFFI> = OnceLock::new();
        INSTANCE.get_or_init(|| Self::load("kissat"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ffi_utils::cstr2str;

    #[test]
    fn test_signature() {
        let ffi = KissatFFI::instance();
        let s = unsafe { cstr2str(ffi.kissat_signature()) };
        println!("signature = {:?}", s);
        assert!(s.starts_with("kissat"));
    }

    #[test]
    fn test_init_and_release() {
        let ffi = KissatFFI::instance();
        let ptr = unsafe { ffi.kissat_init() };
        assert!(!ptr.is_null());
        unsafe { ffi.kissat_release(ptr) };
    }
}
