use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use itertools::Itertools;
use log::{debug, info};

use backdoor::derivation::derive_clauses;
use backdoor::searcher::{BackdoorSearcher, Options, DEFAULT_OPTIONS};
use backdoor::solvers::SatSolver;
use backdoor::utils::{create_line_writer, determine_vars_pool, get_hard_tasks, write_clause};

use cadical::statik::Cadical;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::utils::{parse_dimacs, DisplaySlice};
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

    /// Do derive clauses from backdoors?
    #[arg(long)]
    derive: bool,

    /// Do dump derived clauses after each EA run?
    #[arg(long)]
    dump_derived: bool,

    /// Use Cadical.
    #[arg(long)]
    use_cadical: bool,

    /// Freeze variables.
    #[arg(long)]
    freeze: bool,

    /// Derive ternary clauses.
    #[arg(long)]
    derive_ternary: bool,

    /// Danya's propcheck-based heuristic.
    #[arg(long, value_name = "INT")]
    pool_limit: Option<usize>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,simple_sat::solver=info,backdoor::derivation=info"))
        .init();

    let start_time = Instant::now();
    let args = Cli::parse();
    info!("args = {:?}", args);

    // Initialize SAT solver:
    let solver = if args.use_cadical {
        let solver = Cadical::new();
        for clause in parse_dimacs(&args.path_cnf) {
            solver.add_clause(clause.into_iter().map(|lit| lit.to_external()));
        }
        if args.freeze {
            info!("Freezing variables...");
            for i in 0..solver.vars() {
                let lit = (i + 1) as i32;
                solver.freeze(lit).unwrap();
            }
        }
        solver.limit("conflicts", 0);
        solver.solve()?;
        SatSolver::new_cadical(solver)
    } else {
        let mut solver = Solver::default();
        solver.init_from_file(&args.path_cnf);
        solver.propagate();
        solver.simplify();
        SatSolver::new_simple(solver)
    };

    // Create the pool of variables available for EA:
    let pool: Vec<Var> = determine_vars_pool(&args.path_cnf, &args.allowed_vars, &args.banned_vars);

    // Set up the evolutionary algorithm:
    let options = Options {
        seed: args.seed,
        ban_used_variables: args.ban_used,
        ..DEFAULT_OPTIONS
    };
    let mut searcher = BackdoorSearcher::new(solver, pool, options);

    // Create and open the file with resulting backdoors:
    let mut file_backdoors = args.path_output.as_ref().map(create_line_writer);

    // Create and open the file with derived clauses:
    let mut file_derived_clauses = if args.dump_derived {
        Some(create_line_writer("derived_clauses.txt"))
    } else {
        None
    };

    // Set of ALL clauses (original + derived):
    let mut all_clauses: HashSet<Vec<Lit>> = HashSet::new();
    for mut clause in parse_dimacs(&args.path_cnf) {
        clause.sort_by_key(|lit| lit.inner());
        all_clauses.insert(clause);
    }

    // All derived clauses:
    let mut all_derived_clauses: Vec<Vec<Lit>> = Vec::new();

    for run_number in 1..=args.num_runs {
        info!("Run {} / {}", run_number, args.num_runs);
        let time_run = Instant::now();

        // Run the evolutionary algorithm:
        let result = searcher.run(
            args.backdoor_size,
            args.num_iters,
            args.stagnation_limit,
            Some(args.max_rho),
            args.min_iter,
            args.pool_limit,
        );
        assert!(result.best_fitness.num_hard > 0, "Found strong backdoor?!..");

        // Derive clauses from the best backdoor:
        if args.derive {
            let backdoor = result.best_instance.get_variables();
            // let (hard, easy) = partition_tasks_cadical(&backdoor, &algorithm.solver);
            // debug!(
            //     "Backdoor {} has {} hard and {} easy tasks",
            //     DisplaySlice(&backdoor),
            //     hard.len(),
            //     easy.len()
            // );
            let hard = get_hard_tasks(&backdoor, &mut searcher.solver);
            debug!("Backdoor {} has {} hard tasks", DisplaySlice(&backdoor), hard.len());
            assert_eq!(hard.len() as u64, result.best_fitness.num_hard);

            // TODO: handle the case when `hard.len() == 1`

            let time_derive = Instant::now();
            let derived_clauses = derive_clauses(&hard, args.derive_ternary);
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
                if all_clauses.insert(lemma.clone()) {
                    if let Some(f) = &mut file_derived_clauses {
                        write_clause(f, &lemma)?;
                    }
                    searcher.solver.add_clause(&lemma);
                    new_clauses.push(lemma.clone());
                    all_derived_clauses.push(lemma);
                }
            }
            match &mut searcher.solver {
                SatSolver::SimpleSat(solver) => {
                    solver.simplify();
                }
                SatSolver::Cadical(solver) => {
                    solver.limit("conflicts", 0);
                    solver.solve()?;
                }
            }
            debug!(
                "Derived {} new clauses ({} units, {} binary, {} other)",
                new_clauses.len(),
                new_clauses.iter().filter(|c| c.len() == 1).count(),
                new_clauses.iter().filter(|c| c.len() == 2).count(),
                new_clauses.iter().filter(|c| c.len() > 2).count()
            );
            // debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

            let time_run = time_run.elapsed();
            info!("Done run {} / {} in {:.1}s", run_number, args.num_runs, time_run.as_secs_f64());
            info!(
                "So far derived {} new clauses ({} units, {} binary, {} other)",
                all_derived_clauses.len(),
                all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
                all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
                all_derived_clauses.iter().filter(|c| c.len() > 2).count()
            );
        }

        // Write the best found backdoor to the output file:
        if let Some(f) = &mut file_backdoors {
            writeln!(
                f,
                "Backdoor {} of size {} on iter {} with fitness = {}, rho = {}, hard = {} in {:.3} ms",
                result.best_instance,
                result.best_instance.len(),
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

    if args.derive {
        info!(
            "Total derived {} new clauses ({} units, {} binary, {} other)",
            all_derived_clauses.len(),
            all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
            all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
            all_derived_clauses.iter().filter(|c| c.len() > 2).count()
        );
    }

    println!("\nAll done in {:.3} s", start_time.elapsed().as_secs_f64());
    Ok(())
}
