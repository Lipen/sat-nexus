use std::collections::HashSet;
use std::ffi::CString;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use backdoor::derivation::derive_clauses;
use backdoor::utils::parse_multiple_comma_separated_intervals;
use backdoor::utils::parse_multiple_comma_separated_intervals_from;
use backdoor::utils::{concat_cubes, partition_tasks};

use clap::Parser;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use itertools::{iproduct, Itertools};
use log::{debug, info};
// use rand::prelude::*;

use cadical_sys::statik::*;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::trie::Trie;
use simple_sat::utils::DisplaySlice;
use simple_sat::var::Var;

// Run this example:
// cargo run -p backdoor --bin derive -- data/mult/lec_CvK_12.cnf --backdoors 1994,2915,3557,3695,3912,4273,4383,4475/2095,2734,3905,3912,3977,4090,4158,4260 -o derived_clauses.txt
// cargo run -p backdoor --bin derive -- data/mult/lec_CvK_12.cnf --backdoors @backdoors.csv -o derived_clauses.txt
//
// Clear the output of 'search':
// rg '\[((?:\d+)(?:,\s*\d+)*)\]' backdoors.txt -or '$1' | sed 's/ //g' > backdoors.csv

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// Input file with CNF in DIMACS format.
    #[arg(value_name = "CNF")]
    path_cnf: PathBuf,

    /// Backdoor(s).
    /// Format: either '@filename' for a file with comma-separated list of variables (1,2,3\n4,5,6)
    /// on each line or slash-separated list of comma-separated lists of variables (1,2,3/4,5,6).
    #[arg(short, long, value_name = "INT...|@FILE")]
    backdoors: String,

    /// Path to a file with derived clauses.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    path_output: Option<PathBuf>,

    /// Path to a file with results (in CSV format).
    #[arg(long = "results", value_name = "FILE")]
    path_results: Option<PathBuf>,

    /// Number of conflicts.
    #[arg(long, value_name = "INT", default_value_t = 1000)]
    num_conflicts: usize,

    /// Maximum product size.
    #[arg(long = "max-size", value_name = "INT", default_value_t = 10_000_000)]
    max_product_size: usize,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,simple_sat::solver=info,backdoor::derivation=info"))
        .init();

    let start_time = Instant::now();
    let args = Cli::parse();
    debug!("args = {:?}", args);

    let backdoors = if args.backdoors.starts_with('@') {
        parse_multiple_comma_separated_intervals_from(&args.backdoors[1..])
    } else {
        parse_multiple_comma_separated_intervals(&args.backdoors)
    };
    let mut backdoors: Vec<Vec<Var>> = backdoors
        .into_iter()
        .map(|bd| bd.into_iter().map(|i| Var::from_external(i as u32)).collect())
        .collect();
    info!("Total backdoors: {}", backdoors.len());
    for backdoor in backdoors.iter() {
        debug!("backdoor = {}", DisplaySlice(backdoor));
    }

    if backdoors.len() == 1 {
        one_backdoor(backdoors.remove(0), &args)?;
    } else {
        many_backdoors(backdoors, &args)?;
    }

    let total_time = start_time.elapsed();
    println!("\nAll done in {:.3} s", total_time.as_secs_f64());
    Ok(())
}

fn one_backdoor(backdoor: Vec<Var>, args: &Cli) -> color_eyre::Result<()> {
    // Initialize the SAT solver:
    let mut solver = Solver::default();
    solver.init_from_file(&args.path_cnf);

    // Create and open the file with derived clauses:
    let mut file_derived_clauses = if let Some(path) = &args.path_output {
        let f = File::create(path)?;
        let f = LineWriter::new(f);
        Some(f)
    } else {
        None
    };

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

    // Dump derived clauses:
    if let Some(f) = &mut file_derived_clauses {
        for lemma in derived_clauses.iter() {
            for lit in lemma.iter() {
                write!(f, "{} ", lit)?;
            }
            writeln!(f, "0")?;
        }
    }

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

    Ok(())
}

