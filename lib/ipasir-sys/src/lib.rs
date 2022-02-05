use std::ffi::CStr;

pub mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(deref_nullptr)] // see https://github.com/rust-lang/rust-bindgen/issues/1651
    #![allow(clippy::style)]
    include!(concat!(env!("OUT_DIR"), "/bindings-ipasir.rs"));
}

pub type IpasirFFI = bindings::ipasir;
pub type IpasirPtr = *mut ::std::os::raw::c_void;

impl IpasirFFI {
    pub fn load(name: &str) -> Self {
        unsafe { Self::new(libloading::library_filename(name)) }
            .unwrap_or_else(|e| panic!("Could not load shared library '{}': {}", name, e))
    }

    pub fn init(&self) -> IpasirPtr {
        unsafe { self.ipasir_init() }
    }

    pub fn signature(&self) -> &'static str {
        let c_chars = unsafe { self.ipasir_signature() };
        let c_str = unsafe { CStr::from_ptr(c_chars) };
        c_str
            .to_str()
            .expect("The IPASIR implementation returned invalid UTF-8.")
    }
}

macro_rules! instance {
    ($name:expr) => {{
        use once_cell::sync::OnceCell;
        static FFI: OnceCell<IpasirFFI> = OnceCell::new();
        FFI.get_or_init(|| IpasirFFI::load($name))
    }};
}

impl IpasirFFI {
    pub fn instance_minisat() -> &'static Self {
        instance!("minisat")
    }
    pub fn instance_glucose() -> &'static Self {
        instance!("glucose")
    }
    pub fn instance_cadical() -> &'static Self {
        instance!("cadical")
    }
}
