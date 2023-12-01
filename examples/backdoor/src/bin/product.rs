use std::collections::HashSet;
use std::ffi::CString;
use std::fs::File;
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::{debug, info, trace};

use backdoor::algorithm::{Algorithm, Options, DEFAULT_OPTIONS};
use backdoor::utils::partition_tasks;
use cadical_sys::statik::*;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::utils::DisplaySlice;

// Run this example:
// cargo run -p backdoor --bin product -- data/mult/lec_CvK_12.cnf --backdoor-size 10 --num-iters 10000 --num-runs 1000

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    #[arg(value_name = "CNF")]
    path_cnf: PathBuf,

    /// Backdoor size.
    #[arg(long, value_name = "INT")]
    backdoor_size: usize,

    /// Number of EA iterations.
    #[arg(long, value_name = "INT")]
    num_iters: usize,

    /// Number of runs.
    #[arg(long, value_name = "INT")]
    num_runs: usize,

    /// Number of conflicts.
    #[arg(long, value_name = "INT", default_value_t = 1000)]
    num_conflicts: usize,

    /// Path to a file with results.
    #[arg(long = "results", value_name = "FILE")]
    path_results: Option<PathBuf>,

    /// Random seed.
    #[arg(long, value_name = "INT", default_value_t = DEFAULT_OPTIONS.seed)]
    seed: u64,

    /// Do ban variables used in best backdoors on previous runs?
    #[arg(long)]
    ban_used: bool,

    /// Comma-separated list of banned variables (1-based indices).
    #[arg(long, value_name = "INT...")]
    bans: Option<String>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,simple_sat::solver=info")).init();

    let start_time = Instant::now();
    let args = Cli::parse();
    info!("args = {:?}", args);

    // Initialize the SAT solver:
    let mut solver = Solver::default();
    solver.init_from_file(&args.path_cnf);

    let solver_full = unsafe { ccadical_init() };
    unsafe {
        for clause in solver.clauses_iter() {
            for lit in clause.lits() {
                ccadical_add(solver_full, lit.to_external());
            }
            ccadical_add(solver_full, 0)
        }
    }

    // Setup the evolutionary algorithm:
    let options = Options {
        seed: args.seed,
        ban_used_variables: args.ban_used,
        ..DEFAULT_OPTIONS
    };
    let mut algorithm = Algorithm::new(solver, options);

    // Bans some variables:
    if let Some(bans) = args.bans {
        let bans = parse_comma_separated_intervals(&bans);
        trace!("bans = {:?}", bans);
        for i in bans {
            assert!(i > 0);
            algorithm.banned[i - 1] = true;
        }
    }

    // Create and open the file with results:
    let mut file_results = if let Some(path_results) = &args.path_results {
        let f = File::create(path_results)?;
        let f = LineWriter::new(f);
        Some(f)
    } else {
        None
    };

    let mut cubes_product: Vec<Vec<Lit>> = vec![vec![]];

    if let Some(f) = &mut file_results {
        writeln!(f, "i,retain,size")?;
    }

    for run_number in 1..=args.num_runs {
        // Run the evolutionary algorithm:
        let result = algorithm.run(args.backdoor_size, args.num_iters, None, Some(0.999), 0);
        let backdoor = result.best_instance.get_variables();
        let (hard, easy) = partition_tasks(&backdoor, &mut algorithm.solver);
        debug!(
            "Backdoor {} has {} hard and {} easy tasks",
            DisplaySlice(&backdoor),
            hard.len(),
            easy.len()
        );

        if hard.len() == 1 {
            info!("Adding {} units to the solver", hard.len());
            for &lit in &hard[0] {
                algorithm.solver.add_clause(&[lit]);
                unsafe {
                    ccadical_add(solver_full, lit.to_external());
                    ccadical_add(solver_full, 0);
                }
            }
        }

        info!(
            "Going to produce a product of size {} * {} = {}",
            cubes_product.len(),
            hard.len(),
            cubes_product.len() * hard.len()
        );
        cubes_product = cubes_product
            .into_iter()
            .cartesian_product(hard)
            .map(|(a, b)| concat_cubes(a, b))
            .collect_vec();

        if let Some(f) = &mut file_results {
            writeln!(f, "{},before,{}", run_number, cubes_product.len())?;
        }

        info!("Size of product before retain: {}", cubes_product.len());
        let c = CString::new("conflicts").expect("CString::new failed");
        let pb = ProgressBar::new(cubes_product.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta})")?
                .progress_chars("#>-"),
        );
        cubes_product.retain(|cube| {
            pb.inc(1);

            // let res = algorithm.solver.propcheck(cube);
            // if res {
            //     // debug!("UNKNOWN {} via UP", DisplaySlice(cube));
            // } else {
            //     // debug!("UNSAT {} via UP", DisplaySlice(cube));
            // }
            // res

            unsafe {
                // debug!("cube = {}", DisplaySlice(cube));
                for &lit in cube.iter() {
                    ccadical_assume(solver_full, lit.to_external());
                }
                ccadical_limit(solver_full, c.as_ptr(), args.num_conflicts as i32);
                match ccadical_solve(solver_full) {
                    0 => {
                        // UNKNOWN
                        true
                    }
                    10 => {
                        // SAT
                        false
                    }
                    20 => {
                        // UNSAT
                        false
                    }
                    r => panic!("Unexpected result: {}", r),
                }
            }
        });
        pb.finish_and_clear();
        info!("Size of product after retain: {}", cubes_product.len());

        if let Some(f) = &mut file_results {
            writeln!(f, "{},after,{}", run_number, cubes_product.len())?;
        }

        if cubes_product.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);
            break;
        }
    }

    let elapsed = Instant::now() - start_time;
    println!("\nAll done in {:.3} s", elapsed.as_secs_f64());
    Ok(())
}

fn parse_comma_separated_intervals(input: &str) -> Vec<usize> {
    let mut result = Vec::new();
    for part in input.split(',') {
        let range_parts: Vec<&str> = part.splitn(2, "-").collect();
        if range_parts.len() == 2 {
            let start: usize = range_parts[0].parse().unwrap();
            let end: usize = range_parts[1].parse().unwrap();
            if start <= end {
                result.extend(start..=end);
            } else {
                result.extend((end..=start).rev());
            }
        } else {
            let single: usize = part.parse().unwrap();
            result.push(single);
        }
    }
    result
}

fn concat_cubes(a: Vec<Lit>, b: Vec<Lit>) -> Vec<Lit> {
    let mut r = HashSet::new();
    r.extend(a);
    r.extend(b);
    r.into_iter().collect()
}
