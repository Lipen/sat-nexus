use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::fs::File;
use std::io::LineWriter;
use std::io::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use color_eyre::eyre::bail;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use itertools::{iproduct, zip_eq, Itertools};
use log::{debug, info};
use ordered_float::OrderedFloat;
use rand::prelude::*;

use backdoor::derivation::derive_clauses;
use backdoor::searcher::{BackdoorSearcher, Options, DEFAULT_OPTIONS};
use backdoor::solver::Solver;
use backdoor::utils::{
    concat_cubes, create_line_writer, determine_vars_pool, get_hard_tasks, lits_from_external, propcheck_all_trie_via_internal,
    write_clause,
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

    /// Path to output file in DIMACS format.
    /// If the problem is SAT, contains two lines: "s SATISFIABLE\nv 1 2 ... 0\n",
    /// else contains a single line: "s UNSATISFIABLE" or "s INDET".
    // #[arg(short = 'o', long = "output", value_name = "FILE")]
    // path_output: Option<PathBuf>,

    /// Path to a file with results.
    #[arg(long = "results", value_name = "FILE")]
    path_results: Option<PathBuf>,

    /// Random seed.
    #[arg(long, value_name = "INT", default_value_t = DEFAULT_OPTIONS.seed)]
    seed: u64,

    /// Backdoor size.
    #[arg(long, value_name = "INT")]
    backdoor_size: usize,

    /// Number of EA iterations.
    #[arg(long, value_name = "INT")]
    num_iters: usize,

    /// Number of stagnated iterations before re-initialization.
    #[arg(long, value_name = "INT")]
    stagnation_limit: Option<usize>,

    /// Timeout for each EA run.
    #[arg(long, value_name = "FLOAT")]
    run_timeout: Option<f64>,

    /// Daniil's propcheck-based heuristic.
    #[arg(long, value_name = "INT")]
    pool_limit: Option<usize>,

    /// Do ban variables used in the best backdoors on previous runs?
    #[arg(long)]
    ban_used: bool,

    /// Reset banned used variables on empty product.
    #[arg(long)]
    reset_used_vars: bool,

    /// Comma-separated list of allowed variables (1-based indices).
    #[arg(long = "allow", value_name = "INT...")]
    allowed_vars: Option<String>,

    /// Comma-separated list of banned variables (1-based indices).
    #[arg(long = "ban", value_name = "INT...")]
    banned_vars: Option<String>,

    /// Freeze variables.
    #[arg(long)]
    freeze: bool,

    /// Do not derive clauses.
    #[arg(long)]
    no_derive: bool,

    /// Derive ternary clauses.
    #[arg(long)]
    derive_ternary: bool,

    /// Maximum product size.
    #[arg(long, value_name = "INT")]
    max_product: usize,

    /// Use novel sorted filtering method.
    #[arg(long)]
    use_sorted_filtering: bool,

    /// Number of conflicts (budget per task in filtering).
    #[arg(long, value_name = "INT", default_value_t = 1000)]
    num_conflicts: usize,

    /// Initial budget (in conflicts) for filtering.
    #[arg(long, value_name = "INT")]
    budget_filter: u64,

    /// Multiplicative factor for filtering budget.
    #[arg(long, value_name = "FLOAT", default_value_t = 1.0)]
    factor_budget_filter: f64,

    /// Initial budget (in conflicts) for solving.
    #[arg(long, value_name = "INT")]
    budget_solve: u64,

    /// Multiplicative factor for solving budget.
    #[arg(long, value_name = "FLOAT", default_value_t = 1.0)]
    factor_budget_solve: f64,

    /// Budget (in conflicts) for pre-solve.
    #[arg(long, value_name = "INT", default_value_t = 0)]
    budget_presolve: u64,

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

    /// Do not print solver stats in the end.
    #[arg(long)]
    no_stats: bool,
}

#[allow(dead_code)]
enum SolveResult {
    SAT(Vec<Lit>),
    UNSAT,
    UNKNOWN,
}

