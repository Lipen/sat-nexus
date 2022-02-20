use color_eyre::eyre::Result;

use cadical_sys::dynamic::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    unsafe {
        let lib = CCadicalFFI::load("cadical");
        let ptr = lib.ccadical_init();

        let signature = lib.signature();
        println!("signature = {}", signature);

        lib.ccadical_add(ptr, 1);
        lib.ccadical_add(ptr, 2);
        lib.ccadical_add(ptr, 0);

        lib.ccadical_add(ptr, 3);
        lib.ccadical_add(ptr, 4);
        lib.ccadical_add(ptr, 0);

        lib.ccadical_add(ptr, -1);
        lib.ccadical_add(ptr, -2);
        lib.ccadical_add(ptr, 0);

        lib.ccadical_add(ptr, -3);
        lib.ccadical_add(ptr, -4);
        lib.ccadical_add(ptr, 0);

        const SAT: i32 = 10;
        const UNSAT: i32 = 20;

        // Solving: should be SAT
        let result = lib.ccadical_solve(ptr);
        println!("result = {}", result);
        assert_eq!(result, SAT);

        // Assuming conflicting literals: should be UNSAT
        lib.ccadical_assume(ptr, 1);
        lib.ccadical_assume(ptr, 2);
        let result = lib.ccadical_solve(ptr);
        println!("result = {}", result);
        assert_eq!(result, UNSAT);

        // `solve` automatically resets given assumptions: another call should be SAT
        let result = lib.ccadical_solve(ptr);
        println!("result = {}", result);
        assert_eq!(result, SAT);

        println!("Trying literals:");
        for i in 1..=4 {
            println!("val({}) = {}", i, lib.ccadical_val(ptr, i));
        }
        println!("Trying negative literals:");
        for i in 1..=4 {
            println!("val({}) = {}", -i, lib.ccadical_val(ptr, -i));
        }
    }
    Ok(())
}
