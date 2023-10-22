use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use fragile::Sticky;
use itertools::{join, Itertools};
use log::{debug, info, warn};
use serde::Serialize;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use thread_local::ThreadLocal;

use partition::interval::{get_bounds, solve_interval, solve_interval_reified};
use partition::parsers::{parse_input_variables, parse_integer_maybe_power, parse_intervals};
use partition::utils::{extract_intervals, is_power_of_two, mean, median, median_absolute_deviation, std_deviation};
use sat_nexus_core::cnf::Cnf;
use sat_nexus_core::solver::SolveResponse;
use sat_nexus_core::utils::bootstrap_solver_from_cnf;
use sat_nexus_wrappers::cadical::CadicalSolver;
use sat_nexus_wrappers::kissat::KissatSolver;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    #[arg(value_name = "CNF")]
    path_cnf: PathBuf,

    /// Input variables (comma-separated).
    #[arg(long = "vars", value_name = "INT...")]
    input_variables: String,

    /// Interval size.
    #[arg(long = "size", value_name = "INT")]
    interval_size: String,

    /// Interval indices (0-based) (comma-separated).
    #[arg(long = "index", value_name = "INT...")]
    intervals: String,

    /// Pool size.
    #[arg(short, long, default_value_t = 4)]
    pool_size: usize,

    /// Allow non-power-of-2 intervals.
    #[arg(long, action)]
    allow_arbitrary_intervals: bool,

    /// Use reified constraints.
    #[arg(long, action)]
    reified: bool,

    /// Results.
    #[arg(long = "results")]
    path_results: Option<PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)?;

    let time_start = Instant::now();
    let args = Cli::parse();
    info!("args = {:?}", args);

    let input_variables = parse_input_variables(&args.input_variables);
    let interval_size = parse_integer_maybe_power(&args.interval_size);
    let interval_indices = parse_intervals(&args.intervals);

    info!(
        "Total {} input variables: {}",
        input_variables.len(),
        join(extract_intervals(&input_variables), ",")
    );
    info!("Interval size: {}", interval_size);
    if !is_power_of_two(interval_size) {
        warn!("Interval size {} is NOT a power of 2.", args.interval_size);
        if !args.allow_arbitrary_intervals {
            panic!("Interval size {} is NOT a power of 2.", args.interval_size);
        }
    }
    if interval_indices.len() == 1 {
        info!("Interval index: {}", interval_indices[0]);
    } else {
        info!("Interval indices: {}", join(extract_intervals(&interval_indices), ","));
    }

    for &interval_index in interval_indices.iter() {
        let (low, high) = get_bounds(interval_index, interval_size);
        if high >= (1 << input_variables.len()) {
            panic!(
                "Interval #{} [{}, {}] is out of bounds (2^N = {})",
                interval_index,
                low,
                high,
                1 << input_variables.len()
            );
        }
    }

    let mut results = Vec::new();

    if args.reified {
        let pool_size = args.pool_size.min(interval_indices.len());
        if pool_size > 1 {
            info!("Building a thread pool with {} workers...", pool_size);
            let pool = rayon::ThreadPoolBuilder::new().num_threads(pool_size).build().unwrap();

            let tls = Arc::new(ThreadLocal::new());
            let cnf = Arc::new(Cnf::from_file(&args.path_cnf));

            {
                let tls = Arc::clone(&tls);
                let cnf = Arc::clone(&cnf);
                pool.broadcast(|_ctx| {
                    let _solver = tls.get_or(|| {
                        info!("Spawning a new solver on thread {:?}", std::thread::current().id());
                        let mut solver = CadicalSolver::new();
                        bootstrap_solver_from_cnf(&mut solver, &cnf);
                        Sticky::new(RefCell::new(solver))
                    });
                });
            }

            info!("Spawning {} jobs...", interval_indices.len());
            let (tx, rx) = std::sync::mpsc::channel();
            for &interval_index in interval_indices.iter() {
                let tx = tx.clone();
                let tls = Arc::clone(&tls);
                let input_variables = input_variables.clone();
                pool.spawn(move || {
                    debug!("Executing on thread {:?}", std::thread::current().id());
                    let time_start = Instant::now();
                    let solver = tls.get().unwrap();
                    fragile::stack_token!(tok);
                    let solver = &mut *solver.get(tok).borrow_mut();
                    let result = solve_interval_reified(solver, &input_variables, interval_size, interval_index);
                    let time = time_start.elapsed();
                    let (low, high) = get_bounds(interval_index, interval_size);
                    info!(
                        "Solved interval #{} [{}, {}] of size {} in {:.3}s",
                        interval_index,
                        low,
                        high,
                        interval_size,
                        time.as_secs_f64()
                    );
                    assert_eq!(result, SolveResponse::Unsat);
                    tx.send((interval_index, result, time)).unwrap();
                });
            }
            drop(tx);

            info!("Awaiting results...");
            for result in rx.into_iter() {
                results.push(result);
            }
        } else {
            // Note: actually, intervals.len() == 1 here.
            for &interval_index in interval_indices.iter() {
                let time_start = Instant::now();
                let mut solver = CadicalSolver::new();
                let cnf = Cnf::from_file(&args.path_cnf);
                bootstrap_solver_from_cnf(&mut solver, &cnf);
                let result = solve_interval_reified(&mut solver, &input_variables, interval_size, interval_index);
                let time = time_start.elapsed();
                let (low, high) = get_bounds(interval_index, interval_size);
                info!(
                    "Solved interval #{} [{}, {}] of size {} in {:.3}s",
                    interval_index,
                    low,
                    high,
                    interval_size,
                    time.as_secs_f64()
                );
                assert_eq!(result, SolveResponse::Unsat);
                results.push((interval_index, result, time))
            }
        }
    } else {
        // NOT args.reified
        let pool_size = args.pool_size.min(interval_indices.len());
        if pool_size > 1 {
            info!("Building a thread pool with {} workers...", pool_size);
            let pool = rayon::ThreadPoolBuilder::new().num_threads(pool_size).build().unwrap();

            info!("Spawning {} jobs...", interval_indices.len());
            let (tx, rx) = std::sync::mpsc::channel();
            for &interval_index in interval_indices.iter() {
                let tx = tx.clone();
                let cnf_path = args.path_cnf.clone();
                let input_variables = input_variables.clone();
                pool.spawn(move || {
                    debug!("Executing on thread {:?}", std::thread::current().id());
                    let time_start = Instant::now();
                    let mut solver = KissatSolver::new();
                    let cnf = Cnf::from_file(cnf_path);
                    bootstrap_solver_from_cnf(&mut solver, &cnf);
                    let result = solve_interval(&mut solver, &input_variables, interval_size, interval_index);
                    let time = time_start.elapsed();
                    let (low, high) = get_bounds(interval_index, interval_size);
                    info!(
                        "Solved interval #{} [{}, {}] of size {} in {:.3}s",
                        interval_index,
                        low,
                        high,
                        interval_size,
                        time.as_secs_f64()
                    );
                    assert_eq!(result, SolveResponse::Unsat);
                    tx.send((interval_index, result, time)).unwrap();
                });
            }
            drop(tx);

            info!("Awaiting results...");
            for result in rx.into_iter() {
                results.push(result);
            }
        } else {
            // Note: actually, intervals.len() == 1 here.
            for &interval_index in interval_indices.iter() {
                let time_start = Instant::now();
                let mut solver = KissatSolver::new();
                let cnf = Cnf::from_file(&args.path_cnf);
                bootstrap_solver_from_cnf(&mut solver, &cnf);
                let result = solve_interval(&mut solver, &input_variables, interval_size, interval_index);
                let time = time_start.elapsed();
                let (low, high) = get_bounds(interval_index, interval_size);
                info!(
                    "Solved interval #{} [{}, {}] of size {} in {:.3}s",
                    interval_index,
                    low,
                    high,
                    interval_size,
                    time.as_secs_f64()
                );
                assert_eq!(result, SolveResponse::Unsat);
                results.push((interval_index, result, time))
            }
        }
    }

    results.sort_by_key(|(index, _, _)| *index);
    info!(
        "Done computing {} results after {:.3}s",
        results.len(),
        time_start.elapsed().as_secs_f64()
    );
    // for (interval_index, result, time) in results.iter() {
    //     info!("{}: {} in {:.3}s", interval_index, result, time.as_secs_f64());
    // }

    if let Some(path_results) = &args.path_results {
        #[derive(Serialize)]
        struct Row {
            index: usize,
            size: usize,
            low: usize,
            high: usize,
            time: f64,
        }

        let mut wrt = csv::Writer::from_path(path_results)?;
        for &(interval_index, _result, time) in results.iter() {
            let (low, high) = get_bounds(interval_index, interval_size);
            wrt.serialize(Row {
                index: interval_index,
                size: interval_size,
                low,
                high,
                time: time.as_secs_f64(),
            })?;
        }
    }

    let times = results.iter().map(|(_, _, time)| time.as_secs_f64()).collect_vec();
    let time_mean = mean(&times);
    let time_sd = std_deviation(&times);
    info!("Time mean±sd: {:.3} ± {:.3}", time_mean, time_sd);
    let time_med = median(&times);
    let time_mad = median_absolute_deviation(&times);
    info!("Time med±mad: {:.3} ± {:.3}", time_med, time_mad);

    let total_subtasks = ((1u128 << input_variables.len()) as f64 / interval_size as f64).ceil() as usize;
    info!("Total subtasks: {}", total_subtasks);

    let total_time_estimation = time_mean * total_subtasks as f64;
    info!("Total time estimation: {:.1}", total_time_estimation);

    info!("All done in {:.3}s", time_start.elapsed().as_secs_f64());
    Ok(())
}
