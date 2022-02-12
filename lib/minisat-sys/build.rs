use std::env;
use std::path::PathBuf;

fn main() {
    build_static_lib();
    generate_bindings_static();
}

fn build_static_lib() {
    println!("cargo:warning=Building MiniSat static library...");
    cc::Build::new()
        .cpp(true)
        .include("vendor/minisat/minisat")
        .include("vendor/minisat")
        .file("vendor/minisat/minisat/core/Solver.cc")
        .file("vendor/minisat/minisat/simp/SimpSolver.cc")
        .file("vendor/minisat/minisat/utils/System.cc")
        .file("vendor/minisat-c-bindings/minisat.cc")
        .define("__STDC_LIMIT_MACROS", None)
        .define("__STDC_FORMAT_MACROS", None)
        .warnings(false)
        .compile("minisat");
}

fn generate_bindings_static() {
    println!("cargo:warning=Generating MiniSat static bindings...");
    // Note: to generate these bindings manually, use the following command:
    //   bindgen vendor/minisat-c-bindings/minisat.h -o _bindings-minisat.rs --blocklist-type minisat_bool
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("vendor/minisat-c-bindings/minisat.h")
        .blocklist_type("minisat_bool") // manually aliases to Rust's `bool`
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-minisat.rs"))
        .expect("Could not write bindings!");
}
