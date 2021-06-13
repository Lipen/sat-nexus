use std::env;
use std::path::PathBuf;

fn main() {
    generate_bindings_dynamic();
}

fn generate_bindings_dynamic() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("vendor/ipasir/ipasir.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .dynamic_library_name("ipasir")
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-ipasir.rs"))
        .expect("Could not write bindings!");
}
