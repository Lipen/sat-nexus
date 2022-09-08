use std::fs::File;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::AppSettings;
use clap::Parser;
use num_format::{Locale, ToFormattedString};
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DurationSecondsWithFrac;

use simple_sat::options::Options;
use simple_sat::options::DEFAULT_OPTIONS;
use simple_sat::solver::{SolveResult, Solver};

const HEADING_RESTART: &'static str = "RESTART OPTIONS";
const HEADING_REDUCE_DB: &'static str = "REDUCE-DB OPTIONS";

#[derive(Parser)]
#[clap(author, version)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
struct Cli {
    /// Path to input CNF.
    #[clap(value_name = "PATH")]
    input: PathBuf,

    /// Path to output results.
    #[clap(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Use luby restarts.
    #[clap(help_heading = HEADING_RESTART)]
    #[clap(long, value_name = "BOOL")]
    #[clap(action = clap::ArgAction::Set)]
    // #[clap(default_missing_value = "true")]
    #[clap(default_value_t = DEFAULT_OPTIONS.is_luby)]
    luby: bool,

    /// Base number of conflicts between restarts.
    #[clap(help_heading = HEADING_RESTART)]
    #[clap(long, value_name = "NUM")]
    #[clap(default_value_t = DEFAULT_OPTIONS.restart_init)]
    restart_init: usize,

    /// Increment value for the number of conflicts between restarts.
    #[clap(help_heading = HEADING_RESTART)]
    #[clap(long, value_name = "NUM")]
    #[clap(default_value_t = DEFAULT_OPTIONS.restart_inc)]
    restart_inc: f64,

    #[clap(help_heading = HEADING_REDUCE_DB)]
    #[clap(long, value_name = "NUM")]
    #[clap(default_value_t = DEFAULT_OPTIONS.min_learnts_limit)]
    min_learnts_limit: usize,

    #[clap(help_heading = HEADING_REDUCE_DB)]
    #[clap(long, value_name = "NUM")]
    #[clap(default_value_t = DEFAULT_OPTIONS.learntsize_factor)]
    learntsize_factor: f64,

    #[clap(help_heading = HEADING_REDUCE_DB)]
    #[clap(long, value_name = "NUM")]
    #[clap(default_value_t = DEFAULT_OPTIONS.learntsize_inc)]
    learntsize_inc: f64,

    #[clap(help_heading = HEADING_REDUCE_DB)]
    #[clap(long, value_name = "NUM")]
    #[clap(default_value_t = DEFAULT_OPTIONS.learntsize_adjust_start)]
    learntsize_adjust_start: f64,

    #[clap(help_heading = HEADING_REDUCE_DB)]
    #[clap(long, value_name = "NUM")]
    #[clap(default_value_t = DEFAULT_OPTIONS.learntsize_adjust_inc)]
    learntsize_adjust_inc: f64,
}

#[serde_as]
#[derive(Debug, Serialize)]
struct TheResult {
    name: String,
    result: SolveResult,
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    time_total: Duration,
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    time_search: Duration,
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    time_propagate: Duration,
    num_vars: usize,
    num_clauses: usize,
    num_learnts: usize,
    num_decisions: usize,
    num_propagations: usize,
    num_conflicts: usize,
    num_restarts: usize,
    num_reduces: usize,
}

fn main() -> color_eyre::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    // Setup the solver:
    let time_start = Instant::now();
    let options = Options {
        is_luby: cli.luby,
        restart_init: cli.restart_init,
        restart_inc: cli.restart_inc,
        min_learnts_limit: cli.min_learnts_limit,
        learntsize_factor: cli.learntsize_factor,
        learntsize_inc: cli.learntsize_inc,
        learntsize_adjust_start: cli.learntsize_adjust_start,
        learntsize_adjust_inc: cli.learntsize_adjust_inc,
        // ..DEFAULT_OPTIONS
    };
    let mut solver = Solver::new(options);
    solver.init_from_file(&cli.input);
    let time_create = time_start.elapsed();

    // Solve:
    let result = solver.solve();
    let time_total = time_start.elapsed();

    let result = TheResult {
        name: cli.input.file_name().unwrap().to_str().unwrap().to_string(),
        result,
        time_total,
        time_search: solver.time_search,
        time_propagate: solver.time_propagate,
        num_vars: solver.num_vars(),
        num_clauses: solver.num_clauses(),
        num_learnts: solver.num_learnts(),
        num_decisions: solver.num_decisions(),
        num_propagations: solver.num_propagations(),
        num_conflicts: solver.num_conflicts(),
        num_restarts: solver.num_restarts(),
        num_reduces: solver.num_reduces(),
    };
    // println!("{:#?}", result);

    // Dump the result:
    if let Some(output) = cli.output {
        println!("Writing result to '{}'...", output.display());
        serde_json::to_writer_pretty(File::create(output)?, &result)?;
    }

    // Print the result and timings:
    let format = &Locale::en;
    println!("Solver returned: {:?}", result);
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
    Ok(())
}