fn solve(args: Cli) -> color_eyre::Result<SolveResult> {
    // Initialize Cadical:
    let cadical = Cadical::new();
    // cadical.configure("plain");
    // cadical.set_option("elim", 0);
    // cadical.set_option("walk", 0);
    // cadical.set_option("lucky", 0);
    // cadical.set_option("probe", 0);
    // cadical.set_option("ilb", 0);
    // cadical.set_option("subsume", 0);
    // cadical.set_option("vivify", 0);
    // cadical.set_option("inprocessing", 0);
    // cadical.set_option("check", 1);
    if let Some(s) = &args.cadical_options {
        for part in s.split(",") {
            let parts: Vec<&str> = part.splitn(2, '=').collect();
            let key = parts[0];
            let value = parts[1].parse().unwrap();
            info!("Cadical option: {}={}", key, value);
            cadical.set_option(key, value);
        }
    }
    if let Some(path_proof) = &args.path_proof {
        if args.proof_no_binary {
            cadical.set_option("binary", 0);
        }
        // cadical.set_option("lrat", 1);
        // cadical.set_option("frat", 1);
        cadical.trace_proof(path_proof);
    }
    // solver.read_dimacs(&args.path_cnf, 1);
    for clause in parse_dimacs(&args.path_cnf) {
        cadical.add_clause(clause.into_iter().map(|lit| lit.to_external()));
    }
    if args.freeze {
        info!("Freezing variables...");
        for i in 0..cadical.vars() {
            let lit = (i + 1) as i32;
            cadical.freeze(lit).unwrap();
        }
    }
    cadical.limit("conflicts", 0);
    cadical.solve()?;
    debug!("vars() = {}", cadical.vars());
    debug!("active() = {}", cadical.active());
    debug!("redundant() = {}", cadical.redundant());
    debug!("irredundant() = {}", cadical.irredundant());
    debug!("clauses() = {}", cadical.clauses_iter().count());
    debug!("all_clauses() = {}", cadical.all_clauses_iter().count());

    // Create the pool of variables available for EA:
    let pool: Vec<Var> = determine_vars_pool(&args.path_cnf, &args.allowed_vars, &args.banned_vars);

    // Set up the evolutionary algorithm:
    let options = Options {
        seed: args.seed,
        ban_used_variables: args.ban_used,
        ..DEFAULT_OPTIONS
    };
    let mut searcher = BackdoorSearcher::new(Solver::new(cadical), pool, options);

    // Create and open the file with derived clauses:
    // let mut file_derived_clauses = Some(create_line_writer("derived_clauses.txt"));
    let mut file_derived_clauses: Option<LineWriter<File>> = None;

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

    let mut budget_filter = args.budget_filter;
    let mut budget_solve = args.budget_solve;

    let mut total_time_extract = Duration::ZERO;

    let mut final_model: Option<Vec<Lit>> = None;

    if args.budget_presolve > 0 {
        info!("Pre-solving with {} conflicts budget...", args.budget_presolve);
        searcher.solver.0.limit("conflicts", args.budget_presolve as i32);
        let time_solve = Instant::now();
        let res = searcher.solver.solve();
        let time_solve = time_solve.elapsed();
        match res {
            SolveResponse::Interrupted => {
                info!("UNKNOWN in {:.1} s", time_solve.as_secs_f64());
                // do nothing
            }
            SolveResponse::Unsat => {
                info!("UNSAT in {:.1} s", time_solve.as_secs_f64());
                return Ok(SolveResult::UNSAT);
            }
            SolveResponse::Sat => {
                info!("SAT in {:.1} s", time_solve.as_secs_f64());
                let model = (1..=searcher.solver.0.vars())
                    .map(|i| {
                        let v = Var::from_external(i as u32);
                        match searcher.solver.0.val(i as i32).unwrap() {
                            LitValue::True => Lit::new(v, false),
                            LitValue::False => Lit::new(v, true),
                        }
                    })
                    .collect_vec();
                return Ok(SolveResult::SAT(model));
            }
        }
        searcher.solver.0.internal_backtrack(0);
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
            searcher.banned_vars.clear();
        }

        if let Some(result) = searcher.run(
            args.backdoor_size,
            args.num_iters,
            args.stagnation_limit,
            args.run_timeout,
            Some(((1u64 << args.backdoor_size) - 1) as f64 / (1u64 << args.backdoor_size) as f64),
            0,
            args.pool_limit,
        ) {
            let backdoor = result.best_instance.get_variables();
            let hard = get_hard_tasks(&backdoor, &searcher.solver.0);
            debug!("Backdoor {} has {} hard tasks", DisplaySlice(&backdoor), hard.len());
            assert_eq!(hard.len() as u64, result.best_fitness.num_hard);

            if hard.len() == 0 {
                info!("Found strong backdoor: {}", DisplaySlice(&backdoor));

                info!("Just solving...");
                searcher.solver.0.reset_assumptions();
                let time_solve = Instant::now();
                let res = searcher.solver.solve();
                let time_solve = time_solve.elapsed();
                match res {
                    SolveResponse::Interrupted => {
                        info!("UNKNOWN in {:.1} s", time_solve.as_secs_f64());
                        // do nothing
                    }
                    SolveResponse::Unsat => {
                        info!("UNSAT in {:.1} s", time_solve.as_secs_f64());
                        return Ok(SolveResult::UNSAT);
                    }
                    SolveResponse::Sat => {
                        info!("SAT in {:.1} s", time_solve.as_secs_f64());
                        let model = (1..=searcher.solver.0.vars())
                            .map(|i| {
                                let v = Var::from_external(i as u32);
                                match searcher.solver.0.val(i as i32).unwrap() {
                                    LitValue::True => Lit::new(v, false),
                                    LitValue::False => Lit::new(v, true),
                                }
                            })
                            .collect_vec();
                        return Ok(SolveResult::SAT(model));
                    }
                }

                unreachable!();
                // break;
            }

            // Populate the set of ALL clauses:
            debug!("Retrieving clauses from the solver...");
            let time_extract = Instant::now();
            let mut num_new = 0;
            for clause in searcher.solver.0.all_clauses_iter() {
                let mut clause = lits_from_external(clause);
                clause.sort_by_key(|lit| lit.inner());
                all_clauses.insert(clause);
                num_new += 1;
            }
            let time_extract = time_extract.elapsed();
            total_time_extract += time_extract;
            debug!("Extracted {} new clauses in {:.1}s", num_new, time_extract.as_secs_f64());
            debug!(
                "So far total {} clauses, total spent {:.3}s for extraction",
                all_clauses.len(),
                total_time_extract.as_secs_f64()
            );

            if args.compute_cores {
                let vars_external: Vec<i32> = backdoor
                    .iter()
                    .map(|var| var.to_external() as i32)
                    .filter(|&v| searcher.solver.0.is_active(v))
                    .collect();
                debug!("Using vars for cores: {}", DisplaySlice(&vars_external));
                // for &v in vars_external.iter() {
                //     assert!(solver.is_active(v), "var {} in backdoor is not active", v);
                // }
                // let orig_hard_len = hard.len();
                let mut hard = Vec::new();
                let mut easy = Vec::new();
                let res = searcher
                    .solver
                    .0
                    .propcheck_all_tree_via_internal(&vars_external, 0, Some(&mut hard), Some(&mut easy));
                assert_eq!(hard.len(), res as usize);
                // assert_eq!(hard.len(), orig_hard_len);
                let easy: Vec<Vec<Lit>> = easy
                    .into_iter()
                    .map(|cube| cube.into_iter().map(|i| Lit::from_external(i)).collect())
                    .collect();
                debug!("Easy tasks: {}", easy.len());

                let mut easy_cores: HashSet<Vec<Lit>> = HashSet::new();
                for (i, cube) in easy.iter().enumerate() {
                    let (res, _) = searcher.solver.propcheck_save_core(&cube);
                    assert!(!res, "Unexpected SAT on cube = {}", DisplaySlice(&cube));
                    let core = searcher
                        .solver
                        .0
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
                {
                    let res = searcher.solver.0.internal_propagate();
                    assert!(res);
                }
            }

            for &var in backdoor.iter() {
                // assert!(searcher.solver.is_active(var), "var {} in backdoor is not active", var);
                if !searcher.solver.is_active(var) {
                    log::error!("var {} in backdoor is not active", var);
                }
            }

            if hard.is_empty() {
                info!("No more cubes to solve after {} runs", run_number);

                info!("Just solving with {} conflicts budget...", budget_solve);
                searcher.solver.0.reset_assumptions();
                searcher.solver.0.limit("conflicts", budget_solve as i32);
                let time_solve = Instant::now();
                let res = searcher.solver.solve();
                let time_solve = time_solve.elapsed();
                match res {
                    SolveResponse::Interrupted => {
                        info!("UNKNOWN in {:.1} s", time_solve.as_secs_f64());
                        // do nothing
                    }
                    SolveResponse::Unsat => {
                        info!("UNSAT in {:.1} s", time_solve.as_secs_f64());
                        return Ok(SolveResult::UNSAT);
                    }
                    SolveResponse::Sat => {
                        info!("SAT in {:.1} s", time_solve.as_secs_f64());
                        let model = (1..=searcher.solver.0.vars())
                            .map(|i| {
                                let v = Var::from_external(i as u32);
                                match searcher.solver.0.val(i as i32).unwrap() {
                                    LitValue::True => Lit::new(v, false),
                                    LitValue::False => Lit::new(v, true),
                                }
                            })
                            .collect_vec();
                        return Ok(SolveResult::SAT(model));
                    }
                }
                searcher.solver.0.internal_backtrack(0);

                unreachable!()
                // break;
            }
            if hard.len() == 1 {
                debug!("Adding {} units to the solver", hard[0].len());
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
                    "Derived {} clauses ({} units, {} binary, {} ternary, {} other) for backdoor in {:.1}s",
                    derived_clauses.len(),
                    derived_clauses.iter().filter(|c| c.len() == 1).count(),
                    derived_clauses.iter().filter(|c| c.len() == 2).count(),
                    derived_clauses.iter().filter(|c| c.len() == 3).count(),
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
                    "Derived {} new clauses ({} units, {} binary, {} ternary, {} other)",
                    new_clauses.len(),
                    new_clauses.iter().filter(|c| c.len() == 1).count(),
                    new_clauses.iter().filter(|c| c.len() == 2).count(),
                    new_clauses.iter().filter(|c| c.len() == 3).count(),
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
            propcheck_all_trie_via_internal(
                &searcher.solver.0,
                &variables,
                &trie,
                0,
                Some(&mut valid),
                if args.compute_cores { Some(&mut invalid) } else { None },
            );
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
                debug!("Invalid sub-cubes: {}", invalid.len());
                let mut invalid_cores: HashSet<Vec<Lit>> = HashSet::new();
                for (i, cube) in invalid.iter().enumerate() {
                    let (res, _) = searcher.solver.propcheck_save_core(&cube);
                    assert!(!res, "Unexpected SAT on cube = {}", DisplaySlice(&cube));
                    let core = searcher
                        .solver
                        .0
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

            if cubes_product.is_empty() {
                info!("No more cubes to solve after {} runs", run_number);

                info!("Just solving with {} conflicts budget...", budget_solve);
                searcher.solver.0.reset_assumptions();
                searcher.solver.0.limit("conflicts", budget_solve as i32);
                let time_solve = Instant::now();
                let res = searcher.solver.solve();
                let time_solve = time_solve.elapsed();
                match res {
                    SolveResponse::Interrupted => {
                        info!("UNKNOWN in {:.1} s", time_solve.as_secs_f64());
                        // do nothing
                    }
                    SolveResponse::Unsat => {
                        info!("UNSAT in {:.1} s", time_solve.as_secs_f64());
                        return Ok(SolveResult::UNSAT);
                    }
                    SolveResponse::Sat => {
                        info!("SAT in {:.1} s", time_solve.as_secs_f64());
                        let model = (1..=searcher.solver.0.vars())
                            .map(|i| {
                                let v = Var::from_external(i as u32);
                                match searcher.solver.0.val(i as i32).unwrap() {
                                    LitValue::True => Lit::new(v, false),
                                    LitValue::False => Lit::new(v, true),
                                }
                            })
                            .collect_vec();
                        return Ok(SolveResult::SAT(model));
                    }
                }
                searcher.solver.0.internal_backtrack(0);

                unreachable!()
                // break;
            }
            if cubes_product.len() == 1 {
                debug!("Adding {} units to the solver", cubes_product[0].len());
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
                    "Derived {} clauses ({} units, {} binary, {} ternary, {} other) for {} cubes in {:.1}s",
                    derived_clauses.len(),
                    derived_clauses.iter().filter(|c| c.len() == 1).count(),
                    derived_clauses.iter().filter(|c| c.len() == 2).count(),
                    derived_clauses.iter().filter(|c| c.len() == 3).count(),
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
                    "Derived {} new clauses ({} units, {} binary, {} ternary, {} other)",
                    new_clauses.len(),
                    new_clauses.iter().filter(|c| c.len() == 1).count(),
                    new_clauses.iter().filter(|c| c.len() == 2).count(),
                    new_clauses.iter().filter(|c| c.len() == 3).count(),
                    new_clauses.iter().filter(|c| c.len() > 2).count()
                );
                debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

                debug!("Adding {} new derived clauses to the solver...", new_clauses.len());
                for lemma in new_clauses {
                    searcher.solver.add_clause(&lemma);
                }

                debug!(
                    "So far derived {} new clauses ({} units, {} binary, {} ternary, {} other)",
                    all_derived_clauses.len(),
                    all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
                    all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
                    all_derived_clauses.iter().filter(|c| c.len() == 3).count(),
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
        }

        // Remove non-active variables from all cubes:
        cubes_product = cubes_product
            .into_iter()
            .map(|cube| cube.into_iter().filter(|&lit| searcher.solver.is_active(lit.var())).collect())
            .collect();

        info!("Filtering {} hard cubes via limited solver...", cubes_product.len());
        let time_filter = Instant::now();
        let num_cubes_before_filtering = cubes_product.len();
        let num_conflicts = searcher.solver.0.conflicts() as u64;
        info!("conflicts budget: {}", budget_filter);
        let num_conflicts_limit = num_conflicts + budget_filter;
        let mut in_budget = true;

        if args.use_sorted_filtering {
            debug!("Computing neighbors...");
            let time_compute_neighbors = Instant::now();
            let mut neighbors: HashMap<(Lit, Lit), Vec<usize>> = HashMap::new();
            for (i, cube) in cubes_product.iter().enumerate() {
                for (&a, &b) in cube.iter().tuple_combinations() {
                    neighbors.entry((a, b)).or_default().push(i);
                }
            }
            let time_compute_neighbors = time_compute_neighbors.elapsed();
            debug!(
                "Computed neighbors (size={}, cubes={}) in {:.1}s",
                neighbors.len(),
                neighbors.values().map(|vs| vs.len()).sum::<usize>(),
                time_compute_neighbors.as_secs_f64()
            );

            let compute_cube_score = |cube: &[Lit], neighbors: &HashMap<(Lit, Lit), Vec<usize>>| {
                let mut score: f64 = 0.0;
                for (&a, &b) in cube.iter().tuple_combinations() {
                    if let Some(neighbors) = neighbors.get(&(a, b)) {
                        let d = neighbors.len();
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

            debug!("Computing cube score...");
            let time_cube_scores = Instant::now();
            let mut cube_score: Vec<f64> = cubes_product.iter().map(|cube| compute_cube_score(cube, &neighbors)).collect();
            let time_cube_scores = time_cube_scores.elapsed();
            debug!(
                "Computed cube scores (size={}) in {:.1}s",
                cube_score.len(),
                time_cube_scores.as_secs_f64()
            );

            let mut remaining_cubes: Vec<usize> = (0..cubes_product.len()).collect();
            let mut indet_cubes: Vec<usize> = Vec::new();
            let mut cores: HashSet<Vec<Lit>> = HashSet::new();

            let verb = false;

            while !remaining_cubes.is_empty() {
                let num_conflicts = searcher.solver.0.conflicts() as u64;
                if num_conflicts > num_conflicts_limit {
                    info!("Budget exhausted");
                    break;
                }

                if false {
                    // debug!("Asserting...");
                    let time_asserting = Instant::now();
                    for &i in remaining_cubes.iter() {
                        assert!(
                            (compute_cube_score(&cubes_product[i], &neighbors) - cube_score[i]).abs() <= 1e-6,
                            "compute = {}, score = {}",
                            compute_cube_score(&cubes_product[i], &neighbors),
                            cube_score[i]
                        );
                    }
                    let time_asserting = time_asserting.elapsed();
                    debug!("Asserted in {:.1}s", time_asserting.as_secs_f64());
                }

                let best_cube_position = remaining_cubes
                    .iter()
                    .position_max_by_key(|&&i| OrderedFloat(cube_score[i]))
                    .unwrap();
                let best_cube = remaining_cubes.swap_remove(best_cube_position);
                let best_cube_score = cube_score[best_cube];

                if best_cube_score > 0.0 {
                    // debug!(
                    //     "Max score ({}) cube: {}",
                    //     best_cube_score,
                    //     DisplaySlice(&cubes[best_cube])
                    // );
                    searcher.solver.0.reset_assumptions();
                    for &lit in cubes_product[best_cube].iter() {
                        searcher.solver.assume(lit);
                    }
                    searcher.solver.0.limit("conflicts", (args.num_conflicts as u64) as i32);
                    // debug!("Solving {}...", DisplaySlice(&best_cube));
                    let time_solve = Instant::now();
                    let res = searcher.solver.solve();
                    let time_solve = time_solve.elapsed();
                    match res {
                        SolveResponse::Unsat => {
                            if verb {
                                debug!(
                                    "UNSAT in {:.1}s for cube with score {}: {}",
                                    time_solve.as_secs_f64(),
                                    best_cube_score,
                                    DisplaySlice(&cubes_product[best_cube])
                                );
                            }
                            let time_rescore = Instant::now();
                            for (&a, &b) in cubes_product[best_cube].iter().tuple_combinations() {
                                let d = neighbors[&(a, b)].len();
                                if d == 0 {
                                    continue;
                                } else if d == 1 {
                                    // debug!("should derive {}", DisplaySlice(&[-a, -b]));
                                    assert_eq!(neighbors[&(a, b)][0], best_cube);
                                    cube_score[best_cube] = 0.0;
                                } else {
                                    for &i in neighbors[&(a, b)].iter() {
                                        cube_score[i] -= 1.0 / d as f64;
                                        cube_score[i] += 1.0 / (d - 1) as f64;
                                        if d - 1 == 1 {
                                            cube_score[i] += 50.0;
                                        }
                                    }
                                }
                                neighbors.get_mut(&(a, b)).unwrap().retain(|&i| i != best_cube);
                            }
                            let time_rescore = time_rescore.elapsed();
                            if verb || time_rescore.as_secs_f64() > 0.1 {
                                debug!("Rescored in {:.1}s", time_rescore.as_secs_f64());
                            }

                            if args.compute_cores {
                                let mut core = Vec::new();
                                for &lit in cubes_product[best_cube].iter() {
                                    if searcher.solver.failed(lit) {
                                        core.push(lit);
                                    }
                                }
                                // debug!("UNSAT for cube = {}, core = {}", DisplaySlice(&cube), DisplaySlice(&core));
                                cores.insert(core);
                            }
                        }
                        SolveResponse::Interrupted => {
                            if verb {
                                debug!(
                                    "INDET in {:.1}s for cube with score {}: {}",
                                    time_solve.as_secs_f64(),
                                    best_cube_score,
                                    DisplaySlice(&cubes_product[best_cube])
                                );
                            }
                            let time_rescore = Instant::now();
                            for (&a, &b) in cubes_product[best_cube].iter().tuple_combinations() {
                                let ns = neighbors.get_mut(&(a, b)).unwrap();
                                let d = ns.len();
                                for i in ns.drain(..) {
                                    // score[cube] -= 1 / d
                                    cube_score[i] -= 1.0 / d as f64;
                                }
                                assert_eq!(neighbors[&(a, b)].len(), 0);
                            }
                            let time_rescore = time_rescore.elapsed();
                            if verb {
                                debug!("Rescored in {:.1}s", time_rescore.as_secs_f64());
                            }
                            indet_cubes.push(best_cube);
                        }
                        SolveResponse::Sat => {
                            if verb {
                                debug!(
                                    "SAT in {:.1}s for cube with score {}: {}",
                                    time_solve.as_secs_f64(),
                                    best_cube_score,
                                    DisplaySlice(&cubes_product[best_cube])
                                );
                            }
                            let model = (1..=searcher.solver.0.vars())
                                .map(|i| {
                                    let v = Var::from_external(i as u32);
                                    match searcher.solver.0.val(i as i32).unwrap() {
                                        LitValue::True => Lit::new(v, false),
                                        LitValue::False => Lit::new(v, true),
                                    }
                                })
                                .collect_vec();
                            final_model = Some(model);
                            break;
                        }
                    }
                } else {
                    indet_cubes.push(best_cube);
                    break;
                }
            }

            if let Some(model) = final_model {
                return Ok(SolveResult::SAT(model));
            }

            // Populate the set of ALL clauses:
            debug!("Retrieving clauses from the solver...");
            let time_extract = Instant::now();
            let mut num_new = 0;
            for clause in searcher.solver.0.all_clauses_iter() {
                let mut clause = lits_from_external(clause);
                clause.sort_by_key(|lit| lit.inner());
                (&mut all_clauses).insert(clause);
                num_new += 1;
            }
            let time_extract = time_extract.elapsed();
            total_time_extract += time_extract;
            debug!("Extracted {} new clauses in {:.1}s", num_new, time_extract.as_secs_f64());
            debug!(
                "So far total {} clauses, total spent {:.3}s for extraction",
                all_clauses.len(),
                total_time_extract.as_secs_f64()
            );

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
                        if let Some(f) = &mut &mut file_derived_clauses {
                            write_clause(f, &lemma)?;
                        }
                        searcher.solver.add_clause(&lemma);
                        all_derived_clauses.push(lemma);
                        num_added += 1;
                    }
                }
                debug!("Added {} new lemmas from cores", num_added);
            }

            cubes_product = cubes_product
                .into_iter()
                .enumerate()
                .filter_map(|(i, cube)| {
                    if remaining_cubes.contains(&i) || indet_cubes.contains(&i) {
                        Some(cube)
                    } else {
                        None
                    }
                })
                .collect();
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

                if final_model.is_some() {
                    return false;
                }

                if !in_budget {
                    return true;
                }

                let num_conflicts = searcher.solver.0.conflicts() as u64;
                if num_conflicts > num_conflicts_limit {
                    debug!("Budget exhausted");
                    in_budget = false;
                }

                if !in_budget {
                    return true;
                }

                searcher.solver.0.reset_assumptions();
                for &lit in cube.iter() {
                    searcher.solver.assume(lit);
                }
                searcher.solver.0.limit("conflicts", args.num_conflicts as i32);
                let time_solve = Instant::now();
                let res = searcher.solver.solve();
                let time_solve = time_solve.elapsed();

                match res {
                    SolveResponse::Interrupted => true,
                    SolveResponse::Unsat => {
                        if args.compute_cores {
                            let mut core = Vec::new();
                            for &lit in cube {
                                if searcher.solver.failed(lit) {
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
                        let model = (1..=searcher.solver.0.vars())
                            .map(|i| {
                                let v = Var::from_external(i as u32);
                                match searcher.solver.0.val(i as i32).unwrap() {
                                    LitValue::True => Lit::new(v, false),
                                    LitValue::False => Lit::new(v, true),
                                }
                            })
                            .collect_vec();
                        final_model = Some(model);
                        // TODO: break out of the outer loop (currently not possible due to closure in retain)
                        false
                    }
                }
            });
            pb.finish_and_clear();

            if let Some(model) = final_model {
                return Ok(SolveResult::SAT(model));
            }

            // Populate the set of ALL clauses:
            debug!("Retrieving clauses from the solver...");
            let time_extract = Instant::now();
            let mut num_new = 0;
            for clause in searcher.solver.0.all_clauses_iter() {
                let mut clause = lits_from_external(clause);
                clause.sort_by_key(|lit| lit.inner());
                all_clauses.insert(clause);
                num_new += 1;
            }
            let time_extract = time_extract.elapsed();
            total_time_extract += time_extract;
            debug!("Extracted {} new clauses in {:.1}s", num_new, time_extract.as_secs_f64());
            debug!(
                "So far total {} clauses, total spent {:.3}s for extraction",
                all_clauses.len(),
                total_time_extract.as_secs_f64()
            );

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
        debug!(
            "Filtered {} down to {} cubes via solver in {:.1}s",
            num_cubes_before_filtering,
            cubes_product.len(),
            time_filter.as_secs_f64()
        );
        if let Some(f) = &mut file_results {
            writeln!(f, "{},limited,{}", run_number, cubes_product.len())?;
        }

        // Update the budget for filtering:
        budget_filter = (budget_filter as f64 * args.factor_budget_filter) as u64;

        if cubes_product.is_empty() {
            info!("No more cubes to solve after {} runs", run_number);

            info!("Just solving with {} conflicts budget...", budget_solve);
            searcher.solver.0.reset_assumptions();
            searcher.solver.0.limit("conflicts", budget_solve as i32);
            let time_solve = Instant::now();
            let res = searcher.solver.solve();
            let time_solve = time_solve.elapsed();
            match res {
                SolveResponse::Interrupted => {
                    info!("UNKNOWN in {:.1} s", time_solve.as_secs_f64());
                    // do nothing
                }
                SolveResponse::Unsat => {
                    info!("UNSAT in {:.1} s", time_solve.as_secs_f64());
                    return Ok(SolveResult::UNSAT);
                }
                SolveResponse::Sat => {
                    info!("SAT in {:.1} s", time_solve.as_secs_f64());
                    let model = (1..=searcher.solver.0.vars())
                        .map(|i| {
                            let v = Var::from_external(i as u32);
                            match searcher.solver.0.val(i as i32).unwrap() {
                                LitValue::True => Lit::new(v, false),
                                LitValue::False => Lit::new(v, true),
                            }
                        })
                        .collect_vec();
                    return Ok(SolveResult::SAT(model));
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
                "Derived {} clauses ({} units, {} binary, {} ternary, {} other) for {} cubes in {:.1}s",
                derived_clauses.len(),
                derived_clauses.iter().filter(|c| c.len() == 1).count(),
                derived_clauses.iter().filter(|c| c.len() == 2).count(),
                derived_clauses.iter().filter(|c| c.len() == 3).count(),
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
                "Derived {} new clauses ({} units, {} binary, {} ternary, {} other)",
                new_clauses.len(),
                new_clauses.iter().filter(|c| c.len() == 1).count(),
                new_clauses.iter().filter(|c| c.len() == 2).count(),
                new_clauses.iter().filter(|c| c.len() == 3).count(),
                new_clauses.iter().filter(|c| c.len() > 2).count()
            );
            debug!("[{}]", new_clauses.iter().map(|c| DisplaySlice(c)).join(", "));

            debug!("Adding {} new derived clauses to the solver...", new_clauses.len());
            for lemma in new_clauses {
                searcher.solver.add_clause(&lemma);
            }

            debug!(
                "So far derived {} new clauses ({} units, {} binary, {} ternary, {} other)",
                all_derived_clauses.len(),
                all_derived_clauses.iter().filter(|c| c.len() == 1).count(),
                all_derived_clauses.iter().filter(|c| c.len() == 2).count(),
                all_derived_clauses.iter().filter(|c| c.len() == 3).count(),
                all_derived_clauses.iter().filter(|c| c.len() > 2).count()
            );
        };

        info!("Just solving with {} conflicts budget...", budget_solve);
        searcher.solver.0.reset_assumptions();
        searcher.solver.0.limit("conflicts", budget_solve as i32);
        let time_solve = Instant::now();
        let res = searcher.solver.0.solve().unwrap();
        let time_solve = time_solve.elapsed();
        match res {
            SolveResponse::Interrupted => {
                info!("UNKNOWN in {:.1} s", time_solve.as_secs_f64());
                // do nothing
            }
            SolveResponse::Unsat => {
                info!("UNSAT in {:.1} s", time_solve.as_secs_f64());
                return Ok(SolveResult::UNSAT);
            }
            SolveResponse::Sat => {
                info!("SAT in {:.1} s", time_solve.as_secs_f64());
                let model = (1..=searcher.solver.0.vars())
                    .map(|i| {
                        let v = Var::from_external(i as u32);
                        match searcher.solver.0.val(i as i32).unwrap() {
                            LitValue::True => Lit::new(v, false),
                            LitValue::False => Lit::new(v, true),
                        }
                    })
                    .collect_vec();
                return Ok(SolveResult::SAT(model));
            }
        }
        searcher.solver.0.internal_backtrack(0);

        // Update the budget for solving:
        budget_solve = (budget_solve as f64 * args.factor_budget_solve) as u64;

        let time_run = time_run.elapsed();
        info!("Done run {} in {:.1}s", run_number, time_run.as_secs_f64());
    }
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

    match solve(args)? {
        SolveResult::UNSAT => {
            info!("UNSAT in {:.3} s", start_time.elapsed().as_secs_f64());
            println!("s UNSATISFIABLE");
            std::process::exit(20);
        }
        SolveResult::SAT(model) => {
            info!("SAT in {:.3} s", start_time.elapsed().as_secs_f64());
            println!("s SATISFIABLE");
            let mut line = "v".to_string();
            for &lit in model.iter() {
                if line.len() + format!(" {}", lit).len() > 100 {
                    println!("{}", line);
                    line = "v".to_string();
                }
                write!(line, " {}", lit)?;
            }
            write!(line, " 0")?;
            println!("{}", line);
            std::process::exit(10);
        }
        SolveResult::UNKNOWN => {
            info!("INDET in {:.3} s", start_time.elapsed().as_secs_f64());
            println!("s UNKNOWN");
        }
    }

    Ok(())
}
