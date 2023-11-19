fn main() {
    #[cfg(feature = "static")]
    {
        generate_bindings_static();
        build_static_lib();
    }
}

#[cfg(feature = "static")]
fn generate_bindings_static() {
    build_script::cargo_warning("Generating Espresso static bindings...");
    build_script::cargo_rerun_if_changed("wrapper.h");

    // Note: to generate these bindings manually, use the following command:
    //   bindgen wrapper.h -o _bindings-espresso-static.rs --blocklist-item "__mingw.*"
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .blocklist_item("__mingw.*")
        .blocklist_item("__MINGW.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Could not create bindings!");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings-espresso-static.rs"))
        .expect("Could not write bindings!");
}

#[cfg(feature = "static")]
fn build_static_lib() {
    build_script::cargo_warning("Building Espresso static library...");

    let files = glob::glob("vendor/espresso/espresso-src/*.c")
        .expect("Bad glob")
        .map(|p| p.expect("Could not read file in glob"))
        .filter(|p| {
            let name = p.file_name().expect("Could not get file name");
            name != "main.c"
        })
        .collect::<Vec<_>>();
    cc::Build::new()
        .files(files)
        .std("c99")
        .flag_if_supported("-Wno-misleading-indentation")
        .flag_if_supported("-Wno-unused-result")
        .flag_if_supported("-Wno-format-overflow")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .compile("espresso");
}
