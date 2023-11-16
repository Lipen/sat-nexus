use clap::Parser;
use log::info;
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
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,simple_sat::solver=info")).init();

    let start_time = Instant::now();
    let args = Cli::parse();
    info!("args = {:?}", args);

    let mut solver = Solver::default();
    solver.init_from_file(&args.path_cnf);
    let mut algorithm = Algorithm::new(solver, args.seed);
    algorithm.run(args.backdoor_size, args.num_iters);

    let elapsed = Instant::now() - start_time;
    println!("\nAll done in {:.3} s", elapsed.as_secs_f64());
    Ok(())
}
