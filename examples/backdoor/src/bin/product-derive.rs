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
use backdoor::derivation::derive_clauses;
use backdoor::utils::{concat_cubes, parse_comma_separated_intervals, partition_tasks};
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
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
        writeln!(f, "i,retain,size")?;
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
            "Total {} derived clauses ({} units, {} binary, {} other) after retain",
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

        info!("Size of product before retain: {}", cubes_product.len());
        if let Some(f) = &mut file_results {
            writeln!(f, "{},before,{}", run_number, cubes_product.len())?;
        }

        // debug!("Deriving clauses for {} cubes...", cubes_product.len());
        // let derived_clauses = derive_clauses(&cubes_product);
        // debug!(
        //     "Total {} derived clauses ({} units, {} binary, {} other) BEFORE RETAIN",
        //     derived_clauses.len(),
        //     derived_clauses.iter().filter(|c| c.len() == 1).count(),
        //     derived_clauses.iter().filter(|c| c.len() == 2).count(),
        //     derived_clauses.iter().filter(|c| c.len() > 2).count()
        // );
        // debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

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

        info!("Size of product after retain: {}", cubes_product.len());
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
            "Total {} derived clauses ({} units, {} binary, {} other) after retain",
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

        info!("Size of product after second retain: {}", cubes_product.len());
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
            "Total {} derived clauses ({} units, {} binary, {} other) after second retain",
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

        info!("Size of product after third retain: {}", cubes_product.len());
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

// [2023-12-09T17:02:42Z INFO  x1] Size of product before retain: 24
// [2023-12-09T17:02:42Z DEBUG x1] Total 42 derived clauses (2 units, 40 binary) BEFORE RETAIN
// [2023-12-09T17:02:42Z INFO  x1] Size of product after retain: 24
// [2023-12-09T17:02:42Z DEBUG x1] Total 42 derived clauses (2 units, 40 binary)
// [2023-12-09T17:02:42Z INFO  x1] Size of product after second retain: 9
//
// [2023-12-09T17:02:42Z INFO  x1] Size of product before retain: 90
// [2023-12-09T17:02:42Z DEBUG x1] Total 41 derived clauses (10 units, 31 binary) BEFORE RETAIN
// [2023-12-09T17:02:42Z INFO  x1] Size of product after retain: 60
// [2023-12-09T17:02:42Z DEBUG x1] Total 51 derived clauses (10 units, 41 binary)
// [2023-12-09T17:02:42Z INFO  x1] Size of product after second retain: 43
//
// [2023-12-09T17:02:43Z INFO  x1] Size of product before retain: 774
// [2023-12-09T17:02:43Z DEBUG x1] Total 83 derived clauses (13 units, 70 binary) BEFORE RETAIN
// [2023-12-09T17:02:43Z INFO  x1] Size of product after retain: 345
// [2023-12-09T17:02:43Z DEBUG x1] Total 105 derived clauses (13 units, 92 binary)
// [2023-12-09T17:02:43Z INFO  x1] Size of product after second retain: 345
//
// [2023-12-09T17:02:44Z INFO  x1] Size of product before retain: 4830
// [2023-12-09T17:02:44Z DEBUG x1] Total 124 derived clauses (16 units, 108 binary) BEFORE RETAIN
// [2023-12-09T17:02:44Z INFO  x1] Size of product after retain: 880
// [2023-12-09T17:02:44Z DEBUG x1] Total 76 derived clauses (22 units, 54 binary)
// [2023-12-09T17:02:44Z INFO  x1] Size of product after second retain: 880
//
//
//
//
//
// [2023-12-09T17:02:46Z INFO  x1] Size of product before retain: 18102
// [2023-12-09T17:02:46Z DEBUG x1] Total 118 derived clauses (34 units, 84 binary) BEFORE RETAIN
// [2023-12-09T17:02:47Z INFO  x1] Size of product after retain: 14098
// [2023-12-09T17:02:48Z DEBUG x1] Total 122 derived clauses (34 units, 88 binary)
// [2023-12-09T17:02:48Z INFO  x1] Size of product after second retain: 14098
//
// [2023-12-09T17:02:49Z INFO  x1] Size of product before retain: 126882
// [2023-12-09T17:02:55Z DEBUG x1] Total 139 derived clauses (39 units, 100 binary) BEFORE RETAIN
// [2023-12-09T17:03:01Z INFO  x1] Size of product after retain: 42456
// [2023-12-09T17:03:03Z DEBUG x1] Total 140 derived clauses (40 units, 100 binary)
// [2023-12-09T17:03:05Z INFO  x1] Size of product after second retain: 42456
//
// [2023-12-09T17:03:07Z INFO  x1] Size of product before retain: 382104
// [2023-12-09T17:03:31Z DEBUG x1] Total 170 derived clauses (44 units, 126 binary) BEFORE RETAIN
// [2023-12-09T17:03:50Z INFO  x1] Size of product after retain: 55909
// [2023-12-09T17:03:54Z DEBUG x1] Total 218 derived clauses (44 units, 174 binary)
// [2023-12-09T17:03:57Z INFO  x1] Size of product after second retain: 55909
//
// [2023-12-09T17:03:59Z INFO  x1] Size of product before retain: 447272
// [2023-12-09T17:04:37Z DEBUG x1] Total 228 derived clauses (50 units, 178 binary) BEFORE RETAIN
// [2023-12-09T17:05:03Z INFO  x1] Size of product after retain: 79204
// [2023-12-09T17:05:10Z DEBUG x1] Total 248 derived clauses (50 units, 198 binary)
// [2023-12-09T17:05:15Z INFO  x1] Size of product after second retain: 79204
//
// [2023-12-09T17:05:18Z INFO  x1] Size of product before retain: 712836
// [2023-12-09T17:06:29Z DEBUG x1] Total 265 derived clauses (53 units, 212 binary) BEFORE RETAIN
// [2023-12-09T17:07:08Z INFO  x1] Size of product after retain: 110050
// [2023-12-09T17:07:19Z DEBUG x1] Total 397 derived clauses (53 units, 344 binary)
// [2023-12-09T17:07:27Z INFO  x1] Size of product after second retain: 110050
