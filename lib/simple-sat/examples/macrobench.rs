use std::cmp::Reverse;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use glob::glob;
use itertools::Itertools;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DurationSecondsWithFrac;
use simple_sat::solver::{SolveResult, Solver};
use tabled::settings::Style;
use tabled::{Table, Tabled};

#[serde_as]
#[derive(Debug, Serialize)]
struct TheResult {
    path: PathBuf,
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

fn round_f64(x: f64, digits: i32) -> f64 {
    let p = f64::powi(10.0, digits);
    (x * p).round() / p
}

fn main() -> color_eyre::Result<()> {
    let time_start = Instant::now();
    let mut benchmarks = Vec::new();

    // Add benchmarks from `data` folder:
    let s = "./data/easy/*.cnf.gz";
    for e in glob(s)? {
        let path = e?;
        benchmarks.push(path);
    }

    // Run all the benchmarks:
    println!("Running {} benchmarks...", benchmarks.len());
    assert!(!benchmarks.is_empty(), "No benchmarks to run!");
    let results = benchmarks
        .into_iter()
        .map(|path| {
            println!("==> Solving '{}'...", path.display());
            let time_total_start = Instant::now();
            let mut solver = Solver::default();
            solver.init_from_file(&path);
            let result = solver.solve();
            let time_total = time_total_start.elapsed();

            let result = TheResult {
                path,
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
            println!("{:#?}", result);

            result
        })
        .collect_vec();

    // Write JSON with results:
    let path_results = Path::new("results.json");
    if path_results.exists() {
        let path_results_old = Path::new("results_old.json");
        println!("Saving old results in {}", path_results_old.display());
        std::fs::rename(path_results, path_results_old)?;
    }
    println!("Writing results in {}...", path_results.display());
    serde_json::to_writer_pretty(File::create(path_results)?, &results)?;

    #[derive(Tabled)]
    struct TableLine {
        name: String,
        result: SolveResult,
        time_total: f64,
        time_propagate: f64,
        num_vars: usize,
        num_clauses: usize,
        num_learnts: usize,
        num_decisions: usize,
        num_propagations: usize,
        num_conflicts: usize,
        num_restarts: usize,
        num_reduces: usize,
    }

    // Show the table with results:
    let data = results.into_iter().sorted_by_key(|r| Reverse(r.time_total)).map(|res| TableLine {
        name: res.path.file_name().unwrap().to_string_lossy().to_string(),
        result: res.result,
        time_total: round_f64(res.time_total.as_secs_f64(), 3),
        time_propagate: round_f64(res.time_propagate.as_secs_f64(), 3),
        num_vars: res.num_vars,
        num_clauses: res.num_clauses,
        num_learnts: res.num_learnts,
        num_decisions: res.num_decisions,
        num_propagations: res.num_propagations,
        num_conflicts: res.num_conflicts,
        num_restarts: res.num_restarts,
        num_reduces: res.num_reduces,
    });
    let mut table = Table::new(data);
    table.with(Style::modern());
    println!("{}", table);

    println!("All done in {:?}", time_start.elapsed());
    Ok(())
}
