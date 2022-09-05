use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use num_format::{Locale, ToFormattedString};

use simple_sat::solver::Solver;

#[derive(Parser)]
#[clap(author, version)]
struct Cli {
    /// Path to input CNF.
    #[clap(value_name = "PATH")]
    input: PathBuf,

    /// Use luby restarts.
    #[clap(long, action = clap::ArgAction::Set, default_missing_value = "true", default_value_t = true)]
    luby: bool,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    // Initialize the solver from file:
    let time_start = Instant::now();
    let mut solver = Solver::from_file(&cli.input);
    let time_create = time_start.elapsed();

    // Setup the solver parameters:
    solver.restart_strategy.is_luby = cli.luby;

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
    println!(
        "time insert_var_order: {:?} ({:.2}%) [{} times]",
        solver.var_order.time_insert_var_order,
        100.0 * solver.var_order.time_insert_var_order.as_secs_f64() / time_total.as_secs_f64(),
        solver.var_order.num_insert_var_order,
    );
    println!(
        "time update_var_order: {:?} ({:.2}%) [{} times]",
        solver.var_order.time_update_var_order,
        100.0 * solver.var_order.time_update_var_order.as_secs_f64() / time_total.as_secs_f64(),
        solver.var_order.num_update_var_order,
    );

    println!("All done in {:?}", time_start.elapsed());
}
