use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use itertools::{iproduct, zip_eq, Itertools};
use log::{debug, info};
use ordered_float::OrderedFloat;
use rand::prelude::*;

use backdoor::algorithm::{Algorithm, Options, DEFAULT_OPTIONS};
use backdoor::derivation::derive_clauses;
use backdoor::solvers::SatSolver;
use backdoor::utils::{clause_to_external, concat_cubes, create_line_writer, determine_vars_pool, get_hard_tasks};

use cadical::statik::Cadical;
use cadical::{LitValue, SolveResponse};
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::trie::Trie;
use simple_sat::utils::{parse_dimacs, DisplaySlice};
use simple_sat::var::Var;

// Run this example:
// cargo run -p backdoor --bin interleave -- data/mult/lec_CvK_12.cnf --backdoor-size 10 --num-iters 10000

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

    /// Do not freeze variables.
    #[arg(long)]
    no_freeze: bool,

    /// Derive ternary clauses.
    #[arg(long)]
    derive_ternary: bool,

    /// Maximum product size.
    #[arg(long, value_name = "INT", default_value_t = 10_000)]
    max_product: usize,

    /// Initial budget (in conflicts) for filtering.
    #[arg(long, value_name = "INT", default_value_t = 100_000)]
    budget_filter: u64,

    /// Multiplicative factor for filtering budget.
    #[arg(long, value_name = "FLOAT", default_value_t = 1.0)]
    factor_budget_filter: f64,

    /// Initial budget (in conflicts) for solving.
    #[arg(long, value_name = "INT", default_value_t = 10_000)]
    budget_solve: u64,

    /// Multiplicative factor for solving budget.
    #[arg(long, value_name = "FLOAT", default_value_t = 1.1)]
    factor_budget_solve: f64,

    /// Use novel sorted filtering method.
    #[arg(long)]
    use_sorted_filtering: bool,

    /// Time limit (in seconds).
    #[arg(long, value_name = "INT")]
    time_limit: Option<u64>,

    /// Act as preprocessor only before delegating to Cadical.
    #[arg(long)]
    only_preprocess: bool,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,simple_sat::solver=info,backdoor::derivation=info"))
        .init();

    let start_time = Instant::now();
    let args = Cli::parse();
    info!("args = {:?}", args);

    if args.only_preprocess {
        assert!(
            args.time_limit.is_some(),
            "Whe using '--only-preprocess', option '--time-limit <INT>' must be also specified"
        );
    }

    // Initialize SAT solver:
    let mut mysolver = Solver::default();
    mysolver.init_from_file(&args.path_cnf);
    mysolver.propagate();
    mysolver.simplify();

    // Initialize Cadical:
    let solver = Cadical::new();
    for clause in parse_dimacs(&args.path_cnf) {
        solver.add_clause(clause.into_iter().map(|lit| lit.to_external()));
    }
    if !args.no_freeze {
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
    let mut algorithm = Algorithm::new(SatSolver::new_cadical(solver), pool, options);

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

    let mut budget_filter = args.budget_filter;
    let mut budget_solve = args.budget_solve;

    let mut run_number = 0;
    loop {
        run_number += 1;
        info!("Run {}", run_number);
        let time_run = Instant::now();

        if let Some(time_limit) = args.time_limit {
            if start_time.elapsed().as_secs_f64() > time_limit as f64 {
                info!("Time limit ({}s) reached", time_limit);
                break;
            }
        }

        let result = algorithm.run(
            args.backdoor_size,
            args.num_iters,
            args.stagnation_limit,
            Some(((1u64 << args.backdoor_size) - 1) as f64 / (1u64 << args.backdoor_size) as f64),
            0,
        );
        assert!(result.best_fitness.num_hard > 0, "Found strong backdoor?!..");

        let backdoor = result.best_instance.get_variables();
        // let (hard, easy) = partition_tasks_cadical(&backdoor, &algorithm.solver);
        // debug!(
        //     "Backdoor {} has {} hard and {} easy tasks",
        //     DisplaySlice(&backdoor),
        //     hard.len(),
        //     easy.len()
        // );
        let hard = get_hard_tasks(&backdoor, &mut algorithm.solver);
        debug!("Backdoor {} has {} hard tasks", DisplaySlice(&backdoor), hard.len(),);
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
                    algorithm.solver.add_clause(&[lit]);
                    mysolver.add_clause(&[lit]);
                    all_derived_clauses.push(vec![lit]);
                }
            }
            match &mut algorithm.solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    solver.limit("conflicts", 0);
                    solver.solve()?;
                }
            }
            mysolver.propagate();
            mysolver.simplify();
            continue;
        }

        // ------------------------------------------------------------------------

        info!("Deriving clauses for {} cubes...", hard.len());
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
                    for lit in lemma.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }
                algorithm.solver.add_clause(&lemma);
                mysolver.add_clause(&lemma);
                new_clauses.push(lemma.clone());
                all_derived_clauses.push(lemma);
            }
        }
        match &mut algorithm.solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => {
                solver.limit("conflicts", 0);
                solver.solve()?;
            }
        }
        mysolver.propagate();
        mysolver.simplify();
        info!(
            "Derived {} new clauses ({} units, {} binary, {} other)",
            new_clauses.len(),
            new_clauses.iter().filter(|c| c.len() == 1).count(),
            new_clauses.iter().filter(|c| c.len() == 2).count(),
            new_clauses.iter().filter(|c| c.len() > 2).count()
        );
        // debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

        // ------------------------------------------------------------------------

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

        // let mut new_cubes_product = Vec::new();
        // 'out: for (old, new) in iproduct!(cubes_product, hard) {
        //     let cube = concat_cubes(old, new);
        //     for i in 1..cube.len() {
        //         if cube[i] == -cube[i - 1] {
        //             // Skip the cube with inconsistent literals:
        //             // log::warn!("Skipping the concatenated cube {} with inconsistent literals", DisplaySlice(&cube));
        //             continue 'out;
        //         }
        //     }
        //     new_cubes_product.push(cube);
        // }
        // cubes_product = new_cubes_product;
        // info!("Product now contains {} hard cubes", cubes_product.len());

        info!("Constructing trie out of {} potential cubes...", cubes_product.len() * hard.len());
        let time_trie_construct = Instant::now();
        let mut trie = Trie::new();
        let pb = ProgressBar::new((cubes_product.len() * hard.len()) as u64);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")?
                .progress_chars("#>-"),
        );
        pb.set_message("trie construction");
        let mut num_normal_cubes: u64 = 0;
        // let mut normal_cubes = Vec::new();
        'out: for (old, new) in iproduct!(cubes_product, hard).progress_with(pb) {
            let cube = concat_cubes(old, new);
            for i in 1..cube.len() {
                if cube[i] == -cube[i - 1] {
                    // Skip the cube with inconsistent literals:
                    // log::warn!("Skipping the concatenated cube {} with inconsistent literals", DisplaySlice(&cube));
                    continue 'out;
                }
            }
            assert_eq!(cube.len(), variables.len());
            assert!(zip_eq(&cube, &variables).all(|(lit, var)| lit.var() == *var));
            trie.insert(cube.iter().map(|lit| lit.negated()));
            num_normal_cubes += 1;
            // normal_cubes.push(cube);
        }
        // assert_eq!(normal_cubes.len() as u64, num_normal_cubes);
        let time_trie_construct = time_trie_construct.elapsed();
        info!(
            "Trie of size {} with {} leaves constructed out of {} normal cubes in {:.1}s",
            trie.len(),
            trie.num_leaves(),
            num_normal_cubes,
            time_trie_construct.as_secs_f64()
        );

        if trie.num_leaves() as u64 != num_normal_cubes {
            log::error!("Mismatch!");
            log::error!("trie.num_leaves() = {}", trie.num_leaves());
            log::error!("num_normal_cubes = {}", num_normal_cubes);

            // println!("First 10 cubes out of {}:", normal_cubes.len());
            // for cube in normal_cubes.iter().take(10) {
            //     println!("cube = {}", DisplaySlice(cube));
            // }
            println!("First 10 trie words out of {}:", trie.num_leaves());
            for cube in trie.iter().take(10) {
                let cube: Vec<Lit> = zip_eq(cube, &variables).map(|(b, &v)| Lit::new(v, b)).collect();
                println!("cube = {}", DisplaySlice(&cube));
            }
        }
        assert_eq!(trie.num_leaves() as u64, num_normal_cubes);

        // cubes_product = trie
        //     .iter()
        //     .map(|cube| zip_eq(cube, &variables).map(|(b, &v)| Lit::new(v, b)).collect())
        //     .collect();
        // drop(trie);

        info!("Filtering {} hard cubes via trie...", trie.num_leaves());
        let time_filter = Instant::now();
        let mut valid = Vec::new();
        mysolver.propcheck_all_trie(&variables, &trie, &mut valid);
        drop(trie);
        cubes_product = valid;
        info!(
            "Filtered down to {} cubes via trie in {:.1}s",
            cubes_product.len(),
            time_filter.elapsed().as_secs_f64()
        );
        if let Some(f) = &mut file_results {
            writeln!(f, "{},propagate,{}", run_number, cubes_product.len())?;
        }

        // Derivation after trie-filtering:
        {
            info!("Deriving clauses for {} cubes...", cubes_product.len());
            let time_derive = Instant::now();
            let derived_clauses = derive_clauses(&cubes_product, args.derive_ternary);
            let time_derive = time_derive.elapsed();
            info!(
                "Derived {} clauses ({} units, {} binary, {} other) for {} cubes in {:.1}s",
                derived_clauses.len(),
                derived_clauses.iter().filter(|c| c.len() == 1).count(),
                derived_clauses.iter().filter(|c| c.len() == 2).count(),
                derived_clauses.iter().filter(|c| c.len() > 2).count(),
                cubes_product.len(),
                time_derive.as_secs_f64()
            );
            // debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

            let mut new_clauses: Vec<Vec<Lit>> = Vec::new();
            for mut lemma in derived_clauses {
                lemma.sort_by_key(|lit| lit.inner());
                if all_clauses.insert(lemma.clone()) {
                    if let Some(f) = &mut file_derived_clauses {
                        for lit in lemma.iter() {
                            write!(f, "{} ", lit)?;
                        }
                        writeln!(f, "0")?;
                    }
                    algorithm.solver.add_clause(&lemma);
                    mysolver.add_clause(&lemma);
                    new_clauses.push(lemma.clone());
                    all_derived_clauses.push(lemma);
                }
            }
            match &mut algorithm.solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    solver.limit("conflicts", 0);
                    solver.solve()?;
                }
            }
            mysolver.propagate();
            mysolver.simplify();
            info!(
                "Derived {} new clauses ({} units, {} binary, {} other)",
                new_clauses.len(),
                new_clauses.iter().filter(|c| c.len() == 1).count(),
                new_clauses.iter().filter(|c| c.len() == 2).count(),
                new_clauses.iter().filter(|c| c.len() > 2).count()
            );
            // debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));
            info!(
                "So far derived {} new clauses ({} units, {} binary, {} other)",
                all_derived_clauses.len(),
                all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
                all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
                all_derived_clauses.iter().filter(|c| c.len() > 2).count()
            );
        }

        if cubes_product.len() > args.max_product {
            info!(
                "Too many cubes in the product ({} > {}), restarting",
                cubes_product.len(),
                args.max_product
            );
            cubes_product = vec![vec![]];
            continue;
        }

        info!("Filtering {} hard cubes via solver...", cubes_product.len());
        let time_filter = Instant::now();
        let num_cubes_before_filtering = cubes_product.len();
        let num_conflicts = match &mut algorithm.solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => solver.conflicts() as u64,
        };
        info!("conflicts budget: {}", budget_filter);
        let num_conflicts_limit = num_conflicts + budget_filter;
        let mut in_budget = true;

        if args.use_sorted_filtering {
            let n = variables.len();
            let mut indet_cubes: Vec<Vec<Lit>> = Vec::new();

            let mut degree: HashMap<(Lit, Lit), u64> = HashMap::new();
            for (i, j) in (0..n).tuple_combinations() {
                for cube in cubes_product.iter() {
                    assert_eq!(cube.len(), n);
                    assert_eq!(cube[i].var(), variables[i]);
                    assert_eq!(cube[j].var(), variables[j]);
                    let a = cube[i];
                    let b = cube[j];
                    *degree.entry((a, b)).or_insert(0) += 1;
                }
            }
            // for (&a, &b) in variables.iter().tuple_combinations() {
            //     let pp = degree.get(&(Lit::new(a, false), Lit::new(b, false))).copied().unwrap_or(0);
            //     let pn = degree.get(&(Lit::new(a, false), Lit::new(b,  true))).copied().unwrap_or(0);
            //     let np = degree.get(&(Lit::new(a, true), Lit::new(b, false))).copied().unwrap_or(0);
            //     let nn = degree.get(&(Lit::new(a, true), Lit::new(b, true))).copied().unwrap_or(0);
            //     debug!("degrees for {}-{}: {} / {} / {} / {}", a, b, pp, pn, np, nn);
            // }

            while !cubes_product.is_empty() {
                let time_prepare = Instant::now();
                let num_conflicts = match &mut algorithm.solver {
                    SatSolver::SimpleSat(_) => unreachable!(),
                    SatSolver::Cadical(solver) => solver.conflicts() as u64,
                };
                if num_conflicts > num_conflicts_limit {
                    info!("Budget exhausted");
                    in_budget = false;
                    break;
                }

                let compute_cube_score = |cube: &Vec<Lit>| {
                    let mut score: f64 = 0.0;
                    for (&a, &b) in cube.iter().tuple_combinations() {
                        if let Some(&d) = degree.get(&(a, b)) {
                            if d != 0 {
                                score += 1.0 / d as f64;
                                if d == 1 {
                                    score += 50.0;
                                }
                            }
                        }
                    }
                    score
                };

                let pos = cubes_product
                    .iter()
                    .position_max_by_key(|cube| OrderedFloat(compute_cube_score(cube)))
                    .unwrap();
                let best_cube = cubes_product.swap_remove(pos);
                let best_cube_score = compute_cube_score(&best_cube);

                let time_prepare = time_prepare.elapsed();

                if best_cube_score > 0.0 {
                    debug!(
                        "Max score ({}) cube in {:.1}s: {}",
                        best_cube_score,
                        time_prepare.as_secs_f64(),
                        DisplaySlice(&best_cube)
                    );
                    match &mut algorithm.solver {
                        SatSolver::SimpleSat(_) => unreachable!(),
                        SatSolver::Cadical(solver) => {
                            for &lit in best_cube.iter() {
                                solver.assume(lit.to_external()).unwrap();
                            }
                            solver.limit("conflicts", args.num_conflicts as i32);
                            // info!("Solving {}...", DisplaySlice(&best_cube));
                            let time_solve = Instant::now();
                            match solver.solve().unwrap() {
                                SolveResponse::Unsat => {
                                    debug!(
                                        "UNSAT in {:.1}s for {}",
                                        time_solve.elapsed().as_secs_f64(),
                                        DisplaySlice(&best_cube)
                                    );
                                    for (&a, &b) in best_cube.iter().tuple_combinations() {
                                        // *degree.entry((a, b)).or_insert(0) -= 1;
                                        *degree.get_mut(&(a, b)).unwrap() -= 1;
                                        if degree[&(a, b)] == 0 {
                                            debug!("should derive {}", DisplaySlice(&[-a, -b]));
                                        }
                                    }
                                }
                                SolveResponse::Interrupted => {
                                    debug!(
                                        "INDET in {:.1}s for {}",
                                        time_solve.elapsed().as_secs_f64(),
                                        DisplaySlice(&best_cube)
                                    );
                                    for (&a, &b) in best_cube.iter().tuple_combinations() {
                                        degree.insert((a, b), 0);
                                    }
                                    indet_cubes.push(best_cube);
                                }
                                SolveResponse::Sat => panic!("Unexpected SAT"),
                            }
                        }
                    }
                } else {
                    indet_cubes.push(best_cube);
                    break;
                }
            }

            cubes_product.extend(indet_cubes);
        } else {
            cubes_product.shuffle(&mut algorithm.rng);
            let pb = ProgressBar::new(cubes_product.len() as u64);
            pb.set_style(
                ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")?
                    .progress_chars("#>-"),
            );
            pb.set_message("filtering");
            cubes_product.retain(|cube| {
                pb.inc(1);

                if !in_budget {
                    return true;
                }

                let num_conflicts = match &mut algorithm.solver {
                    SatSolver::SimpleSat(_) => unreachable!(),
                    SatSolver::Cadical(solver) => solver.conflicts() as u64,
                };
                if num_conflicts > num_conflicts_limit {
                    info!("Budget exhausted");
                    in_budget = false;
                }

                if !in_budget {
                    return true;
                }

                match &mut algorithm.solver {
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
                                    mysolver.add_clause(&lemma);
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
        }
        let time_filter = time_filter.elapsed();
        info!(
            "Filtered {} down to {} cubes via solver in {:.1}s",
            num_cubes_before_filtering,
            cubes_product.len(),
            time_filter.as_secs_f64()
        );
        if let Some(f) = &mut file_results {
            writeln!(f, "{},limited,{}", run_number, cubes_product.len())?;
        }

        let num_conflicts = match &mut algorithm.solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => solver.conflicts() as u64,
        };
        // Update the budget for filtering:
        if num_conflicts > num_conflicts_limit {
            budget_filter = (budget_filter as f64 * args.factor_budget_filter) as u64;
        }

        if cubes_product.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);
            break;
        }
        if cubes_product.len() == 1 {
            info!("Adding {} units to the solver", cubes_product[0].len());
            for &lit in &cubes_product[0] {
                algorithm.pool.retain(|&v| v != lit.var());
                if all_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        writeln!(f, "{} 0", lit)?;
                    }
                    algorithm.solver.add_clause(&[lit]);
                    mysolver.add_clause(&[lit]);
                    all_derived_clauses.push(vec![lit]);
                }
            }
            match &mut algorithm.solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    solver.limit("conflicts", 0);
                    solver.solve()?;
                }
            }
            mysolver.propagate();
            mysolver.simplify();
            cubes_product = vec![vec![]];
            continue;
        }

        // Derivation after solver-filtering:
        {
            info!("Deriving clauses for {} cubes...", cubes_product.len());
            let time_derive = Instant::now();
            let derived_clauses = derive_clauses(&cubes_product, args.derive_ternary);
            let time_derive = time_derive.elapsed();
            info!(
                "Derived {} clauses ({} units, {} binary, {} other) for {} cubes in {:.1}s",
                derived_clauses.len(),
                derived_clauses.iter().filter(|c| c.len() == 1).count(),
                derived_clauses.iter().filter(|c| c.len() == 2).count(),
                derived_clauses.iter().filter(|c| c.len() > 2).count(),
                cubes_product.len(),
                time_derive.as_secs_f64()
            );
            // debug!("[{}]", derived_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

            let mut new_clauses: Vec<Vec<Lit>> = Vec::new();
            for mut lemma in derived_clauses {
                lemma.sort_by_key(|lit| lit.inner());
                if all_clauses.insert(lemma.clone()) {
                    if let Some(f) = &mut file_derived_clauses {
                        for lit in lemma.iter() {
                            write!(f, "{} ", lit)?;
                        }
                        writeln!(f, "0")?;
                    }
                    algorithm.solver.add_clause(&lemma);
                    mysolver.add_clause(&lemma);
                    new_clauses.push(lemma.clone());
                    all_derived_clauses.push(lemma);
                }
            }
            match &mut algorithm.solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    solver.limit("conflicts", 0);
                    solver.solve()?;
                }
            }
            mysolver.propagate();
            mysolver.simplify();
            info!(
                "Derived {} new clauses ({} units, {} binary, {} other)",
                new_clauses.len(),
                new_clauses.iter().filter(|c| c.len() == 1).count(),
                new_clauses.iter().filter(|c| c.len() == 2).count(),
                new_clauses.iter().filter(|c| c.len() > 2).count()
            );
            // debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));
            info!(
                "So far derived {} new clauses ({} units, {} binary, {} other)",
                all_derived_clauses.len(),
                all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
                all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
                all_derived_clauses.iter().filter(|c| c.len() > 2).count()
            );
        }

        if !args.only_preprocess {
            info!("Just solving with {} conflicts budget...", budget_solve);
            match &mut algorithm.solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    solver.limit("conflicts", budget_solve as i32);
                    let time_solve = Instant::now();
                    let res = solver.solve().unwrap();
                    let time_solve = time_solve.elapsed();
                    match res {
                        SolveResponse::Interrupted => {
                            info!("UNKNOWN in {:.1} s", time_solve.as_secs_f64());
                            // do nothing
                        }
                        SolveResponse::Unsat => {
                            info!("UNSAT in {:.1} s", time_solve.as_secs_f64());
                            break;
                        }
                        SolveResponse::Sat => {
                            info!("SAT in {:.1} s", time_solve.as_secs_f64());
                            // TODO: dump model
                        }
                    }
                }
            }
            // Update the budget for solving:
            budget_solve = (budget_solve as f64 * args.factor_budget_solve) as u64;
        }

        let time_run = time_run.elapsed();
        info!("Done run {} in {:.1}s", run_number, time_run.as_secs_f64());
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

    if args.only_preprocess {
        info!("Just solving...");
        match &mut algorithm.solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => {
                let time_solve = Instant::now();
                let res = solver.solve().unwrap();
                let time_solve = time_solve.elapsed();
                match res {
                    SolveResponse::Interrupted => {
                        info!("UNKNOWN in {:.1} s", time_solve.as_secs_f64());
                        unreachable!()
                    }
                    SolveResponse::Unsat => {
                        info!("UNSAT in {:.1} s", time_solve.as_secs_f64());
                    }
                    SolveResponse::Sat => {
                        info!("SAT in {:.1} s", time_solve.as_secs_f64());
                        // TODO: dump model
                    }
                }
            }
        }
    }

    println!("\nAll done in {:.3} s", start_time.elapsed().as_secs_f64());
    Ok(())
}
