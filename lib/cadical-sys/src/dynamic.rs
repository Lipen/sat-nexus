pub mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(deref_nullptr)] // see https://github.com/rust-lang/rust-bindgen/issues/1651
    #![allow(clippy::style)]

    include!(concat!(env!("OUT_DIR"), "/bindings-ccadical-dynamic.rs"));
    // include!("../_bindings-ccadical-dynamic.rs");
}

pub type CCadicalFFI = bindings::ccadical;
pub type CCadicalPtr = *mut bindings::CCaDiCaL;

impl CCadicalFFI {
    pub fn load(name: &str) -> Self {
        unsafe { Self::new(libloading::library_filename(name)) }
            .unwrap_or_else(|e| panic!("Could not load shared library '{}': {}", name, e))
    }

    pub fn init(&self) -> CCadicalPtr {
        unsafe { self.ccadical_init() }
    }

    pub fn signature(&self) -> &'static str {
        let c_chars = unsafe { self.ccadical_signature() };
        let c_str = unsafe { std::ffi::CStr::from_ptr(c_chars) };
        c_str.to_str().expect("The implementation returned invalid UTF-8.")
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
