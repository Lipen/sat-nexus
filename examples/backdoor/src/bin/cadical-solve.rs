use std::time::Instant;

use backdoor::utils::clause_to_external;

use cadical::statik::Cadical;
use simple_sat::utils::parse_dimacs;

// Run:
// cargo run --release -p backdoor --bin cadical-solve -- data/my.cnf

fn main() {
    let start_time = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];

    let solver = Cadical::new();
    println!("signature = {}", solver.signature());

    println!("Adding clauses from '{}'...", path);
    let mut num_clauses = 0;
    for clause in parse_dimacs(path) {
        num_clauses += 1;
        solver.add_clause(clause_to_external(&clause));
    }
    println!("num_clauses = {}", num_clauses);
    println!("solver.vars() = {}", solver.vars());

    println!("Solving...");
    let res = solver.solve();
    println!("res = {:?}", res);

    let total_time = start_time.elapsed();
    println!("\nAll done in {:.3} s", total_time.as_secs_f64());
}
