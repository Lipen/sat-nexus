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
    //   bindgen wrapper.h -o _bindings-ccadical-dynamic.rs --dynamic-loading ccadical --no-layout-tests --allowlist-function "ccadical_.*"
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .dynamic_library_name("ccadical")
        // .dynamic_link_require_all(true) // Note: fails in runtime on ccadical instantiation
        .allowlist_function("ccadical_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
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
    //   bindgen wrapper.h -o _bindings-ccadical-static.rs --allowlist-function "ccadical_.*"
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .allowlist_function("ccadical_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Could not create bindings!");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings-ccadical-static.rs"))
        .expect("Could not write bindings!");
}

#[cfg(feature = "static")]
fn build_static_lib() {
    use std::path::Path;
    use std::process::Command;

    build_script::cargo_warning("Building Cadical static library...");
    build_script::cargo_rerun_if_changed("wrapper.h");

    // Initialize the git submodule if necessary:
    if !Path::new("vendor/cadical/src").exists() {
        let _ = Command::new("git")
            .args(["submodule", "update", "--init", "vendor/cadical"])
            .status();
    }

    // Build configuration:
    let mut cfg = cc::Build::new();
    cfg.cpp(true);
    cfg.std("c++11");
    cfg.flag_if_supported("-Wno-nonnull-compare");
    cfg.define("NBUILD", None);
    cfg.define("NUNLOCKED", None);

    // Handle Debug/Release profile:
    let profile = std::env::var("PROFILE").unwrap();
    match profile.as_str() {
        "debug" => {
            // build_script::cargo_warning("Using Debug profile");
            cfg.debug(true);
        }
        "release" => {
            // build_script::cargo_warning("Using Release profile");
            cfg.debug(false);
            cfg.opt_level(3);
            cfg.define("NDEBUG", None);

            // cfg.debug(true);
            // cfg.opt_level(3);
        }
        _ => {
            build_script::cargo_warning(format!("Unsupported profile '{}'", profile));
        }
    }

    // On Windows, `psapi` is needed for `GetProcessMemoryInfo`
    if cfg!(windows) {
        build_script::cargo_rustc_link_lib("psapi");
    }

    // Find all source files:
    let files = glob::glob("vendor/cadical/src/*.cpp")
        .expect("Bad glob")
        .map(|p| p.expect("Could not read file in glob"))
        .filter(|p| {
            let name = p.file_name().expect("Could not get file name");
            name != "cadical.cpp" && name != "mobical.cpp"
        })
        .collect::<Vec<_>>();

    // Rerun Cargo on changes in source files:
    for path in files.iter() {
        build_script::cargo_rerun_if_changed(path);
    }

    // Specify source files to be compiled:
    cfg.files(files);

    // Compile:
    cfg.compile("cadical");
}
