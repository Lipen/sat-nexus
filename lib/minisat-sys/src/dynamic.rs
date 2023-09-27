pub mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clippy::style)]

    include!(concat!(env!("OUT_DIR"), "/bindings-cminisat-dynamic.rs"));
    // include!("../_bindings-cminisat-dynamic.rs");

    // `minisat.h` contains the following declaration:
    //   typedef opaque(int) minisat_bool;
    // However, in the implementation (`minisat.cc`) it is just a plain c-bool.
    // Hence, we blocklist `minisat_bool` in bindgen and declare its Rust counterpart manually.
    pub type minisat_bool = bool;
}

pub type CMiniSatFFI = bindings::cminisat;
pub type CMiniSatPtr = *mut bindings::minisat_solver;

impl CMiniSatFFI {
    pub fn load(name: &str) -> Self {
        unsafe { Self::new(libloading::library_filename(name)) }
            .unwrap_or_else(|e| panic!("Could not load shared library '{}': {}: {:?}", name, e, e))
    }

    pub fn instance() -> &'static Self {
        use ::once_cell::sync::OnceCell;
        static INSTANCE: OnceCell<CMiniSatFFI> = OnceCell::new();
        INSTANCE.get_or_init(|| Self::load("minisat-c"))
    }
}
