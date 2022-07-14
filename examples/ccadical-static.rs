use cadical_sys::statik::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    unsafe {
        let ptr = ccadical_init();

        // let signature = lib.signature();
        // println!("signature = {}", signature);

        ccadical_add(ptr, 1);
        ccadical_add(ptr, 2);
        ccadical_add(ptr, 0);

        ccadical_add(ptr, 3);
        ccadical_add(ptr, 4);
        ccadical_add(ptr, 0);

        ccadical_add(ptr, -1);
        ccadical_add(ptr, -2);
        ccadical_add(ptr, 0);

        ccadical_add(ptr, -3);
        ccadical_add(ptr, -4);
        ccadical_add(ptr, 0);

        const SAT: i32 = 10;
        const UNSAT: i32 = 20;

        // Solving: should be SAT
        let result = ccadical_solve(ptr);
        println!("result = {}", result);
        assert_eq!(result, SAT);

        // Assuming conflicting literals: should be UNSAT
        ccadical_assume(ptr, 1);
        ccadical_assume(ptr, 2);
        let result = ccadical_solve(ptr);
        println!("result = {}", result);
        assert_eq!(result, UNSAT);

        // `solve` automatically resets given assumptions: another call should be SAT
        let result = ccadical_solve(ptr);
        println!("result = {}", result);
        assert_eq!(result, SAT);

        println!("Trying literals:");
        for i in 1..=4 {
            println!("val({}) = {}", i, ccadical_val(ptr, i));
        }
        println!("Trying negative literals:");
        for i in 1..=4 {
            println!("val({}) = {}", -i, ccadical_val(ptr, -i));
        }
    }
    Ok(())
}
