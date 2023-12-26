use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use log::{debug, info};

use backdoor::algorithm::{Algorithm, Options, DEFAULT_OPTIONS};
use backdoor::derivation::derive_clauses;
use backdoor::utils::{create_line_writer, determine_vars_pool, partition_tasks};

// use cadical_sys::statik::*;
use simple_sat::solver::Solver;
use simple_sat::utils::DisplaySlice;
use simple_sat::var::Var;

// Run this example:
// cargo run -p backdoor --bin product-derive -- data/mult/lec_CvK_12.cnf --backdoor-size 10 --num-iters 10000 --num-runs 1000

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// Input file with CNF in DIMACS format.
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

    /// Path to a file with derived clauses.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    path_output: Option<PathBuf>,

    /// Path to a file with results (in CSV format).
    #[arg(long = "results", value_name = "FILE")]
    path_results: Option<PathBuf>,

    /// Random seed.
    #[arg(long, value_name = "INT", default_value_t = DEFAULT_OPTIONS.seed)]
    seed: u64,

    /// Comma-separated list of allowed variables (1-based indices).
    #[arg(long = "allow", value_name = "INT...")]
    allowed_vars: Option<String>,

    /// Comma-separated list of banned variables (1-based indices).
    #[arg(long = "ban", value_name = "INT...")]
    banned_vars: Option<String>,

    /// Do ban variables used in best backdoors on previous runs?
    #[arg(long)]
    ban_used: bool,

    /// Number of stagnated iterations before re-initialization.
    #[arg(long, value_name = "INT")]
    stagnation_limit: Option<usize>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,simple_sat::solver=info,backdoor::derivation=info"))
        .init();

    let start_time = Instant::now();
    let args = Cli::parse();
    info!("args = {:?}", args);

    // Initialize the SAT solver:
    let mut solver = Solver::default();
    solver.init_from_file(&args.path_cnf);

    // let solver_full = unsafe { ccadical_init() };
    // unsafe {
    //     for clause in solver.clauses_iter() {
    //         for lit in clause.lits() {
    //             ccadical_add(solver_full, lit.to_external());
    //         }
    //         ccadical_add(solver_full, 0)
    //     }
    // }

    // Create the pool of variables available for EA:
    let mut pool: Vec<Var> = determine_vars_pool(&solver, &args.allowed_vars, &args.banned_vars);

    // Set up the evolutionary algorithm:
    let options = Options {
        seed: args.seed,
        ban_used_variables: args.ban_used,
        ..DEFAULT_OPTIONS
    };
    let mut algorithm = Algorithm::new(solver, options);

    // Create and open the file with derived clauses:
    let mut file_derived_clauses = args.path_output.as_ref().map(create_line_writer);

    // Create and open the file with results:
    // let mut file_results = args.path_results.as_ref().map(create_line_writer);

    // if let Some(f) = &mut file_results {
    //     writeln!(f, "...")?;
    // }

    for run_number in 1..=args.num_runs {
        info!("EA run {} / {}", run_number, args.num_runs);
        let time_run = Instant::now();

        let result = algorithm.run(
            &mut pool,
            args.backdoor_size,
            args.num_iters,
            args.stagnation_limit,
            Some(((1u64 << args.backdoor_size) - 1) as f64 / (1u64 << args.backdoor_size) as f64),
            100,
        );
        let backdoor = result.best_instance.get_variables();
        let (hard, easy) = partition_tasks(&backdoor, &mut algorithm.solver);
        debug!(
            "Backdoor {} has {} hard and {} easy tasks",
            DisplaySlice(&backdoor),
            hard.len(),
            easy.len()
        );

        if hard.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);
            break;
        }
        if hard.len() == 1 {
            info!("Adding {} units to the solver", hard[0].len());
            for &lit in &hard[0] {
                if algorithm.derived_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    algorithm.solver.add_learnt(&[lit]);
                    // unsafe {
                    //     ccadical_add(solver_full, lit.to_external());
                    //     ccadical_add(solver_full, 0);
                    // }
                }
            }
            continue;
        }

        // ------------------------------------------------------------------------

        info!("Deriving clauses for {} cubes...", hard.len());
        // if hard.len() <= 30 {
        //     for cube in hard.iter() {
        //         debug!("cube = {}", DisplaySlice(&cube));
        //     }
        // }
        let time_derive = Instant::now();
        let derived_clauses = derive_clauses(&hard);
        let time_derive = time_derive.elapsed();
        info!(
            "Derived {} clauses ({} units, {} binary, {} other) for backdoor in {:.1}s",
            derived_clauses.len(),
            derived_clauses.iter().filter(|c| c.len() == 1).count(),
            derived_clauses.iter().filter(|c| c.len() == 2).count(),
            derived_clauses.iter().filter(|c| c.len() > 2).count(),
            time_derive.as_secs_f64()
        );
        // debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        let mut new_clauses = Vec::new();
        for mut lemma in derived_clauses {
            lemma.sort_by_key(|lit| lit.inner());
            if algorithm.derived_clauses.insert(lemma.clone()) {
                if let Some(f) = &mut file_derived_clauses {
                    for lit in lemma.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }
                algorithm.solver.add_learnt(&lemma);
                // unsafe {
                //     for lit in lemma.iter() {
                //         ccadical_add(solver_full, lit.to_external());
                //     }
                //     ccadical_add(solver_full, 0);
                // }
                new_clauses.push(lemma);
            }
        }
        debug!(
            "NEW {} clauses ({} units, {} binary, {} other)",
            new_clauses.len(),
            new_clauses.iter().filter(|c| c.len() == 1).count(),
            new_clauses.iter().filter(|c| c.len() == 2).count(),
            new_clauses.iter().filter(|c| c.len() > 2).count()
        );
        // debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        let time_run = time_run.elapsed();
        info!("Done run {} / {} in {:.1}s", run_number, args.num_runs, time_run.as_secs_f64());
        info!(
            "So far derived {} clauses ({} units, {} binary, {} other)",
            algorithm.derived_clauses.len(),
            algorithm.derived_clauses.iter().filter(|c| c.len() == 1).count(),
            algorithm.derived_clauses.iter().filter(|c| c.len() == 2).count(),
            algorithm.derived_clauses.iter().filter(|c| c.len() > 2).count()
        );
    }

    info!("Finished {} runs in {:.1}s", args.num_runs, start_time.elapsed().as_secs_f64());
    info!(
        "Total derived {} clauses ({} units, {} binary, {} other)",
        algorithm.derived_clauses.len(),
        algorithm.derived_clauses.iter().filter(|c| c.len() == 1).count(),
        algorithm.derived_clauses.iter().filter(|c| c.len() == 2).count(),
        algorithm.derived_clauses.iter().filter(|c| c.len() > 2).count()
    );

    println!("\nAll done in {:.3} s", start_time.elapsed().as_secs_f64());
    Ok(())
}
