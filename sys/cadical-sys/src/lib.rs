pub mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/bindings-ccadical.rs"));
}

pub type CCadicalFFI = bindings::ccadical;
pub type CCadicalPtr = *mut bindings::CCaDiCaL;

impl CCadicalFFI {
    pub fn load(name: &str) -> Self {
        unsafe { Self::new(libloading::library_filename(name)) }
            .unwrap_or_else(|e| panic!("Could not load shared library '{}': {}", name, e))
    }
}

macro_rules! instance {
    ($name:expr) => {{
        use once_cell::sync::OnceCell;
        static FFI: OnceCell<CCadicalFFI> = OnceCell::new();
        FFI.get_or_init(|| CCadicalFFI::load($name))
    }};
}

impl CCadicalFFI {
    pub fn instance() -> &'static Self {
        instance!("cadical")
    }
}
