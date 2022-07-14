fn main() {
    #[cfg(feature = "dynamic")]
    {
        generate_bindings_dynamic();
    }

    #[cfg(feature = "static")]
    {
        generate_bindings_static();
        build_static_lib();
    }
}

#[cfg(feature = "dynamic")]
fn generate_bindings_dynamic() {
    println!("cargo:warning=Generating MiniSat dynamic bindings...");
    // Note: to generate these bindings manually, use the following command:
    //   bindgen vendor/minisat-c-bindings/minisat.h -o _bindings-minisat-dynamic.rs --dynamic-loading minisat --blocklist-type minisat_bool --allowlist-function minisat_.*
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("vendor/minisat-c-bindings/minisat.h")
        .dynamic_library_name("minisat")
        .dynamic_link_require_all(true)
        .blocklist_type("minisat_bool") // manually aliases to Rust's `bool`
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .layout_tests(false)
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-minisat-dynamic.rs"))
        .expect("Could not write bindings!");
}

#[cfg(feature = "static")]
fn generate_bindings_static() {
    println!("cargo:warning=Generating MiniSat static bindings...");
    // Note: to generate these bindings manually, use the following command:
    //   bindgen vendor/minisat-c-bindings/minisat.h -o _bindings-minisat-static.rs --blocklist-type minisat_bool --allowlist-function minisat_.*
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("vendor/minisat-c-bindings/minisat.h")
        .blocklist_type("minisat_bool") // manually aliases to Rust's `bool`
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-minisat-static.rs"))
        .expect("Could not write bindings!");
}

#[cfg(feature = "static")]
fn build_static_lib() {
    println!("cargo:warning=Building MiniSat static library...");
    cc::Build::new()
        .cpp(true)
        .include("vendor/minisat/minisat")
        .include("vendor/minisat")
        .include("vendor/zlib")
        .file("vendor/minisat/minisat/core/Solver.cc")
        .file("vendor/minisat/minisat/simp/SimpSolver.cc")
        .file("vendor/minisat/minisat/utils/System.cc")
        .file("vendor/minisat-c-bindings/minisat.cc")
        .define("__STDC_LIMIT_MACROS", None)
        .define("__STDC_FORMAT_MACROS", None)
        .warnings(false)
        .compile("minisat");
}
