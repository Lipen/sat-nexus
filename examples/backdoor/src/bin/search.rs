use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use itertools::Itertools;
use log::{debug, info};

use backdoor::algorithm::{Algorithm, Options, DEFAULT_OPTIONS};
use backdoor::derivation::derive_clauses;
use backdoor::utils::{create_line_writer, determine_vars_pool, partition_tasks};

use simple_sat::solver::Solver;
use simple_sat::utils::DisplaySlice;
use simple_sat::var::Var;

// Run this example:
// cargo run -p backdoor --bin search -- data/mult/lec_CvK_12.cnf --backdoor-size 10 --num-iters 1000

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

    /// Number of EA runs.
    #[arg(long, value_name = "INT", default_value_t = 1)]
    num_runs: usize,

    /// Path to a output file with backdoors.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    path_output: Option<PathBuf>,

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

    /// Maximum required rho value (break EA upon reaching).
    #[arg(long, value_name = "FLOAT", default_value_t = 1.0)]
    max_rho: f64,

    /// Minimum number of EA iterations.
    #[arg(long, value_name = "INT", default_value_t = 0)]
    min_iter: usize,

    /// Do dump records for each EA run?
    #[arg(long)]
    dump_records: bool,

    /// Do add learnts after analyzing conflicts in `propcheck_all_tree`?
    #[arg(long)]
    add_learnts: bool,

    /// Do dump learnts after each EA run?
    #[arg(long)]
    dump_intermediate_learnts: bool,

    /// Do dump all learnts after all EA runs?
    #[arg(long)]
    dump_learnts: bool,

    /// Do derive clauses from backdoors?
    #[arg(long)]
    derive: bool,

    /// Do dump derived clauses after each EA run?
    #[arg(long)]
    dump_derived: bool,
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

    // Create the pool of variables available for EA:
    let pool: Vec<Var> = determine_vars_pool(&solver, &args.allowed_vars, &args.banned_vars);

    // Set up the evolutionary algorithm:
    let options = Options {
        seed: args.seed,
        add_learnts_in_propcheck_all_tree: args.add_learnts,
        ban_used_variables: args.ban_used,
        ..DEFAULT_OPTIONS
    };
    let mut algorithm = Algorithm::new(solver, pool, options);

    // Create and open the file with resulting backdoors:
    let mut file_backdoors = args.path_output.as_ref().map(create_line_writer);

    let mut file_derived_clauses = if args.dump_derived {
        Some(create_line_writer("derived_clauses.txt"))
    } else {
        None
    };

    for run_number in 1..=args.num_runs {
        info!("EA run {} / {}", run_number, args.num_runs);

        debug!("algorithm.derived_clauses.len() = {}", algorithm.derived_clauses.len());
        debug!("algorithm.learnt_clauses.len() = {}", algorithm.learnt_clauses.len());

        // Run the evolutionary algorithm:
        let result = algorithm.run(
            args.backdoor_size,
            args.num_iters,
            args.stagnation_limit,
            Some(args.max_rho),
            args.min_iter,
        );

        assert!(result.best_fitness.num_hard > 0, "Found strong backdoor?!..");

        // Derive clauses from the best backdoor:
        if args.derive {
            let backdoor = result.best_instance.get_variables();
            let (hard, easy) = partition_tasks(&backdoor, &mut algorithm.solver);
            debug!(
                "Backdoor {} has {} hard and {} easy tasks",
                DisplaySlice(&backdoor),
                hard.len(),
                easy.len()
            );

            let time_derive = Instant::now();
            let derived_clauses = derive_clauses(&hard);
            let time_derive = time_derive.elapsed();
            debug!(
                "Total {} derived clauses in {:.1} s",
                derived_clauses.len(),
                time_derive.as_secs_f64()
            );
            debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

            // Add the derived clauses to the solver:
            for mut lemma in derived_clauses {
                lemma.sort_by_key(|lit| lit.var().inner());

                algorithm.solver.add_learnt(&lemma);

                if let Some(f) = &mut file_derived_clauses {
                    for lit in lemma.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }

                algorithm.derived_clauses.insert(lemma);
            }
        }

        // Dump learnts:
        if args.dump_intermediate_learnts {
            let f = File::create(format!("learnts_{}.txt", run_number))?;
            let mut f = LineWriter::new(f);
            for learnt in algorithm.solver.learnts_iter() {
                for lit in learnt.iter() {
                    write!(f, "{} ", lit)?;
                }
                writeln!(f, " 0")?;
            }
        }

        // Write the best found backdoor to the output file:
        if let Some(f) = &mut file_backdoors {
            // Note: variables in backdoors are reported 1-based.
            writeln!(
                f,
                "Backdoor {} of size {} on iter {} with fitness = {}, rho = {}, hard = {} in {:.3} ms",
                DisplaySlice(&result.best_instance.get_variables()),
                result.best_instance.weight(),
                result.best_iteration,
                result.best_fitness.value,
                result.best_fitness.rho,
                result.best_fitness.num_hard,
                result.time.as_secs_f64() * 1000.0
            )?;
        }

        // Write the run records:
        if args.dump_records {
            let mut writer = csv::Writer::from_path(format!("run_{}.csv", run_number))?;
            writer.write_record(["iteration", "instance", "fitness", "num_hard", "rho"])?;
            for record in result.records {
                writer.serialize((
                    record.iteration,
                    record.instance.get_variables().iter().map(|v| v.to_external()).join(","),
                    record.fitness.value,
                    record.fitness.num_hard,
                    record.fitness.rho,
                ))?;
            }
        }
    }

    // Dump all learnts:
    if args.dump_learnts {
        let f = File::create("learnt_clauses.txt")?;
        let mut f = LineWriter::new(f);
        for lemma in algorithm.learnt_clauses.iter() {
            for lit in lemma.iter() {
                write!(f, "{} ", lit)?;
            }
            writeln!(f, "0")?;
        }
    }

    let elapsed = Instant::now() - start_time;
    println!("\nAll done in {:.3} s", elapsed.as_secs_f64());
    Ok(())
}
