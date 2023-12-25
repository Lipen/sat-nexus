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
    build_script::cargo_warning("Generating Cadical dynamic bindings...");

    // Note: to generate these bindings manually, use the following command:
    //   bindgen wrapper.h -o _bindings-ccadical-dynamic.rs --dynamic-loading ccadical --dynamic-link-require-all --allowlist-function ccadical_.* --no-layout-tests
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .dynamic_library_name("ccadical")
        .dynamic_link_require_all(true)
        .allowlist_function("ccadical_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .layout_tests(false)
        .generate()
        .expect("Could not create bindings!");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings-ccadical-dynamic.rs"))
        .expect("Could not write bindings!");
}

#[cfg(feature = "static")]
fn generate_bindings_static() {
    build_script::cargo_warning("Generating Cadical static bindings...");
    build_script::cargo_rerun_if_changed("wrapper.h");

    // Note: to generate these bindings manually, use the following command:
    //   bindgen wrapper.h -o _bindings-ccadical-static.rs --allowlist-function ccadical_.*
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .allowlist_function("ccadical_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Could not create bindings!");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings-ccadical-static.rs"))
        .expect("Could not write bindings!");
}

#[cfg(feature = "static")]
fn build_static_lib() {
    build_script::cargo_warning("Building Cadical static library...");
    build_script::cargo_rerun_if_changed("wrapper.h");

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
        .std("c++11")
        .files(files)
        .flag_if_supported("-Wno-nonnull-compare")
        .define("NDEBUG", None)
        .define("NBUILD", None)
        .define("NUNLOCKED", None)
        .compile("cadical");

    // On Windows, `psapi` is needed for `GetProcessMemoryInfo`
    if cfg!(windows) {
        build_script::cargo_rustc_link_lib("psapi");
    }
}
