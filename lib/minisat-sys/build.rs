use std::env;
use std::path::PathBuf;

fn main() {
    generate_bindings_dynamic();
}

fn generate_bindings_dynamic() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("headers/minisat.h")
        .dynamic_library_name("minisat")
        .dynamic_link_require_all(true)
        .blocklist_type("minisat_bool") // manually aliases to Rust's `bool`
        .blocklist_item("minisat_l_.*") // unnecessary `extern const`s
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-minisat.rs"))
        .expect("Could not write bindings!");

    // Note: to generate these bindings manually, use the following command:
    //   bindgen headers/minisat.h -o _bindings-minisat.rs --dynamic-loading minisat --blocklist-type minisat_bool --blocklist-item minisat_l_.*
}
