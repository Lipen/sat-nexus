use std::fs::File;
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::{iproduct, Itertools};
use log::{debug, info, trace};

use backdoor::algorithm::{Algorithm, Options, DEFAULT_OPTIONS};
use backdoor::derivation::derive_clauses;
use backdoor::utils::{concat_cubes, parse_comma_separated_intervals, partition_tasks};
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::trie::Trie;
use simple_sat::utils::DisplaySlice;

// Run this example:
// cargo run -p backdoor --bin product-derive -- data/mult/lec_CvK_12.cnf --backdoor-size 10 --num-iters 10000 --num-runs 1000

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

    let mut file_derived_clauses = if true {
        let f = File::create("derived_clauses.txt")?;
        let f = LineWriter::new(f);
        Some(f)
    } else {
        None
    };

    let mut cubes_product: Vec<Vec<Lit>> = vec![vec![]];

    if let Some(f) = &mut file_results {
        writeln!(f, "i,filter,size")?;
    }

    for run_number in 1..=args.num_runs {
        info!("EA run {} / {}", run_number, args.num_runs);

        let result = algorithm.run(
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
                    algorithm.solver.add_clause(&[lit]);
                }
            }
            continue;
        }

        // ------------------------------------------------------------------------

        debug!("Deriving clauses for {} cubes...", hard.len());
        let derived_clauses = derive_clauses(&hard);
        debug!(
            "Total {} derived clauses ({} units, {} binary, {} other) for backdoor",
            derived_clauses.len(),
            derived_clauses.iter().filter(|c| c.len() == 1).count(),
            derived_clauses.iter().filter(|c| c.len() == 2).count(),
            derived_clauses.iter().filter(|c| c.len() > 2).count()
        );
        debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        let mut new_clauses = Vec::new();
        for mut lemma in derived_clauses {
            lemma.sort_by_key(|lit| lit.var().0);
            if algorithm.derived_clauses.insert(lemma.clone()) {
                if let Some(f) = &mut file_derived_clauses {
                    for lit in lemma.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }
                algorithm.solver.add_learnt(&lemma);
                new_clauses.push(lemma);
            }
        }
        debug!(
            "NEW {} derived clauses ({} units, {} binary, {} other) for backdoor",
            new_clauses.len(),
            new_clauses.iter().filter(|c| c.len() == 1).count(),
            new_clauses.iter().filter(|c| c.len() == 2).count(),
            new_clauses.iter().filter(|c| c.len() > 2).count()
        );
        debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        // ------------------------------------------------------------------------

        info!(
            "Going to produce a product of size {} * {} = {}",
            cubes_product.len(),
            hard.len(),
            cubes_product.len() * hard.len()
        );
        if let Some(f) = &mut file_results {
            writeln!(f, "{},before,{}", run_number, cubes_product.len() * hard.len())?;
        }

        let variables = concat_cubes(cubes_product[0].clone(), hard[0].clone())
            .iter()
            .map(|lit| lit.var())
            .collect_vec();
        let mut trie = Trie::new();
        let pb = ProgressBar::new((cubes_product.len() * hard.len()) as u64);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta})")?
                .progress_chars("#>-"),
        );
        for (old, new) in iproduct!(cubes_product, hard) {
            let cube = concat_cubes(old, new);
            trie.insert(cube.iter().map(|lit| lit.negated()));
            pb.inc(1);
        }
        pb.finish_and_clear();

        let mut valid = Vec::new();
        info!("Filtering hard cubes via trie of size {}...", trie.len());
        algorithm.solver.propcheck_all_trie(&variables, &trie, &mut valid);
        drop(trie);
        cubes_product = valid;

        info!("Size of product after filtering: {}", cubes_product.len());
        if let Some(f) = &mut file_results {
            writeln!(f, "{},after,{}", run_number, cubes_product.len())?;
        }

        if cubes_product.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);
            break;
        }
        if cubes_product.len() == 1 {
            info!("Adding {} units to the solver", cubes_product[0].len());
            for &lit in &cubes_product[0] {
                if algorithm.derived_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    algorithm.solver.add_clause(&[lit]);
                }
            }
            cubes_product = vec![vec![]];
            continue;
        }

        // ------------------------------------------------------------------------

        debug!("Deriving clauses for {} cubes...", cubes_product.len());
        let derived_clauses = derive_clauses(&cubes_product);
        debug!(
            "Total {} derived clauses ({} units, {} binary, {} other) AFTER filtering",
            derived_clauses.len(),
            derived_clauses.iter().filter(|c| c.len() == 1).count(),
            derived_clauses.iter().filter(|c| c.len() == 2).count(),
            derived_clauses.iter().filter(|c| c.len() > 2).count()
        );
        debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        let mut new_clauses = Vec::new();
        for mut lemma in derived_clauses {
            lemma.sort_by_key(|lit| lit.var().0);
            if algorithm.derived_clauses.insert(lemma.clone()) {
                if let Some(f) = &mut file_derived_clauses {
                    for lit in lemma.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }
                algorithm.solver.add_learnt(&lemma);
                new_clauses.push(lemma);
            }
        }
        debug!(
            "NEW {} derived clauses ({} units, {} binary, {} other) AFTER filtering",
            new_clauses.len(),
            new_clauses.iter().filter(|c| c.len() == 1).count(),
            new_clauses.iter().filter(|c| c.len() == 2).count(),
            new_clauses.iter().filter(|c| c.len() > 2).count()
        );
        debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        // ------------------------------------------------------------------------

        // ===
        continue;
        // ===

        let pb = ProgressBar::new(cubes_product.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta})")?
                .progress_chars("#>-"),
        );
        cubes_product.retain(|cube| {
            pb.inc(1);
            let res = algorithm.solver.propcheck(cube);
            // if res {
            //     // debug!("UNKNOWN {} via UP", DisplaySlice(cube));
            // } else {
            //     // debug!("UNSAT {} via UP", DisplaySlice(cube));
            // }
            res
        });
        pb.finish_and_clear();

        info!("Size of product after second filtering: {}", cubes_product.len());
        if let Some(f) = &mut file_results {
            writeln!(f, "{},after2,{}", run_number, cubes_product.len())?;
        }

        if cubes_product.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);
            break;
        }
        if cubes_product.len() == 1 {
            info!("Adding {} units to the solver", cubes_product[0].len());
            for &lit in &cubes_product[0] {
                if algorithm.derived_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    algorithm.solver.add_clause(&[lit]);
                }
            }
            cubes_product = vec![vec![]];
            continue;
        }

        // ------------------------------------------------------------------------

        debug!("Deriving clauses for {} cubes...", cubes_product.len());
        let derived_clauses = derive_clauses(&cubes_product);
        debug!(
            "Total {} derived clauses ({} units, {} binary, {} other) after second filtering",
            derived_clauses.len(),
            derived_clauses.iter().filter(|c| c.len() == 1).count(),
            derived_clauses.iter().filter(|c| c.len() == 2).count(),
            derived_clauses.iter().filter(|c| c.len() > 2).count()
        );
        debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        for mut lemma in derived_clauses {
            lemma.sort_by_key(|lit| lit.var().0);
            if algorithm.derived_clauses.insert(lemma.clone()) {
                if let Some(f) = &mut file_derived_clauses {
                    for lit in lemma.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }
                algorithm.solver.add_learnt(&lemma);
            }
        }

        // ------------------------------------------------------------------------

        let pb = ProgressBar::new(cubes_product.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta})")?
                .progress_chars("#>-"),
        );
        cubes_product.retain(|cube| {
            pb.inc(1);
            let res = algorithm.solver.propcheck(cube);
            // if res {
            //     // debug!("UNKNOWN {} via UP", DisplaySlice(cube));
            // } else {
            //     // debug!("UNSAT {} via UP", DisplaySlice(cube));
            // }
            res
        });
        pb.finish_and_clear();

        info!("Size of product after third filtering: {}", cubes_product.len());
        if let Some(f) = &mut file_results {
            writeln!(f, "{},after3,{}", run_number, cubes_product.len())?;
        }

        if cubes_product.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);
            break;
        }
        if cubes_product.len() == 1 {
            info!("Adding {} units to the solver", cubes_product[0].len());
            for &lit in &cubes_product[0] {
                if let Some(f) = &mut file_derived_clauses {
                    writeln!(f, "{} 0", lit)?;
                }
                algorithm.solver.add_clause(&[lit]);
            }
            cubes_product = vec![vec![]];
            continue;
        }
    }

    let elapsed = Instant::now() - start_time;
    println!("\nAll done in {:.3} s", elapsed.as_secs_f64());
    Ok(())
}
