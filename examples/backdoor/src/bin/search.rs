use clap::Parser;
use itertools::Itertools;
use log::{debug, info, trace};

use std::fs::OpenOptions;
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use backdoor::algorithm::{Algorithm, Options, DEFAULT_OPTIONS};
use backdoor::minimization::minimize_backdoor;
use backdoor::utils::partition_tasks;
use simple_sat::solver::Solver;
use simple_sat::utils::DisplaySlice;

// Run this example:
// cargo run -p backdoor --bin search -- data/mult/lec_CvK_12.cnf --backdoor-size 10 --num-iters 1000

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

    /// Number of EA runs.
    #[arg(long, value_name = "INT", default_value_t = 1)]
    num_runs: usize,

    /// Random seed.
    #[arg(long, value_name="INT", default_value_t = DEFAULT_OPTIONS.seed)]
    seed: u64,

    /// Path to a file with results.
    #[arg(long = "results", value_name = "FILE")]
    path_results: Option<PathBuf>,

    /// Do dump learnts after each EA run?
    #[arg(long)]
    dump_learnts: bool,

    /// Do add learnts after analyzing conflicts in `propcheck_all_tree`?
    #[arg(long)]
    add_learnts: bool,

    /// Do ban variables used in best backdoors on previous runs?
    #[arg(long)]
    ban_used: bool,

    /// Comma-separated list of banned variables (1-based indices).
    #[arg(long, value_name = "INT...")]
    bans: Option<String>,

    /// Do minimize the best backdoors into clauses using Espresso?
    #[arg(long)]
    minimize: bool,
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
    let options = Options {
        seed: args.seed,
        add_learnts_in_propcheck_all_tree: args.add_learnts,
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
    let mut f = if let Some(path_results) = &args.path_results {
        let f = OpenOptions::new().write(true).create(true).truncate(true).open(path_results)?;
        let f = LineWriter::new(f);
        Some(f)
    } else {
        None
    };

    // let fm = OpenOptions::new().write(true).create(true).truncate(true).open("learnts.txt")?;
    // let mut fm = LineWriter::new(fm);

    for run_number in 1..=args.num_runs {
        info!("EA run {} / {}", run_number, args.num_runs);

        // Run the evolutionary algorithm:
        let result = algorithm.run(args.backdoor_size, args.num_iters);

        if args.minimize {
            // Minimize the best backdoor:
            let backdoor = result.best_instance.get_variables();
            let (hard, easy) = partition_tasks(&backdoor, &mut algorithm.solver);
            debug!("Backdoor has {} hard and {} easy tasks", hard.len(), easy.len());
            let clauses = minimize_backdoor(&easy);
            debug!(
                "Total {} minimized clauses: [{}]",
                clauses.len(),
                clauses.iter().map(|c| DisplaySlice(c)).join(", ")
            );

            // Add the minimized clauses as learnts:
            for lemma in clauses.iter() {
                algorithm.solver.add_learnt(lemma);
                // for lit in lemma.iter() {
                //     write!(fm, "{} ", lit)?;
                // }
                // writeln!(fm, "0")?;
            }

            // Note: currently, `add_learnt` does the propagation and simplification at the end.
            //
            // // Post-propagate:
            // if let Some(conflict) = algorithm.solver.propagate() {
            //     warn!("Conflict during post-propagation: {}", algorithm.solver.clause(conflict));
            // }
            //
            // // Simplify:
            // algorithm.solver.simplify();
        }

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
            let single: usize = input.parse().unwrap();
            result.push(single);
        }
    }
    result
}
