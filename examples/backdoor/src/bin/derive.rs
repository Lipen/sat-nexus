use std::collections::HashSet;
use std::ffi::CString;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use itertools::{iproduct, Itertools};
use log::{debug, info};
use rand::prelude::*;

use backdoor::derivation::derive_clauses;
use backdoor::utils::parse_multiple_comma_separated_intervals;
use backdoor::utils::parse_multiple_comma_separated_intervals_from;
use backdoor::utils::{concat_cubes, create_line_writer, partition_tasks};

use cadical_sys::statik::*;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::trie::Trie;
use simple_sat::utils::{parse_dimacs, DisplaySlice};
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
    #[arg(short, long, value_name = "INT...")]
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
    let mut file_derived_clauses = args.path_output.as_ref().map(create_line_writer);

    // Create and open the file with results:
    let mut file_results = args.path_results.as_ref().map(create_line_writer);

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
        for clause in parse_dimacs(&args.path_cnf) {
            for lit in clause {
                ccadical_add(solver_full, lit.to_external());
            }
            ccadical_add(solver_full, 0);
        }
    }

    // Create and open the file with derived clauses:
    let mut file_derived_clauses = args.path_output.as_ref().map(create_line_writer);

    // Create and open the file with results:
    let mut file_results = args.path_results.as_ref().map(create_line_writer);

    // TODO: write some CSV header here
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

    // Cumulative Cartesian product of hard tasks (cubes):
    let mut cubes_product: Vec<Vec<Lit>> = vec![vec![]];

    // Random number generator:
    let mut rng = StdRng::seed_from_u64(42);

    let time_runs = Instant::now();

    for (run_number, backdoor) in backdoors.iter().enumerate() {
        let run_number = run_number + 1; // 1-based
        info!("Run {} / {}", run_number, backdoors.len());

        let (hard, easy) = partition_tasks(backdoor, &mut solver);
        debug!(
            "Backdoor {} has {} hard and {} easy tasks",
            DisplaySlice(backdoor),
            hard.len(),
            easy.len()
        );

        if hard.is_empty() {
            info!("No more cubes to solve after {} backdoors", run_number);
            break;
        }
        if hard.len() == 1 {
            info!("Adding {} units to the solver", hard[0].len());
            for &lit in &hard[0] {
                if all_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    solver.add_learnt(&[lit]);
                    unsafe {
                        ccadical_add(solver_full, lit.to_external());
                        ccadical_add(solver_full, 0);
                    }
                    all_derived_clauses.push(vec![lit]);
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
            "Derived {} clauses ({} units, {} binary, {} other) for backdoor in {:.1}s",
            derived_clauses.len(),
            derived_clauses.iter().filter(|c| c.len() == 1).count(),
            derived_clauses.iter().filter(|c| c.len() == 2).count(),
            derived_clauses.iter().filter(|c| c.len() > 2).count(),
            time_derive.elapsed().as_secs_f64()
        );
        // debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        let mut new_derived_clauses = Vec::new();
        for mut clause in derived_clauses {
            clause.sort_by_key(|lit| lit.inner());
            if all_clauses.insert(clause.clone()) {
                if let Some(f) = &mut file_derived_clauses {
                    for lit in clause.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }
                solver.add_learnt(&clause);
                unsafe {
                    for lit in clause.iter() {
                        ccadical_add(solver_full, lit.to_external());
                    }
                    ccadical_add(solver_full, 0);
                }
                new_derived_clauses.push(clause.clone());
                all_derived_clauses.push(clause);
            }
        }
        debug!(
            "Derived {} new clauses ({} units, {} binary, {} other) for backdoor",
            new_derived_clauses.len(),
            new_derived_clauses.iter().filter(|c| c.len() == 1).count(),
            new_derived_clauses.iter().filter(|c| c.len() == 2).count(),
            new_derived_clauses.iter().filter(|c| c.len() > 2).count()
        );
        // debug!("[{}]", new_derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        // ------------------------------------------------------------------------

        debug!(
            "Going to produce a product of size {} * {} = {}",
            cubes_product.len(),
            hard.len(),
            cubes_product.len() * hard.len()
        );
        if let Some(f) = &mut file_results {
            writeln!(f, "{},product,{}", run_number, cubes_product.len() * hard.len())?;
        }

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
        let mut num_normal_cubes = 0u64;
        'out: for (old, new) in iproduct!(cubes_product, hard).progress_with(pb) {
            let cube = concat_cubes(old, new);
            for i in 1..cube.len() {
                if cube[i] == -cube[i - 1] {
                    // Skip the cube with inconsistent literals:
                    continue 'out;
                }
            }
            // assert!(std::iter::zip(&cube, &variables).all(|(lit, var)| lit.var() == *var));
            trie.insert(cube.iter().map(|lit| lit.negated()));
            num_normal_cubes += 1;
        }
        let time_trie_construct = time_trie_construct.elapsed();
        info!(
            "Trie of size {} with {} leaves constructed out of {} cubes in {:.1}s",
            trie.len(),
            trie.num_leaves(),
            num_normal_cubes,
            time_trie_construct.as_secs_f64()
        );
        if let Some(f) = &mut file_results {
            writeln!(f, "{},concat,{}", run_number, trie.num_leaves())?;
        }

        info!("Filtering {} hard cubes via trie...", trie.num_leaves());
        let time_filter = Instant::now();
        let mut valid = Vec::new();
        solver.propcheck_all_trie(&variables, &trie, &mut valid);
        drop(trie);
        cubes_product = valid;
        let time_filter = time_filter.elapsed();
        info!(
            "Filtered down to {} cubes in {:.1}s",
            cubes_product.len(),
            time_filter.as_secs_f64()
        );
        if let Some(f) = &mut file_results {
            writeln!(f, "{},propagate,{}", run_number, cubes_product.len())?;
        }

        info!("Filtering {} hard cubes via solver...", cubes_product.len());
        let time_filter = Instant::now();
        let c = CString::new("conflicts").expect("CString::new failed");
        let pb = ProgressBar::new(cubes_product.len() as u64);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")?
                .progress_chars("#>-"),
        );
        pb.set_message("filtering");
        cubes_product.retain(|cube| {
            pb.inc(1);

            if rng.gen_bool(0.9) {
                return true;
            }

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
                        let model = (1..=ccadical_vars(solver_full))
                            .map(|i| ccadical_val(solver_full, i as i32))
                            .collect::<Vec<_>>();
                        {
                            let f = File::create("model.txt").unwrap();
                            let mut f = BufWriter::new(f);
                            writeln!(f, "{}", model.iter().join(" ")).unwrap();
                        }
                        {
                            let f = File::create("model.cnf").unwrap();
                            let mut f = BufWriter::new(f);
                            for lit in model.iter() {
                                writeln!(f, "{} 0", lit).unwrap();
                            }
                        }
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
            info!("No more cubes to solve after {} backdoors", run_number);
            break;
        }
        if cubes_product.len() == 1 {
            info!("Adding {} units to the solver", cubes_product[0].len());
            for &lit in &cubes_product[0] {
                if all_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    solver.add_learnt(&[lit]);
                    unsafe {
                        ccadical_add(solver_full, lit.to_external());
                        ccadical_add(solver_full, 0);
                    }
                    all_derived_clauses.push(vec![lit]);
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
            "Derived {} clauses ({} units, {} binary, {} other) in {:.1}s",
            derived_clauses.len(),
            derived_clauses.iter().filter(|c| c.len() == 1).count(),
            derived_clauses.iter().filter(|c| c.len() == 2).count(),
            derived_clauses.iter().filter(|c| c.len() > 2).count(),
            time_derive.as_secs_f64()
        );
        // debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        let mut new_derived_clauses = Vec::new();
        for mut clause in derived_clauses {
            clause.sort_by_key(|lit| lit.inner());
            if all_clauses.insert(clause.clone()) {
                if let Some(f) = &mut file_derived_clauses {
                    for lit in clause.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }
                solver.add_learnt(&clause);
                unsafe {
                    for lit in clause.iter() {
                        ccadical_add(solver_full, lit.to_external());
                    }
                    ccadical_add(solver_full, 0);
                }
                new_derived_clauses.push(clause.clone());
                all_derived_clauses.push(clause);
            }
        }
        info!(
            "Derived {} NEW clauses ({} units, {} binary, {} other)",
            new_derived_clauses.len(),
            new_derived_clauses.iter().filter(|c| c.len() == 1).count(),
            new_derived_clauses.iter().filter(|c| c.len() == 2).count(),
            new_derived_clauses.iter().filter(|c| c.len() > 2).count()
        );
        // debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        info!(
            "So far derived {} new clauses ({} units, {} binary, {} other)",
            all_derived_clauses.len(),
            all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
            all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
            all_derived_clauses.iter().filter(|c| c.len() > 2).count()
        )
    }

    let time_runs = time_runs.elapsed();
    info!("Finished {} runs in {:.1}s", backdoors.len(), time_runs.as_secs_f64());
    info!(
        "Total derived {} new clauses ({} units, {} binary, {} other)",
        all_derived_clauses.len(),
        all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
        all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
        all_derived_clauses.iter().filter(|c| c.len() > 2).count()
    );

    Ok(())
}
