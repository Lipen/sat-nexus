use std::env;
use std::path::PathBuf;

fn main() {
    generate_bindings_dynamic();
}

fn generate_bindings_dynamic() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        // .header("vendor/cadical/src/ccadical.h")
        .header("headers/ccadical.h")
        .dynamic_library_name("ccadical")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .layout_tests(false)
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-ccadical.rs"))
        .expect("Could not write bindings!");

    // Note: in order to build those bindings manually, use the following command:
    //   bindgen headers/ccadical.h -o _bindings-ccadical.rs --dynamic-loading ccadical --no-layout-tests
}
