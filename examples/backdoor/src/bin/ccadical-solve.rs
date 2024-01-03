use std::time::Instant;

use cadical_sys::statik::*;
use ffi_utils::cstr2str;
use simple_sat::utils::parse_dimacs;

// Run:
// cargo run --release -p backdoor --bin ccadical-solve -- data/my.cnf

fn main() {
    let start_time = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];

    unsafe {
        let ptr = ccadical_init();
        println!("ptr = {:?}", ptr);
        println!("signature = {}", cstr2str(ccadical_signature()));

        println!("Adding clauses from '{}'...", path);
        let mut num_clauses = 0;
        for clause in parse_dimacs(path) {
            num_clauses += 1;
            for lit in clause {
                ccadical_add(ptr, lit.to_external());
            }
            ccadical_add(ptr, 0);
        }
        println!("num_clauses = {}", num_clauses);
        println!("ccadical_vars() = {}", ccadical_vars(ptr));

        println!("Solving...");
        let res = ccadical_solve(ptr);
        println!("res = {}", res);
    }

    let total_time = start_time.elapsed();
    println!("\nAll done in {:.3} s", total_time.as_secs_f64());
}
