pub mod bindings;

pub type IpasirFFI = bindings::ipasir;
pub type IpasirPtr = *mut ::std::os::raw::c_void;

impl IpasirFFI {
    pub fn load(name: &str) -> Self {
        unsafe { Self::new(libloading::library_filename(name)) }
            .unwrap_or_else(|e| panic!("Could not load shared library '{}': {}: {:?}", name, e, e))
    }

    pub fn init(&self) -> IpasirPtr {
        unsafe { self.ipasir_init() }
    }

    pub fn signature(&self) -> &'static str {
        let c_chars = unsafe { self.ipasir_signature() };
        let c_str = unsafe { std::ffi::CStr::from_ptr(c_chars) };
        c_str.to_str().expect("The IPASIR implementation returned invalid UTF-8.")
    }
}
