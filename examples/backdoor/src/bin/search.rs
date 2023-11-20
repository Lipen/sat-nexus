use clap::Parser;
use itertools::Itertools;
use log::info;
use std::fs::OpenOptions;
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use simple_sat::solver::Solver;

use backdoor::algorithm::Algorithm;

// Run this example:
// cargo run -p backdoor --bin search -- data/mult/lec_CvK_12.cnf --backdoor-size 10 --num-iters 1000

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    #[arg(value_name = "CNF")]
    path_cnf: PathBuf,

    /// Backdoor size.
    #[arg(long)]
    backdoor_size: usize,

    /// Number of EA iterations.
    #[arg(long)]
    num_iters: usize,

    /// Number of EA runs.
    #[arg(long, default_value_t = 1)]
    num_runs: usize,

    /// Random seed.
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Results.
    #[arg(long = "results")]
    path_results: Option<PathBuf>,

    /// Do dump learnts after each EA run?
    #[arg(long)]
    dump_learnts: bool,
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

    // Setup the evolutionary algorithm:
    let mut algorithm = Algorithm::new(solver, args.seed);

    // Create and open the file with results:
    let mut f = if let Some(path_results) = &args.path_results {
        let f = OpenOptions::new().write(true).create(true).truncate(true).open(path_results)?;
        let f = LineWriter::new(f);
        Some(f)
    } else {
        None
    };

    for run_number in 1..=args.num_runs {
        info!("EA run {} / {}", run_number, args.num_runs);

        // Run the evolutionary algorithm:
        let result = algorithm.run(args.backdoor_size, args.num_iters);

        // Dump learnts:
        if args.dump_learnts {
            let lf = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(format!("learnts_{}.txt", run_number))?;
            let mut lf = LineWriter::new(lf);
            for learnt in algorithm.solver.learnts_iter() {
                for lit in learnt.iter() {
                    write!(lf, "{} ", lit)?;
                }
                writeln!(lf, " 0")?;
            }
        }

        // Write the best found backdoor into the resulting file:
        if let Some(f) = &mut f {
            // Note: variables in backdoors are reported 1-based.
            writeln!(
                f,
                "Backdoor [{}] of size {} on iter {} with fitness = {}, rho = {}, hard = {} in {:.3} ms",
                result.best_instance.get_variables().iter().map(|v| v.0 + 1).join(", "),
                result.best_instance.weight(),
                result.best_iteration,
                result.best_fitness.value,
                result.best_fitness.rho,
                result.best_fitness.num_hard,
                result.time.as_secs_f64() * 1000.0
            )?;
        }
        assert!(result.best_fitness.num_hard > 0, "Found strong backdoor?!..");
    }

    let elapsed = Instant::now() - start_time;
    println!("\nAll done in {:.3} s", elapsed.as_secs_f64());
    Ok(())
}
