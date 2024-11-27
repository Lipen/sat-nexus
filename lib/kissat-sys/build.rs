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
    build_script::cargo_warning("Building Kissat dynamic bindings...");
    build_script::cargo_rerun_if_changed("wrapper.h");

    // Note: to generate these bindings manually, use the following command:
    //   bindgen wrapper.h -o _bindings-kissat-dynamic.rs --dynamic-loading kissat_ffi --dynamic-link-require-all --no-layout-tests --allowlist-function "kissat_.*" --blocklist-item "kissat_changes" --blocklist-item "kissat_changed" --blocklist-item "kissat_options_print_value" --clang-arg "-DCOMPACT" --clang-arg "-DNDEBUG" --clang-arg "-DNPROOFS" --clang-arg "-DQUIET"
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .dynamic_library_name("kissat_ffi")
        .dynamic_link_require_all(true)
        .allowlist_function("kissat_.*")
        .blocklist_item("kissat_changes")
        .blocklist_item("kissat_changed")
        .blocklist_item("kissat_options_print_value")
        .clang_arg("-DCOMPACT")
        .clang_arg("-DNDEBUG")
        .clang_arg("-DNPROOFS")
        .clang_arg("-DQUIET")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Could not create bindings!");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings-kissat-dynamic.rs"))
        .expect("Could not write bindings!");
}

#[cfg(feature = "static")]
fn generate_bindings_static() {
    build_script::cargo_warning("Building Kissat static bindings...");
    build_script::cargo_rerun_if_changed("wrapper.h");

    // Note: to generate these bindings manually, use the following command:
    //   bindgen wrapper.h -o _bindings-kissat-static.rs --allowlist-function "kissat_.*"
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .allowlist_function("kissat_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Could not create bindings!");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings-kissat-static.rs"))
        .expect("Could not write bindings!");
}

#[cfg(feature = "static")]
fn build_static_lib() {
    build_script::cargo_warning("Building Kissat static library...");

    let mut cfg = cc::Build::new();
    cfg.opt_level(3)
        .warnings(false)
        .define("COMPACT", None)
        .define("NDEBUG", None)
        .define("NPROOFS", None)
        .define("QUIET", None);

    // Fix bitfields on Windows:
    cfg.flag_if_supported("-mno-ms-bitfields");

    // Generate 'build.h':
    let version = std::fs::read_to_string("vendor/kissat/VERSION")
        .expect("missing kissat submodule")
        .trim()
        .to_string();
    let compiler = cfg
        .get_compiler()
        .to_command()
        .get_args()
        .map(|x| x.to_string_lossy().into_owned())
        .collect::<Vec<String>>()
        .join(" ");
    let mut build_header = String::new();
    use std::fmt::Write;
    writeln!(build_header, "#define VERSION {:?}", version).unwrap();
    writeln!(build_header, "#define COMPILER {:?}", compiler).unwrap();
    writeln!(build_header, "#define ID {:?}", "unknown").unwrap();
    writeln!(build_header, "#define BUILD {:?}", "unknown").unwrap();
    writeln!(build_header, "#define DIR {:?}", "unknown").unwrap();
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let build_header_file = out_dir.join("build.h");
    std::fs::write(build_header_file.as_path(), build_header).expect("Could not write 'build.h'");
    if cfg.is_flag_supported("-include").unwrap_or(false) {
        cfg.flag("-include");
        cfg.flag(build_header_file.display().to_string().as_str());
    } else {
        cfg.include(out_dir);
    }

    // Filter source files:
    let excluded = ["application.c", "handle.c", "main.c", "parse.c", "witness.c"];
    let files = glob::glob("vendor/kissat/src/*.c")
        .expect("Bad glob")
        .map(|p| p.expect("Could not read file in glob"))
        .filter(|p| {
            let name = p.file_name().expect("Could not get file name");
            let name = name.to_string_lossy();
            !excluded.contains(&name.as_ref())
        })
        .collect::<Vec<_>>();
    for file in files.iter() {
        build_script::cargo_rerun_if_changed(file);
    }
    cfg.files(files);

    cfg.compile("kissat");
}
