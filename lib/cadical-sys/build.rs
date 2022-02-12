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
    let files = [
        "analyze.cpp",
        "arena.cpp",
        "assume.cpp",
        "averages.cpp",
        "backtrack.cpp",
        "backward.cpp",
        "bins.cpp",
        "block.cpp",
        "ccadical.cpp",
        "checker.cpp",
        "clause.cpp",
        "collect.cpp",
        "compact.cpp",
        "condition.cpp",
        "config.cpp",
        "constrain.cpp",
        "contract.cpp",
        "cover.cpp",
        "decide.cpp",
        "decompose.cpp",
        "deduplicate.cpp",
        "elim.cpp",
        "ema.cpp",
        "extend.cpp",
        "external.cpp",
        "file.cpp",
        "flags.cpp",
        "format.cpp",
        "gates.cpp",
        "instantiate.cpp",
        "internal.cpp",
        "ipasir.cpp",
        "limit.cpp",
        "logging.cpp",
        "lookahead.cpp",
        "lucky.cpp",
        "message.cpp",
        "minimize.cpp",
        "occs.cpp",
        "options.cpp",
        "parse.cpp",
        "phases.cpp",
        "probe.cpp",
        "profile.cpp",
        "proof.cpp",
        "propagate.cpp",
        "queue.cpp",
        "random.cpp",
        "reap.cpp",
        "reduce.cpp",
        "rephase.cpp",
        "report.cpp",
        "resources.cpp",
        "restart.cpp",
        "restore.cpp",
        "score.cpp",
        "shrink.cpp",
        "signal.cpp",
        "solution.cpp",
        "solver.cpp",
        "stats.cpp",
        "subsume.cpp",
        "terminal.cpp",
        "ternary.cpp",
        "tracer.cpp",
        "transred.cpp",
        "util.cpp",
        "var.cpp",
        "version.cpp",
        "vivify.cpp",
        "walk.cpp",
        "watch.cpp",
    ];
    let files = files.map(|x| format!("vendor/cadical/src/{}", x));
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
    //   bindgen vendor/cadical/src/ccadical.h -o _bindings-ccadical-static.rs
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("vendor/cadical/src/ccadical.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-ccadical-static.rs"))
        .expect("Could not write bindings!");
}

fn generate_bindings_dynamic() {
    println!("cargo:warning=Generating Cadical dynamic bindings...");
    // Note: to generate these bindings manually, use the following command:
    //   bindgen vendor/cadical/src/ccadical.h -o _bindings-ccadical-dynamic.rs --dynamic-loading ccadical
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::builder()
        .header("vendor/cadical/src/ccadical.h")
        .dynamic_library_name("ccadical")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .layout_tests(false)
        .generate()
        .expect("Could not create bindings!")
        .write_to_file(out_path.join("bindings-ccadical-dynamic.rs"))
        .expect("Could not write bindings!");
}
