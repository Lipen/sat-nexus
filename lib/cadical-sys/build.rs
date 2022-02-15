use std::env;
use std::path::PathBuf;

fn main() {
    if cfg!(feature = "static") {
        build_static_lib();
        generate_bindings_static();
    }

    if cfg!(feature = "dynamic") {
        generate_bindings_dynamic();
    }
}

fn build_static_lib() {
    println!("cargo:warning=Building Cadical static library...");
    let files = glob::glob("vendor/cadical/src/*.cpp")
        .expect("Bad glob")
        .map(|p| p.expect("Could not read file in glob"))
        .filter(|p| {
            let name = p.file_name().expect("Could not get file name");
            name != "cadical.cpp" && name != "mobical.cpp"
        })
        .collect::<Vec<_>>();
    cc::Build::new()
        .cpp(true)
        .files(files)
        .define("NDEBUG", None)
        .define("NBUILD", None)
        .define("NUNLOCKED", None)
        .compile("cadical");

    // On Windows, `psapi` is needed for `GetProcessMemoryInfo`
    if cfg!(windows) {
        println!("cargo:rustc-link-lib=psapi");
    }
}

fn generate_bindings_static() {
    println!("cargo:warning=Generating Cadical static bindings...");
    // Note: to generate these bindings manually, use the following command:
    //   bindgen vendor/cadical/src/ccadical.h -o _bindings-ccadical-static.rs --allowlist-function ccadical_.*
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("vendor/cadical/src/ccadical.h")
        .allowlist_function("ccadical_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-ccadical-static.rs"))
        .expect("Could not write bindings!");
}

fn generate_bindings_dynamic() {
    println!("cargo:warning=Generating Cadical dynamic bindings...");
    // Note: to generate these bindings manually, use the following command:
    //   bindgen vendor/cadical/src/ccadical.h -o _bindings-ccadical-dynamic.rs --dynamic-loading ccadical  --allowlist-function ccadical_.*
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("vendor/cadical/src/ccadical.h")
        .dynamic_library_name("ccadical")
        .allowlist_function("ccadical_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .layout_tests(false)
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-ccadical-dynamic.rs"))
        .expect("Could not write bindings!");
}
