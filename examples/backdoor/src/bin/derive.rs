use std::fs::File;
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use backdoor::derivation::derive_clauses;
use backdoor::utils::{parse_comma_separated_intervals, partition_tasks};

use clap::Parser;
use itertools::Itertools;
use log::{debug, info};

use simple_sat::solver::Solver;
use simple_sat::utils::DisplaySlice;
use simple_sat::var::Var;

// Run this example:
// cargo run -p backdoor --bin derive -- data/mult/lec_CvK_12.cnf --backdoor 1-10

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    #[arg(value_name = "CNF")]
    path_cnf: PathBuf,

    /// Backdoor.
    #[arg(long, value_name = "INT...")]
    backdoor: String,

    /// Path to a file with results.
    #[arg(long = "results", value_name = "FILE")]
    path_results: Option<PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,simple_sat::solver=info,backdoor::derivation=info"))
        .init();

    let start_time = Instant::now();
    let args = Cli::parse();
    debug!("args = {:?}", args);

    let backdoor = parse_comma_separated_intervals(&args.backdoor)
        .into_iter()
        .map(|x| Var::from_external(x as u32))
        .collect_vec();
    // debug!("backdoor = {}", DisplaySlice(&backdoor));

    // Initialize the SAT solver:
    let mut solver = Solver::default();
    solver.init_from_file(&args.path_cnf);

    // Create and open the file with results:
    let mut file_results = if let Some(path_results) = &args.path_results {
        let f = File::create(path_results)?;
        let f = LineWriter::new(f);
        Some(f)
    } else {
        None
    };

    let (hard, easy) = partition_tasks(&backdoor, &mut solver);
    info!(
        "Backdoor {} has {} hard and {} easy tasks",
        DisplaySlice(&backdoor),
        hard.len(),
        easy.len()
    );

    // debug!("{} easy tasks:", easy.len());
    // for cube in easy.iter() {
    //     debug!("  {}", DisplaySlice(cube));
    // }
    // debug!("{} hard tasks:", hard.len());
    // for cube in hard.iter() {
    //     debug!("  {}", DisplaySlice(cube));
    // }

    info!("Deriving clauses for {} cubes...", hard.len());
    let time_derive = Instant::now();
    let derived_clauses = derive_clauses(&hard);
    let time_derive = time_derive.elapsed();
    info!(
        "Total {} derived clauses ({} units, {} binary, {} other) for backdoor in {:.1}s",
        derived_clauses.len(),
        derived_clauses.iter().filter(|c| c.len() == 1).count(),
        derived_clauses.iter().filter(|c| c.len() == 2).count(),
        derived_clauses.iter().filter(|c| c.len() > 2).count(),
        time_derive.as_secs_f64()
    );

    if let Some(f) = &mut file_results {
        writeln!(f, "hard,easy,derived,units,binary,other,time")?;
        writeln!(
            f,
            "{},{},{},{},{},{},{}",
            hard.len(),
            easy.len(),
            derived_clauses.len(),
            derived_clauses.iter().filter(|c| c.len() == 1).count(),
            derived_clauses.iter().filter(|c| c.len() == 2).count(),
            derived_clauses.iter().filter(|c| c.len() > 2).count(),
            time_derive.as_secs_f64()
        )?;
    }

    println!("\nAll done in {:.3} s", start_time.elapsed().as_secs_f64());
    Ok(())
}
