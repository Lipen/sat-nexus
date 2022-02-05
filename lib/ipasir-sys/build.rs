use std::env;
use std::path::PathBuf;

fn main() {
    generate_bindings_dynamic();
}

fn generate_bindings_dynamic() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        // .header("vendor/ipasir/ipasir.h")
        .header("headers/ipasir.h")
        .dynamic_library_name("ipasir")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .layout_tests(false)
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-ipasir.rs"))
        .expect("Could not write bindings!");

    // Note: in order to build those bindings manually, use the following command:
    //   bindgen headers/ipasir.h -o _bindings-ipasir.rs --dynamic-loading ipasir --no-layout-tests
}
