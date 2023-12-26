use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use itertools::Itertools;
use log::{debug, info};
use rand::prelude::*;

use backdoor::utils::parse_comma_separated_intervals;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::utils::DisplaySlice;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// Input file with CNF in DIMACS format.
    #[arg(value_name = "CNF")]
    path_cnf: PathBuf,

    /// Input variables.
    #[arg(long = "inputs", value_name = "INT...")]
    input_variables: String,

    /// Number of random samples.
    #[arg(long = "samples", value_name = "INT")]
    num_samples: usize,

    /// Random seed.
    #[arg(long, value_name = "INT", default_value_t = 42)]
    seed: u64,

    /// Path to a file with results.
    #[arg(long = "results", value_name = "FILE")]
    path_results: Option<PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,simple_sat::solver=info")).init();

    let start_time = Instant::now();
    let args = Cli::parse();
    debug!("args = {:?}", args);

    info!("Seed: {}", args.seed);
    let mut rng = StdRng::seed_from_u64(args.seed);

    let input_variables = parse_comma_separated_intervals(&args.input_variables);
    let n = input_variables.len();
    info!("Total {} input variables: {:?}", n, input_variables);
    assert!(n <= 32);

    // Initialize the SAT solver:
    let mut solver = Solver::default();
    solver.init_from_file(&args.path_cnf);

    debug!("solver.num_vars() = {}", solver.num_vars());
    debug!("solver.num_clauses() = {}", solver.num_clauses());

    let mut num_ok: usize = 0;
    let mut num_fail: usize = 0;

    for _ in 0..args.num_samples {
        let mut cube = Vec::<bool>::with_capacity(n);
        for _ in 0..n {
            cube.push(rng.gen());
        }
        let assumptions = input_variables
            .iter()
            .zip(cube.iter())
            .map(|(&i, &b)| Lit::from_external(if b { i as i32 } else { -(i as i32) }))
            .collect_vec();
        // info!("Trying assumptions = {}", DisplaySlice(&assumptions));
        // let result = solver.propcheck(&assumptions);
        let result = solver.solve_under_assumptions(&assumptions) == simple_sat::solver::SolveResult::Sat;
        if result {
            num_ok += 1;
        } else {
            info!("UNSAT on assumptions = {}", DisplaySlice(&assumptions));
            num_fail += 1;
        }
    }

    let p = num_ok as f64 / args.num_samples as f64;
    info!("OK: {}, FAIL: {}, p = {}", num_ok, num_fail, p);

    let elapsed = Instant::now() - start_time;
    println!("\nAll done in {:.3} s", elapsed.as_secs_f64());
    Ok(())
}
