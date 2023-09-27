pub unsafe fn cstr2str(c_chars: *const std::ffi::c_char) -> &'static str {
    let c_str = std::ffi::CStr::from_ptr(c_chars);
    c_str.to_str().expect("Invalid UTF-8")
}
