use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use num_format::{Locale, ToFormattedString};

use simple_sat::options::Options;
use simple_sat::options::DEFAULT_OPTIONS;
use simple_sat::solver::Solver;

#[derive(Parser)]
#[clap(author, version)]
struct Cli {
    /// Path to input CNF.
    #[clap(value_name = "PATH")]
    input: PathBuf,

    /// Use luby restarts.
    #[clap(long, action = clap::ArgAction::Set)]
    #[clap(default_missing_value = "true")]
    #[clap(default_value_t = DEFAULT_OPTIONS.is_luby)]
    luby: bool,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    // Setup the solver:
    let time_start = Instant::now();
    let options = Options {
        is_luby: cli.luby,
        ..DEFAULT_OPTIONS
    };
    let mut solver = Solver::new(options);
    solver.init_from_file(&cli.input);
    let time_create = time_start.elapsed();

    // Solve:
    let res = solver.solve();
    let time_total = time_start.elapsed();

    // Print the result and timings:
    let format = &Locale::en;
    println!("Solver returned: {:?}", res);
    println!("vars:         {}", solver.num_vars().to_formatted_string(format));
    println!("clauses:      {}", solver.num_clauses().to_formatted_string(format));
    println!("learnts:      {}", solver.num_learnts().to_formatted_string(format));
    println!("decisions:    {}", solver.num_decisions().to_formatted_string(format));
    println!("propagations: {}", solver.num_propagations().to_formatted_string(format));
    println!("conflicts:    {}", solver.num_conflicts().to_formatted_string(format));
    println!("restarts:     {}", solver.num_restarts().to_formatted_string(format));
    println!("reduces:      {}", solver.num_reduces().to_formatted_string(format));
    println!("time total:      {:?}", time_total);
    println!(
        "time create:     {:?} ({:.2}%)",
        time_create,
        100.0 * time_create.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time search:     {:?} ({:.2}%)",
        solver.time_search,
        100.0 * solver.time_search.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time propagate:  {:?} ({:.2}%)",
        solver.time_propagate,
        100.0 * solver.time_propagate.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time analyze:    {:?} ({:.2}%)",
        solver.time_analyze,
        100.0 * solver.time_analyze.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time backtrack:  {:?} ({:.2}%)",
        solver.time_backtrack,
        100.0 * solver.time_backtrack.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time decide:     {:?} ({:.2}%)",
        solver.time_decide,
        100.0 * solver.time_decide.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time restart:    {:?} ({:.2}%)",
        solver.time_restart,
        100.0 * solver.time_restart.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time reduce:     {:?} ({:.2}%)",
        solver.time_reduce,
        100.0 * solver.time_reduce.as_secs_f64() / time_total.as_secs_f64(),
    );

    println!("All done in {:?}", time_start.elapsed());
}
