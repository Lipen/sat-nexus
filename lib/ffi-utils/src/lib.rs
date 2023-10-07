use std::ffi::{c_char, CStr};

pub unsafe fn cstr2str(c_chars: *const c_char) -> &'static str {
    let c_str = CStr::from_ptr(c_chars);
    c_str.to_str().expect("Invalid UTF-8")
}
