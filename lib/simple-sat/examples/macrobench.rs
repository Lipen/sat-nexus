use std::path::PathBuf;
use std::time::{Duration, Instant};

use glob::glob;
use itertools::Itertools;
use tap::Tap;

use simple_sat::solver::Solver;
use simple_sat::utils::measure_time;

#[derive(Debug)]
struct TheResult {
    path: PathBuf,
    res: bool,
    time_total: Duration,
    time_solve: Duration,
    time_search: Duration,
    time_propagate: Duration,
    num_vars: usize,
    num_clauses: usize,
    num_learnts: usize,
    num_conflicts: usize,
    num_decisions: usize,
    num_propagations: usize,
}

fn main() -> color_eyre::Result<()> {
    let mut benchmarks = Vec::new();

    // Add benchmarks from `data` folder:
    for e in glob("./data/easy/*.cnf.gz")? {
        let path = e?;
        benchmarks.push(path);
    }

    // Run all the benchmarks:
    println!("Running {} benchmarks...", benchmarks.len());
    let results = benchmarks
        .into_iter()
        .map(|path| {
            println!("==> Running '{}'", path.display());
            let time_start = Instant::now();

            let mut solver = Solver::from_file(&path);
            let (time_solve, res) = measure_time(|| solver.solve());

            let time_total = time_start.elapsed();
            println!("{} on '{}' in {:?}", if res { "SAT" } else { "UNSAT" }, path.display(), time_total);

            TheResult {
                path,
                res,
                time_total,
                time_solve,
                time_search: solver.time_search,
                time_propagate: solver.time_propagate,
                num_vars: solver.num_vars(),
                num_clauses: solver.num_clauses(),
                num_learnts: solver.num_learnts(),
                num_conflicts: solver.num_conflicts(),
                num_decisions: solver.num_decisions(),
                num_propagations: solver.num_propagations(),
            }
        })
        .collect_vec();

    // Sort the results by `time_total`:
    let results = results.tap_mut(|rs| rs.sort_by_key(|r| r.time_total));

    // Print the results:
    for result in results {
        println!("Result for {}: {:#?}", result.path.display(), result);
    }

    Ok(())
}
