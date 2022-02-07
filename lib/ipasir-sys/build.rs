use std::env;
use std::path::PathBuf;

fn main() {
    generate_bindings_dynamic();
}

fn generate_bindings_dynamic() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("headers/ipasir.h")
        .dynamic_library_name("ipasir")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-ipasir.rs"))
        .expect("Could not write bindings!");

    // Note: to generate these bindings manually, use the following command:
    //   bindgen headers/ipasir.h -o _bindings-ipasir.rs --dynamic-loading ipasir
}
