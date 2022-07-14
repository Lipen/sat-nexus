pub mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(deref_nullptr)] // see https://github.com/rust-lang/rust-bindgen/issues/1651
    #![allow(clippy::style)]

    include!(concat!(env!("OUT_DIR"), "/bindings-minisat-dynamic.rs"));
    // include!("../_bindings-minisat-dynamic.rs");

    // `minisat.h` contains the following declaration:
    //   typedef opaque(int) minisat_bool;
    // However, in the implementation (`minisat.cc`) it is just a plain c-bool.
    // Hence, we blocklist `minisat_bool` in bindgen and declare its Rust counterpart manually.
    pub type minisat_bool = bool;
}

pub type MiniSatFFI = bindings::minisat;
pub type MiniSatPtr = *mut bindings::minisat_solver;

impl MiniSatFFI {
    pub fn load(name: &str) -> Self {
        unsafe { Self::new(libloading::library_filename(name)) }
            .unwrap_or_else(|e| panic!("Could not load shared library '{}': {}: {:?}", name, e, e))
    }

    pub fn init(&self) -> MiniSatPtr {
        unsafe { self.minisat_new() }
    }
}

macro_rules! instance {
    ($name:expr) => {{
        use ::once_cell::sync::OnceCell;
        static INSTANCE: OnceCell<MiniSatFFI> = OnceCell::new();
        INSTANCE.get_or_init(|| MiniSatFFI::load($name))
    }};
}

impl MiniSatFFI {
    pub fn instance() -> &'static Self {
        instance!("minisat-c")
    }
}

impl MiniSatFFI {
    pub fn minisat_l_true(&self) -> bindings::minisat_lbool {
        unsafe { self.minisat_get_l_True() }
    }
    pub fn minisat_l_false(&self) -> bindings::minisat_lbool {
        unsafe { self.minisat_get_l_False() }
    }
    pub fn minisat_l_undef(&self) -> bindings::minisat_lbool {
        unsafe { self.minisat_get_l_Undef() }
    }
}
