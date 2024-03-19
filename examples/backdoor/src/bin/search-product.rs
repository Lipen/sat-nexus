use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::{iproduct, Itertools};
use log::{debug, info};

use backdoor::searcher::{BackdoorSearcher, Options, DEFAULT_OPTIONS};
use backdoor::solvers::SatSolver;
use backdoor::utils::{clause_to_external, concat_cubes, create_line_writer, determine_vars_pool, get_hard_tasks};

use cadical::statik::Cadical;
use cadical::{LitValue, SolveResponse};
use simple_sat::lit::Lit;
use simple_sat::utils::{parse_dimacs, DisplaySlice};
use simple_sat::var::Var;

// Run this example:
// cargo run -p backdoor --bin search-product -- data/mult/lec_CvK_12.cnf --backdoor-size 10 --num-iters 10000

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

    /// Number of conflicts.
    #[arg(long, value_name = "INT", default_value_t = 1000)]
    num_conflicts: usize,

    /// Path to a file with results.
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

    /// Freeze variables.
    #[arg(long)]
    freeze: bool,

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

    // Initialize Cadical:
    let solver = Cadical::new();
    for clause in parse_dimacs(&args.path_cnf) {
        solver.add_clause(clause.into_iter().map(|lit| lit.to_external()));
    }
    if args.freeze {
        for i in 0..solver.vars() {
            let lit = (i + 1) as i32;
            solver.freeze(lit).unwrap();
        }
    }
    solver.limit("conflicts", 0);
    solver.solve()?;

    // Create the pool of variables available for EA:
    let pool: Vec<Var> = determine_vars_pool(&args.path_cnf, &args.allowed_vars, &args.banned_vars);

    // Set up the evolutionary algorithm:
    let options = Options {
        seed: args.seed,
        ban_used_variables: args.ban_used,
        ..DEFAULT_OPTIONS
    };
    let mut searcher = BackdoorSearcher::new(SatSolver::new_cadical(solver), pool, options);

    // Create and open the file with derived clauses:
    let mut file_derived_clauses = Some(create_line_writer("derived_clauses.txt"));

    // Create and open the file with results:
    let mut file_results = args.path_results.as_ref().map(create_line_writer);
    if let Some(f) = &mut file_results {
        writeln!(f, "run,status,size")?;
    }

    // Set of ALL clauses (original + derived):
    let mut all_clauses: HashSet<Vec<Lit>> = HashSet::new();
    for mut clause in parse_dimacs(&args.path_cnf) {
        clause.sort_by_key(|lit| lit.inner());
        all_clauses.insert(clause);
    }

    // All derived clauses:
    let mut all_derived_clauses: Vec<Vec<Lit>> = Vec::new();

    // Cartesian product of hard tasks:
    let mut cubes_product: Vec<Vec<Lit>> = vec![vec![]];

    let time_runs = Instant::now();

    let mut run_number = 0;
    loop {
        run_number += 1;
        info!("Run {}", run_number);
        let time_run = Instant::now();

        let result = searcher.run(
            args.backdoor_size,
            args.num_iters,
            args.stagnation_limit,
            Some(((1u64 << args.backdoor_size) - 1) as f64 / (1u64 << args.backdoor_size) as f64),
            0,
            args.pool_limit,
        );
        assert!(result.best_fitness.num_hard > 0, "Found strong backdoor?!..");

        let backdoor = result.best_instance.get_variables();
        let hard = get_hard_tasks(&backdoor, &mut searcher.solver);
        debug!("Backdoor {} has {} hard tasks", DisplaySlice(&backdoor), hard.len());
        assert_eq!(hard.len() as u64, result.best_fitness.num_hard);

        if hard.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);
            break;
        }
        if hard.len() == 1 {
            info!("Adding {} units to the solver", hard[0].len());
            for &lit in &hard[0] {
                if all_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    searcher.solver.add_clause(&[lit]);
                    all_derived_clauses.push(vec![lit]);
                }
            }
            continue;
        }

        info!(
            "Going to produce a product of size {} * {} = {}",
            cubes_product.len(),
            hard.len(),
            cubes_product.len() * hard.len()
        );
        if let Some(f) = &mut file_results {
            writeln!(f, "{},product,{}", run_number, cubes_product.len() * hard.len())?;
        }
        let variables = {
            let mut s = HashSet::new();
            s.extend(cubes_product[0].iter().map(|lit| lit.var()));
            s.extend(hard[0].iter().map(|lit| lit.var()));
            s.into_iter().sorted().collect_vec()
        };
        debug!("Total {} variables: {}", variables.len(), DisplaySlice(&variables));

        let mut new_cubes_product = Vec::new();
        'out: for (old, new) in iproduct!(cubes_product, hard) {
            let cube = concat_cubes(old, new);
            for i in 1..cube.len() {
                if cube[i] == -cube[i - 1] {
                    // Skip the cube with inconsistent literals:
                    // log::warn!("Skipping the concatenated cube {} with inconsistent literals", DisplaySlice(&cube));
                    continue 'out;
                }
            }
            new_cubes_product.push(cube);
        }
        cubes_product = new_cubes_product;
        info!("Product now contains {} hard cubes", cubes_product.len());

        info!("Filtering {} hard cubes via solver...", cubes_product.len());
        let time_filter = Instant::now();
        let pb = ProgressBar::new(cubes_product.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")?
                .progress_chars("#>-"),
        );
        pb.set_message("filtering");
        cubes_product.retain(|cube| {
            pb.inc(1);

            match &mut searcher.solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    for &lit in cube.iter() {
                        solver.assume(lit.to_external()).unwrap();
                    }
                    solver.limit("conflicts", args.num_conflicts as i32);
                    match solver.solve().unwrap() {
                        SolveResponse::Interrupted => true,
                        SolveResponse::Unsat => {
                            let mut lemma = Vec::new();
                            for &lit in cube {
                                if solver.failed(lit.to_external()).unwrap() {
                                    lemma.push(-lit);
                                }
                            }
                            // debug!("UNSAT for cube = {}, lemma = {}", DisplaySlice(&cube), DisplaySlice(&lemma));
                            lemma.sort_by_key(|lit| lit.inner());
                            if lemma.len() <= 5 && all_clauses.insert(lemma.clone()) {
                                pb.println(format!("new lemma from unsat core: {}", DisplaySlice(&lemma)));
                                if let Some(f) = &mut file_derived_clauses {
                                    for lit in lemma.iter() {
                                        write!(f, "{} ", lit).unwrap();
                                    }
                                    writeln!(f, "0").unwrap();
                                }
                                solver.add_clause(clause_to_external(&lemma));
                                all_derived_clauses.push(lemma);
                            }
                            false
                        }
                        SolveResponse::Sat => {
                            let model: Vec<_> = (1..=solver.vars()).map(|i| solver.val(i as i32).unwrap()).collect();
                            {
                                let f = File::create("model.txt").unwrap();
                                let mut f = BufWriter::new(f);
                                writeln!(
                                    f,
                                    "{}",
                                    model
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &value)| {
                                            let lit = (i + 1) as i32;
                                            match value {
                                                LitValue::True => lit,
                                                LitValue::False => -lit,
                                            }
                                        })
                                        .join(" ")
                                )
                                .unwrap();
                            }
                            {
                                let f = File::create("model.cnf").unwrap();
                                let mut f = BufWriter::new(f);
                                for (i, &value) in model.iter().enumerate() {
                                    let lit = (i + 1) as i32;
                                    let lit = match value {
                                        LitValue::True => lit,
                                        LitValue::False => -lit,
                                    };
                                    writeln!(f, "{} 0", lit).unwrap();
                                }
                            }
                            panic!("unexpected SAT");
                            // false
                        }
                    }
                }
            }
        });
        pb.finish_and_clear();
        let time_filter = time_filter.elapsed();
        info!(
            "Filtered down to {} cubes in {:.1}s",
            cubes_product.len(),
            time_filter.as_secs_f64()
        );
        if let Some(f) = &mut file_results {
            writeln!(f, "{},limited,{}", run_number, cubes_product.len())?;
        }

        if cubes_product.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);
            break;
        }
        if cubes_product.len() == 1 {
            info!("Adding {} units to the solver", cubes_product[0].len());
            for &lit in &cubes_product[0] {
                searcher.pool.retain(|&v| v != lit.var());
                if all_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    searcher.solver.add_clause(&[lit]);
                    all_derived_clauses.push(vec![lit]);
                }
            }
            cubes_product = vec![vec![]];
            continue;
        }

        let time_run = time_run.elapsed();
        info!("Done run {} in {:.1}s", run_number, time_run.as_secs_f64());
        info!(
            "So far derived {} new clauses ({} units, {} binary, {} other)",
            all_derived_clauses.len(),
            all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
            all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
            all_derived_clauses.iter().filter(|c| c.len() > 2).count()
        );
    }

    let time_runs = time_runs.elapsed();
    info!("Finished {} runs in {:.1}s", run_number, time_runs.as_secs_f64());
    info!(
        "Total derived {} new clauses ({} units, {} binary, {} other)",
        all_derived_clauses.len(),
        all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
        all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
        all_derived_clauses.iter().filter(|c| c.len() > 2).count()
    );

    println!("\nAll done in {:.3} s", start_time.elapsed().as_secs_f64());
    Ok(())
}
