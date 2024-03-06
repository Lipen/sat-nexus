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
    build_script::cargo_warning("Generating MiniSat dynamic bindings...");
    build_script::cargo_rerun_if_changed("wrapper.h");

    // Note: to generate these bindings manually, use the following command:
    //   bindgen wrapper.h -o _bindings-cminisat-dynamic.rs --dynamic-loading cminisat --dynamic-link-require-all --no-layout-tests --allowlist-item "minisat_.*"
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .dynamic_library_name("cminisat")
        .dynamic_link_require_all(true)
        .allowlist_item("minisat_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .layout_tests(false)
        .generate()
        .expect("Could not create bindings!");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings-cminisat-dynamic.rs"))
        .expect("Could not write bindings!");
}

#[cfg(feature = "static")]
fn generate_bindings_static() {
    build_script::cargo_warning("Generating MiniSat static bindings...");
    build_script::cargo_rerun_if_changed("wrapper.h");

    // Note: to generate these bindings manually, use the following command:
    //   bindgen wrapper.h -o _bindings-cminisat-static.rs --allowlist-item "minisat_.*"
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .allowlist_item("minisat_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Could not create bindings!");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings-cminisat-static.rs"))
        .expect("Could not write bindings!");
}

#[cfg(feature = "static")]
fn build_static_lib() {
    build_script::cargo_warning("Building MiniSat static library...");

    cc::Build::new()
        .cpp(true)
        .include("vendor/minisat/minisat")
        .include("vendor/minisat")
        .file("vendor/minisat/minisat/capi/cminisat.cc")
        .file("vendor/minisat/minisat/core/Solver.cc")
        .file("vendor/minisat/minisat/simp/SimpSolver.cc")
        .file("vendor/minisat/minisat/utils/System.cc")
        .define("__STDC_LIMIT_MACROS", None)
        .define("__STDC_FORMAT_MACROS", None)
        .warnings(false)
        .compile("minisat");
}
