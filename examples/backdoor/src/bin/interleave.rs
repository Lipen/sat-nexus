use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use color_eyre::eyre::bail;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use itertools::{iproduct, zip_eq, Itertools};
use log::{debug, info};
use rand::prelude::*;

use backdoor::derivation::derive_clauses;
use backdoor::searcher::{BackdoorSearcher, Options, DEFAULT_OPTIONS};
use backdoor::solvers::SatSolver;
use backdoor::utils::{
    clause_from_external, concat_cubes, create_line_writer, determine_vars_pool, filter_cubes, get_hard_tasks,
    propcheck_all_trie_via_internal, write_clause,
};
use cadical::statik::Cadical;
use cadical::{LitValue, SolveResponse};
use simple_sat::lit::Lit;
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

    /// Do ban variables used in the best backdoors on previous runs?
    #[arg(long)]
    ban_used: bool,

    /// Reset banned used variables on empty product.
    #[arg(long)]
    reset_used_vars: bool,

    /// Number of stagnated iterations before re-initialization.
    #[arg(long, value_name = "INT")]
    stagnation_limit: Option<usize>,

    /// Freeze variables.
    #[arg(long)]
    freeze: bool,

    /// Do not derive clauses (units and binary).
    #[arg(long)]
    no_derive: bool,

    /// Derive ternary clauses.
    #[arg(long)]
    derive_ternary: bool,

    /// Maximum product size.
    #[arg(long, value_name = "INT")]
    max_product: usize,

    /// Initial budget (in conflicts) for filtering.
    #[arg(long, value_name = "INT")]
    budget_filter: u64,

    /// Multiplicative factor for filtering budget.
    #[arg(long, value_name = "FLOAT", default_value_t = 1.0)]
    factor_budget_filter: f64,

    /// Budget (in conflicts) for pre-solve.
    #[arg(long, value_name = "INT", default_value_t = 0)]
    budget_presolve: u64,

    /// Initial budget (in conflicts) for solving.
    #[arg(long, value_name = "INT")]
    budget_solve: u64,

    /// Multiplicative factor for solving budget.
    #[arg(long, value_name = "FLOAT", default_value_t = 1.0)]
    factor_budget_solve: f64,

    /// Use novel sorted filtering method.
    #[arg(long)]
    use_sorted_filtering: bool,

    /// Daniil's propcheck-based heuristic.
    #[arg(long, value_name = "INT")]
    pool_limit: Option<usize>,

    /// Always update the budget for filtering.
    #[arg(long)]
    always_update_filter_budget: bool,

    /// Path to a file with proof.
    #[arg(long = "proof", value_name = "FILE")]
    path_proof: Option<PathBuf>,

    /// Write non-binary proof.
    #[arg(long)]
    proof_no_binary: bool,

    /// Do compute cores for easy tasks and invalid cubes.
    #[arg(long)]
    compute_cores: bool,

    /// Do add lemmas from cores.
    #[arg(long)]
    add_cores: bool,

    /// Maximum core size to be added (0 = unlimited).
    #[arg(long, default_value_t = 0)]
    max_core_size: usize,

    /// Comma-separated list of Cadical options ('key=value' pairs, e.g. 'elim=0,ilb=0,check=1').
    #[arg(long)]
    cadical_options: Option<String>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,backdoor::derivation=info")).init();

    let start_time = Instant::now();
    let args = Cli::parse();
    info!("args = {:?}", args);

    if args.add_cores && !args.compute_cores {
        bail!("Cannot add cores (`--add-cores` flag) without computing them (`--compute-cores` flag)");
    }

    // Initialize Cadical:
    let solver = Cadical::new();
    // solver.configure("plain");
    // solver.set_option("elim", 0);
    // solver.set_option("walk", 0);
    // solver.set_option("lucky", 0);
    // solver.set_option("probe", 0);
    // solver.set_option("ilb", 0);
    // solver.set_option("subsume", 0);
    // solver.set_option("vivify", 0);
    // solver.set_option("inprocessing", 0);
    // solver.set_option("check", 1);
    if let Some(s) = &args.cadical_options {
        for part in s.split(",") {
            let mut parts: Vec<&str> = part.splitn(2, '=').collect();
            let key = parts[0];
            let value = parts[1].parse().unwrap();
            info!("Cadical option: {}={}", key, value);
            solver.set_option(key, value);
        }
    }
    if let Some(path_proof) = &args.path_proof {
        if args.proof_no_binary {
            solver.set_option("binary", 0);
        }
        // solver.set_option("lrat", 1);
        // solver.set_option("frat", 1);
        solver.trace_proof(path_proof);
    }
    // solver.read_dimacs(&args.path_cnf, 1);
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
    debug!("solver.vars() = {}", solver.vars());
    debug!("solver.active() = {}", solver.active());
    debug!("solver.redundant() = {}", solver.redundant());
    debug!("solver.irredundant() = {}", solver.irredundant());
    debug!("solver.clauses() = {}", solver.clauses_iter().count());
    debug!("solver.all_clauses() = {}", solver.all_clauses_iter().count());

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

    let mut budget_filter = args.budget_filter;
    let mut budget_solve = args.budget_solve;

    let mut total_time_extract = Duration::ZERO;

    if args.budget_presolve > 0 {
        info!("Pre-solving with {} conflicts budget...", args.budget_presolve);
        match &mut searcher.solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => {
                solver.limit("conflicts", args.budget_presolve as i32);
                let time_solve = Instant::now();
                let res = solver.solve().unwrap();
                let time_solve = time_solve.elapsed();
                solver.internal_backtrack(0);
                match res {
                    SolveResponse::Interrupted => {
                        info!("UNKNOWN in {:.1} s", time_solve.as_secs_f64());
                        // do nothing
                    }
                    SolveResponse::Unsat => {
                        info!("UNSAT in {:.1} s", time_solve.as_secs_f64());
                        panic!("Unexpected UNSAT during pre-solve");
                    }
                    SolveResponse::Sat => {
                        info!("SAT in {:.1} s", time_solve.as_secs_f64());
                        panic!("Unexpected SAT during pre-solve");
                    }
                }
            }
        }

        match &searcher.solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => {
                let res = solver.internal_propagate();
                assert!(res);
            }
        }
    }

    let mut run_number = 0;
    loop {
        run_number += 1;
        info!("Run {}", run_number);
        let time_run = Instant::now();

        // Remove non-active variables from all cubes:
        cubes_product = cubes_product
            .into_iter()
            .map(|cube| cube.into_iter().filter(|&lit| searcher.solver.is_active(lit.var())).collect())
            .collect();

        // Reset banned used variables:
        if args.reset_used_vars && cubes_product == vec![vec![]] {
            searcher.pool.extend(searcher.used_vars.drain());
        }

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

        // Populate the set of ALL clauses:
        match &mut searcher.solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => {
                debug!("Retrieving clauses from the solver...");
                let time_extract = Instant::now();
                let mut num_new = 0;
                for clause in solver.all_clauses_iter() {
                    let mut clause = clause_from_external(clause);
                    clause.sort_by_key(|lit| lit.inner());
                    all_clauses.insert(clause);
                    num_new += 1;
                }
                let time_extract = time_extract.elapsed();
                total_time_extract += time_extract;
                debug!("Extracted {} new clauses in {:.1}s", num_new, time_extract.as_secs_f64());
                debug!(
                    "So far total {} clauses, spent {:.3}s for extraction",
                    all_clauses.len(),
                    total_time_extract.as_secs_f64()
                );
            }
        };

        if args.compute_cores {
            match &searcher.solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    let vars_external: Vec<i32> = backdoor.iter().map(|var| var.to_external() as i32).collect();
                    for &v in vars_external.iter() {
                        assert!(solver.is_active(v), "var {} in backdoor is not active", v);
                    }
                    let orig_hard_len = hard.len();
                    let mut hard = Vec::new();
                    let mut easy = Vec::new();
                    let res = solver.propcheck_all_tree_via_internal(&vars_external, 0, Some(&mut hard), Some(&mut easy));
                    assert_eq!(hard.len(), res as usize);
                    assert_eq!(hard.len(), orig_hard_len);
                    let easy: Vec<Vec<Lit>> = easy
                        .into_iter()
                        .map(|cube| cube.into_iter().map(|i| Lit::from_external(i)).collect())
                        .collect();
                    debug!("Easy tasks: {}", easy.len());

                    let mut easy_cores: HashSet<Vec<Lit>> = HashSet::new();
                    for (i, cube) in easy.iter().enumerate() {
                        let (res, _) = solver.propcheck(&cube.iter().map(|lit| lit.to_external()).collect_vec(), false, false, true);
                        if res {
                            panic!("Unexpected SAT on cube = {}", DisplaySlice(&cube));
                        } else {
                            let core = solver
                                .propcheck_get_core()
                                .into_iter()
                                .map(|i| Lit::from_external(i))
                                .rev()
                                .collect_vec();
                            assert!(!core.is_empty());
                            debug!(
                                "{}/{}: core = {} for cube = {}",
                                i + 1,
                                easy.len(),
                                DisplaySlice(&core),
                                DisplaySlice(cube)
                            );
                            assert_eq!(
                                core.last().unwrap(),
                                cube.last().unwrap(),
                                "core.last() = {}, cube.last() = {}",
                                core.last().unwrap(),
                                cube.last().unwrap()
                            );
                            easy_cores.insert(core);
                        }
                    }
                    debug!("Unique cores from easy tasks: {}", easy_cores.len());
                    debug!("[{}]", easy_cores.iter().map(|c| DisplaySlice(c)).join(", "));

                    if args.add_cores {
                        debug!("Adding {} cores...", easy_cores.len());
                        let mut num_added = 0;
                        for core in easy_cores.iter() {
                            // Skip big cores:
                            if args.max_core_size > 0 && core.len() > args.max_core_size {
                                continue;
                            }

                            let mut lemma = core.iter().map(|&lit| -lit).collect_vec();
                            lemma.sort_by_key(|lit| lit.inner());
                            if all_clauses.insert(lemma.clone()) {
                                if let Some(f) = &mut file_derived_clauses {
                                    write_clause(f, &lemma)?;
                                }
                                searcher.solver.add_clause(&lemma);
                                all_derived_clauses.push(lemma);
                                num_added += 1;
                            }
                        }
                        debug!("Added {} new lemmas from cores", num_added);
                    }
                }
            }

            match &searcher.solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    let res = solver.internal_propagate();
                    assert!(res);
                }
            }
        }

        for &var in backdoor.iter() {
            // assert!(searcher.solver.is_active(var), "var {} in backdoor is not active", var);
            if !searcher.solver.is_active(var) {
                log::warn!("var {} in backdoor is not active", var);
            }
        }

        if hard.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);

            {
                info!("Just solving with {} conflicts budget...", budget_solve);
                match &mut searcher.solver {
                    SatSolver::SimpleSat(_) => unreachable!(),
                    SatSolver::Cadical(solver) => {
                        solver.limit("conflicts", budget_solve as i32);
                        let time_solve = Instant::now();
                        let res = solver.solve().unwrap();
                        let time_solve = time_solve.elapsed();
                        solver.internal_backtrack(0);
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
                                break;
                            }
                        }
                    }
                }
            }

            unreachable!()
            // break;
        }
        if hard.len() == 1 {
            info!("Adding {} units to the solver", hard[0].len());
            for &lit in &hard[0] {
                if all_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        write_clause(f, &[lit])?;
                    }
                    searcher.solver.add_clause(&[lit]);
                    all_derived_clauses.push(vec![lit]);
                }
            }
            cubes_product = vec![vec![]];
            continue;
        }

        // ------------------------------------------------------------------------

        // Derivation for backdoor:
        if !args.no_derive {
            info!("Deriving clauses for {} hard tasks...", hard.len());
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
                    new_clauses.push(lemma.clone());
                    all_derived_clauses.push(lemma);
                }
            }
            info!(
                "Derived {} new clauses ({} units, {} binary, {} other)",
                new_clauses.len(),
                new_clauses.iter().filter(|c| c.len() == 1).count(),
                new_clauses.iter().filter(|c| c.len() == 2).count(),
                new_clauses.iter().filter(|c| c.len() > 2).count()
            );
            debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

            debug!("Adding {} new derived clauses to the solver...", new_clauses.len());
            for lemma in new_clauses {
                searcher.solver.add_clause(&lemma);
            }
        }

        // Remove non-active variables from all cubes:
        cubes_product = cubes_product
            .into_iter()
            .map(|cube| cube.into_iter().filter(|&lit| searcher.solver.is_active(lit.var())).collect())
            .collect();

        let hard: Vec<Vec<Lit>> = hard
            .into_iter()
            .map(|cube| cube.into_iter().filter(|lit| searcher.solver.is_active(lit.var())).collect())
            .collect();

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
            s.extend(backdoor.iter().filter(|&&var| searcher.solver.is_active(var)));
            s.into_iter().sorted().collect_vec()
        };
        debug!("Total {} variables: {}", variables.len(), DisplaySlice(&variables));
        for &var in variables.iter() {
            assert!(searcher.solver.is_active(var), "var {} is not active", var);
        }

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
            if let (true, _) = trie.insert(cube.iter().map(|lit| lit.negated())) {
                num_normal_cubes += 1;
            }
        }
        let time_trie_construct = time_trie_construct.elapsed();
        info!(
            "Trie of size {} with {} leaves constructed out of {} normal cubes in {:.1}s",
            trie.len(),
            trie.num_leaves(),
            num_normal_cubes,
            time_trie_construct.as_secs_f64()
        );

        info!("Filtering {} hard cubes via trie...", trie.num_leaves());
        let time_filter = Instant::now();
        let mut valid = Vec::new();
        let mut invalid = Vec::new(); // TODO: remove 'invalid' extraction
        match &mut searcher.solver {
            SatSolver::SimpleSat(solver) => {
                solver.propcheck_all_trie(&variables, &trie, &mut valid);
            }
            SatSolver::Cadical(solver) => {
                propcheck_all_trie_via_internal(
                    solver,
                    &variables,
                    &trie,
                    0,
                    Some(&mut valid),
                    if args.compute_cores { Some(&mut invalid) } else { None },
                );
            }
        }
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

        if args.compute_cores {
            match &searcher.solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    debug!("Invalid sub-cubes: {}", invalid.len());
                    let mut invalid_cores: HashSet<Vec<Lit>> = HashSet::new();
                    for (i, cube) in invalid.iter().enumerate() {
                        let (res, _) = solver.propcheck(&cube.iter().map(|lit| lit.to_external()).collect_vec(), false, false, true);
                        if res {
                            panic!("Unexpected SAT on cube = {}", DisplaySlice(&cube));
                        } else {
                            let core = solver
                                .propcheck_get_core()
                                .into_iter()
                                .map(|i| Lit::from_external(i))
                                .rev()
                                .collect_vec();
                            assert!(!core.is_empty());
                            debug!(
                                "{}/{}: core = {} for cube = {}",
                                i + 1,
                                invalid.len(),
                                DisplaySlice(&core),
                                DisplaySlice(cube)
                            );
                            assert_eq!(
                                core.last().unwrap(),
                                cube.last().unwrap(),
                                "core.last() = {}, cube.last() = {}",
                                core.last().unwrap(),
                                cube.last().unwrap()
                            );
                            invalid_cores.insert(core);
                        }
                    }
                    debug!("Unique cores from invalid cubes: {}", invalid_cores.len());
                    debug!("[{}]", invalid_cores.iter().map(|c| DisplaySlice(c)).join(", "));

                    if args.add_cores {
                        debug!("Adding {} cores...", invalid_cores.len());
                        let mut num_added = 0;
                        for core in invalid_cores.iter() {
                            // Skip big cores:
                            if args.max_core_size > 0 && core.len() > args.max_core_size {
                                continue;
                            }

                            let mut lemma = core.iter().map(|&lit| -lit).collect_vec();
                            lemma.sort_by_key(|lit| lit.inner());
                            if all_clauses.insert(lemma.clone()) {
                                if let Some(f) = &mut file_derived_clauses {
                                    write_clause(f, &lemma)?;
                                }
                                searcher.solver.add_clause(&lemma);
                                all_derived_clauses.push(lemma);
                                num_added += 1;
                            }
                        }
                        debug!("Added {} new lemmas from cores", num_added);
                    }
                }
            }
        }

        if cubes_product.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);

            {
                info!("Just solving with {} conflicts budget...", budget_solve);
                match &mut searcher.solver {
                    SatSolver::SimpleSat(_) => unreachable!(),
                    SatSolver::Cadical(solver) => {
                        let time_solve = Instant::now();
                        solver.limit("conflicts", budget_solve as i32);
                        let res = solver.solve().unwrap();
                        solver.internal_backtrack(0);
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
                                break;
                            }
                        }
                    }
                }
            }

            unreachable!()
            // break;
        }
        if cubes_product.len() == 1 {
            info!("Adding {} units to the solver", cubes_product[0].len());
            for &lit in &cubes_product[0] {
                if all_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        write_clause(f, &[lit])?;
                    }
                    searcher.solver.add_clause(&[lit]);
                    all_derived_clauses.push(vec![lit]);
                }
            }
            cubes_product = vec![vec![]];
            continue;
        }

        // Derivation after trie-filtering:
        if !args.no_derive {
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
                        write_clause(f, &lemma)?;
                    }
                    new_clauses.push(lemma.clone());
                    all_derived_clauses.push(lemma);
                }
            }
            info!(
                "Derived {} new clauses ({} units, {} binary, {} other)",
                new_clauses.len(),
                new_clauses.iter().filter(|c| c.len() == 1).count(),
                new_clauses.iter().filter(|c| c.len() == 2).count(),
                new_clauses.iter().filter(|c| c.len() > 2).count()
            );
            debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

            debug!("Adding {} new derived clauses to the solver...", new_clauses.len());
            for lemma in new_clauses {
                searcher.solver.add_clause(&lemma);
            }

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

        // Remove non-active variables from all cubes:
        cubes_product = cubes_product
            .into_iter()
            .map(|cube| cube.into_iter().filter(|&lit| searcher.solver.is_active(lit.var())).collect())
            .collect();

        info!("Filtering {} hard cubes via limited solver...", cubes_product.len());
        let time_filter = Instant::now();
        let num_cubes_before_filtering = cubes_product.len();
        let num_conflicts = match &mut searcher.solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => solver.conflicts() as u64,
        };
        info!("conflicts budget: {}", budget_filter);
        let num_conflicts_limit = num_conflicts + budget_filter;
        let mut in_budget = true;

        if args.use_sorted_filtering {
            cubes_product = filter_cubes(
                cubes_product,
                args.num_conflicts as u64,
                num_conflicts_limit,
                &mut searcher.solver,
                &mut all_clauses,
                &mut all_derived_clauses,
                &mut file_derived_clauses,
            );
        } else {
            let mut cores: HashSet<Vec<Lit>> = HashSet::new();

            cubes_product.shuffle(&mut searcher.rng);
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

                let num_conflicts = match &mut searcher.solver {
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

                match &searcher.solver {
                    SatSolver::SimpleSat(_) => unreachable!(),
                    SatSolver::Cadical(solver) => {
                        let time_solve = Instant::now();
                        for &lit in cube.iter() {
                            solver.assume(lit.to_external()).unwrap();
                        }
                        solver.limit("conflicts", args.num_conflicts as i32);
                        let res = solver.solve().unwrap();
                        let time_solve = time_solve.elapsed();

                        match res {
                            SolveResponse::Interrupted => true,
                            SolveResponse::Unsat => {
                                if args.compute_cores {
                                    let mut core = Vec::new();
                                    for &lit in cube {
                                        if solver.failed(lit.to_external()).unwrap() {
                                            core.push(lit);
                                        }
                                    }
                                    pb.println(format!(
                                        "UNSAT for cube = {} in {:.1}s, core = {}",
                                        DisplaySlice(&cube),
                                        time_solve.as_secs_f64(),
                                        DisplaySlice(&core)
                                    ));
                                    cores.insert(core);
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

            // Populate the set of ALL clauses:
            match &mut searcher.solver {
                SatSolver::SimpleSat(_) => unreachable!(),
                SatSolver::Cadical(solver) => {
                    debug!("Retrieving clauses from the solver...");
                    let time_extract = Instant::now();
                    let mut num_new = 0;
                    for clause in solver.all_clauses_iter() {
                        let mut clause = clause_from_external(clause);
                        clause.sort_by_key(|lit| lit.inner());
                        all_clauses.insert(clause);
                        num_new += 1;
                    }
                    let time_extract = time_extract.elapsed();
                    total_time_extract += time_extract;
                    debug!("Extracted {} new clauses in {:.1}s", num_new, time_extract.as_secs_f64());
                    debug!(
                        "So far total {} clauses, spent {:.3}s for extraction",
                        all_clauses.len(),
                        total_time_extract.as_secs_f64()
                    );
                }
            }

            if args.add_cores {
                debug!("Adding {} cores...", cores.len());
                let mut num_added = 0;
                for core in cores.iter() {
                    // Skip big cores:
                    if args.max_core_size > 0 && core.len() > args.max_core_size {
                        continue;
                    }

                    let lemma = core.iter().map(|&lit| -lit).collect_vec();
                    if all_clauses.insert(lemma.clone()) {
                        if let Some(f) = &mut file_derived_clauses {
                            write_clause(f, &lemma)?;
                        }
                        searcher.solver.add_clause(&lemma);
                        all_derived_clauses.push(lemma);
                        num_added += 1;
                    }
                }
                debug!("Added {} new lemmas from cores", num_added);
            }
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

        let num_conflicts = match &mut searcher.solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => solver.conflicts() as u64,
        };
        // Update the budget for filtering:
        if args.always_update_filter_budget || num_conflicts > num_conflicts_limit {
            budget_filter = (budget_filter as f64 * args.factor_budget_filter) as u64;
        }

        if cubes_product.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);

            {
                info!("Just solving with {} conflicts budget...", budget_solve);
                match &mut searcher.solver {
                    SatSolver::SimpleSat(_) => unreachable!(),
                    SatSolver::Cadical(solver) => {
                        solver.limit("conflicts", budget_solve as i32);
                        let time_solve = Instant::now();
                        let res = solver.solve().unwrap();
                        let time_solve = time_solve.elapsed();
                        solver.internal_backtrack(0);
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
                                break;
                            }
                        }
                    }
                }
            }

            unreachable!()
            // break;
        }
        if cubes_product.len() == 1 {
            info!("Adding {} units to the solver", cubes_product[0].len());
            for &lit in &cubes_product[0] {
                if all_clauses.insert(vec![lit]) {
                    if let Some(f) = &mut file_derived_clauses {
                        write_clause(f, &[lit])?;
                    }
                    searcher.solver.add_clause(&[lit]);
                    all_derived_clauses.push(vec![lit]);
                }
            }
            cubes_product = vec![vec![]];
            continue;
        }

        // Derivation after filtering:
        if !args.no_derive {
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
                        write_clause(f, &lemma)?;
                    }
                    new_clauses.push(lemma.clone());
                    all_derived_clauses.push(lemma);
                }
            }
            info!(
                "Derived {} new clauses ({} units, {} binary, {} other)",
                new_clauses.len(),
                new_clauses.iter().filter(|c| c.len() == 1).count(),
                new_clauses.iter().filter(|c| c.len() == 2).count(),
                new_clauses.iter().filter(|c| c.len() > 2).count()
            );
            debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

            debug!("Adding {} new derived clauses to the solver...", new_clauses.len());
            for lemma in new_clauses {
                searcher.solver.add_clause(&lemma);
            }

            info!(
                "So far derived {} new clauses ({} units, {} binary, {} other)",
                all_derived_clauses.len(),
                all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
                all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
                all_derived_clauses.iter().filter(|c| c.len() > 2).count()
            );
        };

        // match &mut searcher.solver {
        //     SatSolver::SimpleSat(_) => unreachable!(),
        //     SatSolver::Cadical(solver) => {
        //         debug!("Retrieving clauses from the solver...");
        //         let time_all_clauses = Instant::now();
        //         let mut all_cadical_clauses = HashSet::new();
        //         for clause in solver.all_clauses_iter() {
        //             let mut clause = clause.into_iter().map(Lit::from_external).collect_vec();
        //             clause.sort_by_key(|lit| lit.inner());
        //             all_cadical_clauses.insert(clause);
        //         }
        //         let time_all_clauses = time_all_clauses.elapsed();
        //         debug!(
        //             "Retrieved {} clauses from the solver in {:.1}s",
        //             all_cadical_clauses.len(),
        //             time_all_clauses.as_secs_f64()
        //         );
        //         info!("Solver currently has {} clauses", all_cadical_clauses.len());
        //     }
        // };

        info!("Just solving with {} conflicts budget...", budget_solve);
        match &mut searcher.solver {
            SatSolver::SimpleSat(_) => unreachable!(),
            SatSolver::Cadical(solver) => {
                solver.limit("conflicts", budget_solve as i32);
                let time_solve = Instant::now();
                solver.reset_assumptions();
                let res = solver.solve().unwrap();
                let time_solve = time_solve.elapsed();
                solver.internal_backtrack(0);
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
                        break;
                    }
                }
            }
        }
        // Update the budget for solving:
        budget_solve = (budget_solve as f64 * args.factor_budget_solve) as u64;

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

    debug!("Time spent on extracting all clauses: {:.3}s", total_time_extract.as_secs_f64());

    println!("\nAll done in {:.3} s", start_time.elapsed().as_secs_f64());
    Ok(())
}
