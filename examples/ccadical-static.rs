use cadical_sys::statik::*;
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
        let ptr = ccadical_init();
        println!("ptr = {:?}", ptr);
        println!("signature = {}", cstr2str(ccadical_signature()));

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

        // Solving: should be SAT
        let result = ccadical_solve(ptr);
        println!("result = {} ({})", result, result2str(result));
        assert_eq!(result, SAT);

        // Assuming conflicting literals: should be UNSAT
        ccadical_assume(ptr, 1);
        ccadical_assume(ptr, 2);
        let result = ccadical_solve(ptr);
        println!("result = {} ({})", result, result2str(result));
        assert_eq!(result, UNSAT);

        println!("Checking failed literals:");
        for i in 1..=4 {
            println!("failed({}) = {}", i, ccadical_failed(ptr, i));
            println!("failed(-{}) = {}", i, ccadical_failed(ptr, -i));
        }
        assert_eq!(1, ccadical_failed(ptr, 1));
        assert_eq!(1, ccadical_failed(ptr, 2));

        // `solve` automatically resets given assumptions: another call should be SAT
        let result = ccadical_solve(ptr);
        println!("result = {} ({})", result, result2str(result));
        assert_eq!(result, SAT);

        println!("Trying literals:");
        for i in 1..=4 {
            println!("val({}) = {}", i, ccadical_val(ptr, i));
        }
        println!("Trying negative literals:");
        for i in 1..=4 {
            println!("val({}) = {}", -i, ccadical_val(ptr, -i));
        }

        ccadical_release(ptr);
    }

    println!("----------");

    unsafe {
        let ptr = ccadical_init();
        println!("ptr = {:?}", ptr);
        println!("signature = {}", cstr2str(ccadical_signature()));

        ccadical_add(ptr, 3);
        ccadical_add(ptr, -1);
        ccadical_add(ptr, -2);
        ccadical_add(ptr, 0);

        ccadical_add(ptr, -3);
        ccadical_add(ptr, 1);
        ccadical_add(ptr, 0);

        ccadical_add(ptr, -3);
        ccadical_add(ptr, 2);
        ccadical_add(ptr, 0);

        ccadical_add(ptr, -1);
        ccadical_add(ptr, 2);
        ccadical_add(ptr, 0);

        ccadical_propcheck_tree_begin(ptr);
        ccadical_propcheck_tree_add(ptr, 1);
        ccadical_propcheck_tree_add(ptr, 2);
        let count = ccadical_propcheck_tree(ptr, 0);
        println!("count = {}", count);

        ccadical_release(ptr);
    }

    Ok(())
}
