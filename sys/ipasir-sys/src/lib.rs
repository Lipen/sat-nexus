pub mod bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clippy::all)]
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
