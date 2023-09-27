use std::env;
use std::path::PathBuf;

fn main() {
    generate_bindings_dynamic();
}

fn generate_bindings_dynamic() {
    build_script::cargo_warning("Building IPASIR dynamic bindings...");
    build_script::cargo_rerun_if_changed("headers/ipasir.h");

    // Note: to generate these bindings manually, use the following command:
    //   bindgen headers/ipasir.h -o _bindings-ipasir.rs --dynamic-loading ipasir --dynamic-link-require-all --allowlist-function ipasir_.* --no-layout-tests
    let bindings = bindgen::builder()
        .header("headers/ipasir.h")
        .dynamic_library_name("ipasir")
        .dynamic_link_require_all(true)
        .allowlist_function("ipasir_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .layout_tests(false)
        .generate()
        .expect("Could not create bindings!");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings-ipasir.rs"))
        .expect("Could not write bindings!");
}
