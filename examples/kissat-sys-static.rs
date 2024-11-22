use ffi_utils::cstr2str;
use kissat_sys::statik::*;

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
        let ptr = kissat_init();
        println!("ptr = {:?}", ptr);
        println!("signature = {}", cstr2str(kissat_signature()));
        println!("id = {}", cstr2str(kissat_id()));
        println!("version = {}", cstr2str(kissat_version()));
        println!("compiler = {}", cstr2str(kissat_compiler()));

        kissat_add(ptr, 1);
        kissat_add(ptr, 2);
        kissat_add(ptr, 0);

        kissat_add(ptr, 3);
        kissat_add(ptr, 4);
        kissat_add(ptr, 0);

        kissat_add(ptr, -1);
        kissat_add(ptr, -2);
        kissat_add(ptr, 0);

        kissat_add(ptr, -3);
        kissat_add(ptr, -4);
        kissat_add(ptr, 0);

        kissat_add(ptr, 3);
        kissat_add(ptr, 0);

        // Solving: should be SAT
        let result = kissat_solve(ptr);
        println!("result = {} ({})", result, result2str(result));
        assert_eq!(result, SAT);

        // // Assuming conflicting literals: should be UNSAT
        // kissat_assume(ptr, 1);
        // kissat_assume(ptr, 2);
        // let result = kissat_solve(ptr);
        // println!("result = {}", result);
        // assert_eq!(result, UNSAT);
        //
        // // `solve` automatically resets given assumptions: another call should be SAT
        // let result = kissat_solve(ptr);
        // println!("result = {}", result);
        // assert_eq!(result, SAT);

        println!("Trying literals:");
        for i in 1..=4 {
            println!("value({}) = {}", i, kissat_value(ptr, i));
        }
        println!("Trying negative literals:");
        for i in 1..=4 {
            println!("value({}) = {}", -i, kissat_value(ptr, -i));
        }
    }

    Ok(())
}
