use std::env;
use std::path::PathBuf;

fn main() {
    generate_bindings_dynamic();
}

fn generate_bindings_dynamic() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("vendor/cadical/src/ccadical.h")
        .dynamic_library_name("cadical")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-ccadical.rs"))
        .expect("Could not write bindings!");
}
