use cadical_sys::dynamic::*;
use ffi_utils::cstr2str;

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
        let ffi = CCadicalFFI::instance();
        // let ffi = CCadicalFFI::load("cadical");
        println!("ffi created");
        let ptr = ffi.ccadical_init();
        println!("ptr = {:?}", ptr);
        println!("signature = {}", cstr2str(ffi.ccadical_signature()));

        ffi.ccadical_add(ptr, 1);
        ffi.ccadical_add(ptr, 2);
        ffi.ccadical_add(ptr, 0);

        ffi.ccadical_add(ptr, 3);
        ffi.ccadical_add(ptr, 4);
        ffi.ccadical_add(ptr, 0);

        ffi.ccadical_add(ptr, -1);
        ffi.ccadical_add(ptr, -2);
        ffi.ccadical_add(ptr, 0);

        ffi.ccadical_add(ptr, -3);
        ffi.ccadical_add(ptr, -4);
        ffi.ccadical_add(ptr, 0);

        // Solving: should be SAT
        let result = ffi.ccadical_solve(ptr);
        println!("result = {} ({})", result, result2str(result));
        assert_eq!(result, SAT);

        // Assuming conflicting literals: should be UNSAT
        ffi.ccadical_assume(ptr, 1);
        ffi.ccadical_assume(ptr, 2);
        let result = ffi.ccadical_solve(ptr);
        println!("result = {} ({})", result, result2str(result));
        assert_eq!(result, UNSAT);

        // `solve` automatically resets given assumptions: another call should be SAT
        let result = ffi.ccadical_solve(ptr);
        println!("result = {} ({})", result, result2str(result));
        assert_eq!(result, SAT);

        println!("Trying literals:");
        for i in 1..=4 {
            println!("val({}) = {}", i, ffi.ccadical_val(ptr, i));
        }
        println!("Trying negative literals:");
        for i in 1..=4 {
            println!("val({}) = {}", -i, ffi.ccadical_val(ptr, -i));
        }
    }

    Ok(())
}
