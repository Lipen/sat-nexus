use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::fs::File;
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use itertools::{iproduct, Itertools};
use log::{debug, info, trace};
use rand::prelude::*;

use backdoor::algorithm::{Algorithm, Options, DEFAULT_OPTIONS};
use backdoor::derivation::derive_clauses;
use backdoor::utils::{concat_cubes, parse_comma_separated_intervals, partition_tasks};
use cadical_sys::statik::*;
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

    let solver_full = unsafe { ccadical_init() };
    unsafe {
        for clause in solver.clauses_iter() {
            for lit in clause.lits() {
                ccadical_add(solver_full, lit.to_external());
            }
            ccadical_add(solver_full, 0)
        }
    }

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
                    algorithm.solver.add_learnt(&[lit]);
                    unsafe {
                        ccadical_add(solver_full, lit.to_external());
                        ccadical_add(solver_full, 0);
                    }
                }
            }
            continue;
        }

        // ------------------------------------------------------------------------

        info!("Deriving clauses for {} cubes...", hard.len());
        let time_derive = Instant::now();
        if hard.len() <= 30 {
            for cube in hard.iter() {
                debug!("cube = {}", DisplaySlice(&cube));
            }
        }
        let derived_clauses = derive_clauses(&hard);
        info!(
            "Total {} derived clauses ({} units, {} binary, {} other) for backdoor in {:.1}s",
            derived_clauses.len(),
            derived_clauses.iter().filter(|c| c.len() == 1).count(),
            derived_clauses.iter().filter(|c| c.len() == 2).count(),
            derived_clauses.iter().filter(|c| c.len() > 2).count(),
            time_derive.elapsed().as_secs_f64()
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
                unsafe {
                    for lit in lemma.iter() {
                        ccadical_add(solver_full, lit.to_external());
                    }
                    ccadical_add(solver_full, 0);
                }
                new_clauses.push(lemma);
            }
        }
        debug!(
            "NEW {} derived clauses ({} units, {} binary, {} other)",
            new_clauses.len(),
            new_clauses.iter().filter(|c| c.len() == 1).count(),
            new_clauses.iter().filter(|c| c.len() == 2).count(),
            new_clauses.iter().filter(|c| c.len() > 2).count()
        );
        // debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        // ------------------------------------------------------------------------

        // info!(
        //     "Going to produce a product of size {} * {} = {}",
        //     cubes_product.len(),
        //     hard.len(),
        //     cubes_product.len() * hard.len()
        // );
        // cubes_product = cubes_product
        //     .into_iter()
        //     .cartesian_product(hard)
        //     .map(|(a, b)| concat_cubes(a, b))
        //     .collect_vec();
        // info!("Size of product before retain: {}", cubes_product.len());
        // if let Some(f) = &mut file_results {
        //     writeln!(f, "{},before,{}", run_number, cubes_product.len())?;
        // }

        debug!(
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

        info!("Constructing trie out of {} cubes...", cubes_product.len() * hard.len());
        let time_trie_construct = Instant::now();
        let mut trie = Trie::new();
        let pb = ProgressBar::new((cubes_product.len() * hard.len()) as u64);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")?
                .progress_chars("#>-"),
        );
        pb.set_message("trie construction");
        // let mut count: u64 = 0; // !
        let mut num_normal_cubes = 0u64;
        'out: for (old, new) in iproduct!(cubes_product, hard).progress_with(pb) {
            let cube = concat_cubes(old, new);
            for i in 1..cube.len() {
                if cube[i] == -cube[i - 1] {
                    // Skip the cube with inconsistent literals:
                    // log::warn!("Skipping the concatenated cube {} with inconsistent literals", DisplaySlice(&cube));
                    continue 'out;
                }
            }
            // if algorithm.solver.propcheck(&cube) {
            //     count += 1;
            // }
            // assert!(std::iter::zip(&cube, &variables).all(|(lit, var)| lit.var() == *var));
            trie.insert(cube.iter().map(|lit| lit.negated()));
            num_normal_cubes += 1;
        }
        info!(
            "Trie of size {} with {} leaves constructed out of {} cubes in {:.1}s",
            trie.len(),
            trie.num_leaves(),
            num_normal_cubes,
            time_trie_construct.elapsed().as_secs_f64()
        );

        info!("Filtering {} hard cubes via trie...", trie.num_leaves());
        let time_filter = Instant::now();
        let mut valid = Vec::new();
        algorithm.solver.propcheck_all_trie(&variables, &trie, &mut valid);
        // if valid.len() as u64 != count {
        //     log::error!("Mismatch: trie->{}, propcheck->{}", valid.len(), count);
        // }
        // assert_eq!(valid.len() as u64, count); // !
        // drop(trie);
        cubes_product = valid;
        info!(
            "Filtered down to {} cubes in {:.1}s",
            cubes_product.len(),
            time_filter.elapsed().as_secs_f64()
        );
        if let Some(f) = &mut file_results {
            writeln!(f, "{},propagate,{}", run_number, cubes_product.len())?;
        }

        info!("Filtering {} hard cubes via solver...", cubes_product.len());
        let time_filter = Instant::now();
        let cubes_product_set: HashSet<Vec<Lit>> = cubes_product.iter().cloned().collect();
        let hard_neighbors: HashMap<Vec<Lit>, usize> = cubes_product
            .iter()
            .choose_multiple(&mut algorithm.rng, 50000)
            .into_iter()
            .map(|cube| {
                let mut s = 0;
                for i in 0..cube.len() {
                    let mut other = cube.clone();
                    other[i] = !other[i];
                    if cubes_product_set.contains(&other) {
                        s += 1;
                    }
                }
                // info!("cube {} has {} hard and {} easy neighbors", DisplaySlice(&cube), s, cube.len() - s);
                (cube.clone(), s)
            })
            .collect();
        // debug!("hard neighbor counts: {:?}", hard_neighbors.values().sorted());
        let c = CString::new("conflicts").expect("CString::new failed");
        let pb = ProgressBar::new(cubes_product.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")?
                .progress_chars("#>-"),
        );
        pb.set_message("filtering");
        cubes_product.retain(|cube| {
            pb.inc(1);

            if let Some(&num_hard_neighbors) = hard_neighbors.get(cube) {
                if num_hard_neighbors > 3 {
                    // pb.println(format!(
                    //     "skipping cube with {} hard neighbors: {}",
                    //     num_hard_neighbors
                    //     DisplaySlice(cube),
                    // ));
                    return true;
                }
            } else {
                return true;
            }

            // let res = algorithm.solver.propcheck(cube);
            // // if res {
            // //     // debug!("UNKNOWN {} via UP", DisplaySlice(cube));
            // // } else {
            // //     // debug!("UNSAT {} via UP", DisplaySlice(cube));
            // // }
            // res

            unsafe {
                // debug!("cube = {}", DisplaySlice(cube));
                for &lit in cube.iter() {
                    ccadical_assume(solver_full, lit.to_external());
                }
                ccadical_limit(solver_full, c.as_ptr(), args.num_conflicts as i32);
                match ccadical_solve(solver_full) {
                    0 => {
                        // UNKNOWN
                        true
                    }
                    10 => {
                        // SAT
                        panic!("unexpected SAT");
                        // false
                    }
                    20 => {
                        // UNSAT
                        false
                    }
                    r => panic!("Unexpected result: {}", r),
                }
            }
        });
        pb.finish_and_clear();
        drop(trie);
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
                if algorithm.derived_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    algorithm.solver.add_learnt(&[lit]);
                    unsafe {
                        ccadical_add(solver_full, lit.to_external());
                        ccadical_add(solver_full, 0);
                    }
                }
            }
            cubes_product = vec![vec![]];
            continue;
        }

        // ------------------------------------------------------------------------

        info!("Deriving clauses for {} cubes...", cubes_product.len());
        let time_derive = Instant::now();
        let derived_clauses = derive_clauses(&cubes_product);
        info!(
            "Total {} derived clauses ({} units, {} binary, {} other) in {:.1}s",
            derived_clauses.len(),
            derived_clauses.iter().filter(|c| c.len() == 1).count(),
            derived_clauses.iter().filter(|c| c.len() == 2).count(),
            derived_clauses.iter().filter(|c| c.len() > 2).count(),
            time_derive.elapsed().as_secs_f64()
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
                unsafe {
                    for lit in lemma.iter() {
                        ccadical_add(solver_full, lit.to_external());
                    }
                    ccadical_add(solver_full, 0);
                }
                new_clauses.push(lemma);
            }
        }
        debug!(
            "NEW {} derived clauses ({} units, {} binary, {} other)",
            new_clauses.len(),
            new_clauses.iter().filter(|c| c.len() == 1).count(),
            new_clauses.iter().filter(|c| c.len() == 2).count(),
            new_clauses.iter().filter(|c| c.len() > 2).count()
        );
        // debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        // ------------------------------------------------------------------------

        // ===
        continue;
        // ===

        // let pb = ProgressBar::new(cubes_product.len() as u64);
        // pb.set_style(
        //     ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta})")?
        //         .progress_chars("#>-"),
        // );
        // cubes_product.retain(|cube| {
        //     pb.inc(1);
        //     let res = algorithm.solver.propcheck(cube);
        //     // if res {
        //     //     // debug!("UNKNOWN {} via UP", DisplaySlice(cube));
        //     // } else {
        //     //     // debug!("UNSAT {} via UP", DisplaySlice(cube));
        //     // }
        //     res
        // });
        // pb.finish_and_clear();
        //
        // info!("Size of product after second filtering: {}", cubes_product.len());
        // if let Some(f) = &mut file_results {
        //     writeln!(f, "{},after2,{}", run_number, cubes_product.len())?;
        // }
        //
        // if cubes_product.is_empty() {
        //     info!("No more cubes to solve after {} runs", run_number);
        //     break;
        // }
        // if cubes_product.len() == 1 {
        //     info!("Adding {} units to the solver", cubes_product[0].len());
        //     for &lit in &cubes_product[0] {
        //         if algorithm.derived_clauses.insert(vec![lit]) {
        //             if let Some(f) = &mut file_derived_clauses {
        //                 writeln!(f, "{} 0", lit)?;
        //             }
        //             algorithm.solver.add_learnt(&[lit]);
        //         }
        //     }
        //     cubes_product = vec![vec![]];
        //     continue;
        // }
        //
        // // ------------------------------------------------------------------------
        //
        // debug!("Deriving clauses for {} cubes...", cubes_product.len());
        // let derived_clauses = derive_clauses(&cubes_product);
        // debug!(
        //     "Total {} derived clauses ({} units, {} binary, {} other) after second filtering",
        //     derived_clauses.len(),
        //     derived_clauses.iter().filter(|c| c.len() == 1).count(),
        //     derived_clauses.iter().filter(|c| c.len() == 2).count(),
        //     derived_clauses.iter().filter(|c| c.len() > 2).count()
        // );
        // debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));
        //
        // for mut lemma in derived_clauses {
        //     lemma.sort_by_key(|lit| lit.0);
        //     if algorithm.derived_clauses.insert(lemma.clone()) {
        //         if let Some(f) = &mut file_derived_clauses {
        //             for lit in lemma.iter() {
        //                 write!(f, "{} ", lit)?;
        //             }
        //             writeln!(f, "0")?;
        //         }
        //         algorithm.solver.add_learnt(&lemma);
        //     }
        // }
        //
        // // ------------------------------------------------------------------------
        //
        // let pb = ProgressBar::new(cubes_product.len() as u64);
        // pb.set_style(
        //     ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta})")?
        //         .progress_chars("#>-"),
        // );
        // cubes_product.retain(|cube| {
        //     pb.inc(1);
        //     let res = algorithm.solver.propcheck(cube);
        //     // if res {
        //     //     // debug!("UNKNOWN {} via UP", DisplaySlice(cube));
        //     // } else {
        //     //     // debug!("UNSAT {} via UP", DisplaySlice(cube));
        //     // }
        //     res
        // });
        // pb.finish_and_clear();
        //
        // info!("Size of product after third filtering: {}", cubes_product.len());
        // if let Some(f) = &mut file_results {
        //     writeln!(f, "{},after3,{}", run_number, cubes_product.len())?;
        // }
        //
        // if cubes_product.is_empty() {
        //     info!("No more cubes to solve after {} runs", run_number);
        //     break;
        // }
        // if cubes_product.len() == 1 {
        //     info!("Adding {} units to the solver", cubes_product[0].len());
        //     for &lit in &cubes_product[0] {
        //         if let Some(f) = &mut file_derived_clauses {
        //             writeln!(f, "{} 0", lit)?;
        //         }
        //         algorithm.solver.add_learnt(&[lit]);
        //     }
        //     cubes_product = vec![vec![]];
        //     continue;
        // }
    }

    println!("\nAll done in {:.3} s", start_time.elapsed().as_secs_f64());
    Ok(())
}