fn many_backdoors(backdoors: Vec<Vec<Var>>, args: &Cli) -> color_eyre::Result<()> {
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

    // Create and open the file with derived clauses:
    let mut file_derived_clauses = if let Some(path) = &args.path_output {
        let f = File::create(path)?;
        let f = LineWriter::new(f);
        Some(f)
    } else {
        None
    };

    // Create and open the file with results:
    // let mut file_results = if let Some(path) = &args.path_results {
    //     let f = File::create(path)?;
    //     let f = LineWriter::new(f);
    //     Some(f)
    // } else {
    //     None
    // };

    // TODO: write some CSV header here
    // if let Some(f) = &mut file_results {
    //     // writeln!(f, "...")?;
    // }

    // Cumulative Cartesian product of hard tasks (cubes):
    let mut cubes_product: Vec<Vec<Lit>> = vec![vec![]];

    // Set of all derived clauses:
    let mut all_derived_clauses: HashSet<Vec<Lit>> = HashSet::new();

    // Random number generator:
    // let mut rng = StdRng::seed_from_u64(42);

    for (i, backdoor) in backdoors.iter().enumerate() {
        info!("Run {} / {}", i + 1, backdoors.len());

        let (hard, easy) = partition_tasks(backdoor, &mut solver);
        debug!(
            "Backdoor {} has {} hard and {} easy tasks",
            DisplaySlice(backdoor),
            hard.len(),
            easy.len()
        );

        if hard.is_empty() {
            info!("No more cubes to solve after {} backdoors", i + 1);
            break;
        }
        if hard.len() == 1 {
            info!("Adding {} units to the solver", hard[0].len());
            for &lit in &hard[0] {
                if all_derived_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    solver.add_learnt(&[lit]);
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
        // if hard.len() <= 30 {
        //     for cube in hard.iter() {
        //         debug!("cube = {}", DisplaySlice(&cube));
        //     }
        // }
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
            if all_derived_clauses.insert(lemma.clone()) {
                if let Some(f) = &mut file_derived_clauses {
                    for lit in lemma.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }
                solver.add_learnt(&lemma);
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

        if cubes_product.len() * hard.len() > args.max_product_size {
            info!(
                "Reached maximum product size: {} > {}",
                cubes_product.len() * hard.len(),
                args.max_product_size
            );
            break;
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
        solver.propcheck_all_trie(&variables, &trie, &mut valid);
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
        // TODO: write some results/stats in CSV
        // if let Some(f) = &mut file_results {
        //     // writeln!(f, "...")?;
        // }

        info!("Filtering {} hard cubes via solver...", cubes_product.len());
        let time_filter = Instant::now();
        // let cubes_product_set: HashSet<Vec<Lit>> = cubes_product.iter().cloned().collect();
        // let hard_neighbors: HashMap<Vec<Lit>, usize> = cubes_product
        //     .iter()
        //     .choose_multiple(&mut rng, 50000)
        //     .into_iter()
        //     .map(|cube| {
        //         let mut s = 0;
        //         for i in 0..cube.len() {
        //             let mut other = cube.clone();
        //             other[i] = !other[i];
        //             if cubes_product_set.contains(&other) {
        //                 s += 1;
        //             }
        //         }
        //         // info!("cube {} has {} hard and {} easy neighbors", DisplaySlice(&cube), s, cube.len() - s);
        //         (cube.clone(), s)
        //     })
        //     .collect();
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

            // if let Some(&num_hard_neighbors) = hard_neighbors.get(cube) {
            //     if num_hard_neighbors > 3 {
            //         // pb.println(format!(
            //         //     "skipping cube with {} hard neighbors: {}",
            //         //     num_hard_neighbors
            //         //     DisplaySlice(cube),
            //         // ));
            //         return true;
            //     }
            // } else {
            //     return true;
            // }

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
        // TODO: write some results/stats in CSV
        // if let Some(f) = &mut file_results {
        //     writeln!(f, "{},limited,{}", i, cubes_product.len())?;
        // }

        if cubes_product.is_empty() {
            info!("No more cubes to solve after {} backdoors", i + 1);
            break;
        }
        if cubes_product.len() == 1 {
            info!("Adding {} units to the solver", cubes_product[0].len());
            for &lit in &cubes_product[0] {
                if all_derived_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    solver.add_learnt(&[lit]);
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
        let time_derive = time_derive.elapsed();
        info!(
            "Total {} derived clauses ({} units, {} binary, {} other) in {:.1}s",
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
            if all_derived_clauses.insert(lemma.clone()) {
                if let Some(f) = &mut file_derived_clauses {
                    for lit in lemma.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }
                solver.add_learnt(&lemma);
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
    }

    info!(
        "Overall, derived {} clauses ({} units, {} binary, {} other)",
        all_derived_clauses.len(),
        all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
        all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
        all_derived_clauses.iter().filter(|c| c.len() > 2).count()
    );

    Ok(())
}
