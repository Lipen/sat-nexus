use ffi_utils::cstr2str;
use kissat_sys::dynamic::*;

const SAT: i32 = 10;
const UNSAT: i32 = 20;

fn result2str(result: i32) -> &'static str {
    match result {
        SAT => "SAT",
        UNSAT => "UNSAT",
        _ => "?",
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    unsafe {
        let ffi = KissatFFI::instance();
        // let ffi = KissatFFI::load("kissat");
        println!("ffi created");
        let ptr = ffi.kissat_init();
        println!("ptr = {:?}", ptr);
        println!("signature = {}", cstr2str(ffi.kissat_signature()));
        println!("id = {}", cstr2str(ffi.kissat_id()));
        println!("version = {}", cstr2str(ffi.kissat_version()));
        println!("compiler = {}", cstr2str(ffi.kissat_compiler()));

        ffi.kissat_add(ptr, 1);
        ffi.kissat_add(ptr, 2);
        ffi.kissat_add(ptr, 0);

        ffi.kissat_add(ptr, 3);
        ffi.kissat_add(ptr, 4);
        ffi.kissat_add(ptr, 0);

        ffi.kissat_add(ptr, -1);
        ffi.kissat_add(ptr, -2);
        ffi.kissat_add(ptr, 0);

        ffi.kissat_add(ptr, -3);
        ffi.kissat_add(ptr, -4);
        ffi.kissat_add(ptr, 0);

        // Solving: should be SAT
        let result = ffi.kissat_solve(ptr);
        println!("result = {} ({})", result, result2str(result));
        assert_eq!(result, SAT);

        println!("Trying literals:");
        for i in 1..=4 {
            println!("value({}) = {}", i, ffi.kissat_value(ptr, i));
        }
        println!("Trying negative literals:");
        for i in 1..=4 {
            println!("value({}) = {}", -i, ffi.kissat_value(ptr, -i));
        }
    }

    Ok(())
}
