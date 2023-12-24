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
