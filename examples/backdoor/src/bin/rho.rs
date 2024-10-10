use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};

use backdoor::solver::Solver;
use backdoor::utils::parse_comma_separated_intervals;

use cadical::statik::Cadical;
use cadical::SolveResponse;
use simple_sat::lit::Lit;
use simple_sat::utils::{display_slice, parse_dimacs};
use simple_sat::var::Var;

// Run this example:
// cargo run --release -p backdoor --bin rho -- data/mult/lec_CvK_12.cnf --vars 1-16 --tree

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// Input file with CNF in DIMACS format.
    #[arg(value_name = "CNF")]
    path_cnf: PathBuf,

    /// Comma-separated list of variables (1-based indices).
    #[arg(long = "vars", value_name = "INT...")]
    vars_str: String,

    /// Use tree-based propcheck.
    #[arg(long)]
    tree: bool,

    // /// Do derive clauses from backdoor?
    // #[arg(long)]
    // derive: bool,
    //
    // /// Derive ternary clauses.
    // #[arg(long)]
    // derive_ternary: bool,
    //
    // /// Do probe variables?
    // #[arg(long)]
    // probe: bool,
    //
    /// Freeze all variables.
    #[arg(long)]
    freeze_all: bool,

    /// Freeze backdoor variables.
    #[arg(long)]
    freeze: bool,

    // /// Do compute cores for easy tasks and invalid cubes.
    // #[arg(long)]
    // compute_cores: bool,
    /// Budget (in conflicts) for pre-solve.
    #[arg(long, value_name = "INT", default_value_t = 0)]
    budget_presolve: u64,
}

fn _main(args: &Cli) -> color_eyre::Result<()> {
    // Parse input variables:
    let vars = parse_comma_separated_intervals(&args.vars_str);
    let vars: Vec<Var> = vars.into_iter().map(|i| Var::from_external(i as u32)).collect();
    info!("Got {} variables: {}", vars.len(), display_slice(&vars));

    assert!(!vars.is_empty());
    assert!(vars.len() < 64, "too many variables");

    // Initialize SAT solver:
    info!("Initializing SAT solver...");
    let cadical = Cadical::new();
    for clause in parse_dimacs(&args.path_cnf) {
        cadical.add_clause(clause.into_iter().map(|lit| lit.to_external()));
    }
    if args.freeze_all {
        info!("Freezing all {} variables...", cadical.vars());
        for i in 0..cadical.vars() {
            let lit = (i + 1) as i32;
            cadical.freeze(lit)?;
        }
    } else if args.freeze {
        info!("Freezing {} backdoor variables...", vars.len());
        for var in vars.iter() {
            let lit = var.to_external() as i32;
            cadical.freeze(lit)?;
        }
    }
    cadical.limit("conflicts", 0);
    cadical.solve()?;

    let solver = Solver::new(cadical);
    debug!("solver = {:?}", solver);

    if args.budget_presolve > 0 {
        info!("Pre-solving with {} conflicts budget...", args.budget_presolve);
        solver.0.limit("conflicts", args.budget_presolve as i32);
        let time_solve = Instant::now();
        let res = solver.0.solve()?;
        let time_solve = time_solve.elapsed();
        solver.0.internal_backtrack(0);
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

        {
            let res = solver.0.internal_propagate();
            assert!(res);
        }
    }

    // // Probe variables:
    // if args.probe {
    //     info!("Probing variables...");
    //     let time_failed = Instant::now();
    //     let mut failed = Vec::new();
    //     for i in 1..solver.num_vars() {
    //         let var = Var::from_external(i as u32);
    //         if !solver.is_active(var) {
    //             continue;
    //         }
    //         let pos_lit = Lit::new(var, false);
    //         let (res, _) = solver.propcheck(&[pos_lit]);
    //         if !res {
    //             info!("failed literal {}", pos_lit);
    //             solver.add_clause(&[-pos_lit]);
    //             failed.push(pos_lit);
    //         } else {
    //             let neg_lit = Lit::new(var, true);
    //             let (res, _) = solver.propcheck(&[neg_lit]);
    //             if !res {
    //                 info!("failed literal {}", neg_lit);
    //                 solver.add_clause(&[-neg_lit]);
    //                 failed.push(neg_lit);
    //             } else {
    //                 // neither positive nor negative literal is failed
    //                 // info!("neither {} nor {} is failed", pos_lit, neg_lit);
    //             }
    //         }
    //     }
    //     let time_failed = time_failed.elapsed();
    //     debug!("Found {} failed literals in {:.3}s", failed.len(), time_failed.as_secs_f64());
    //     for &lit in failed.iter() {
    //         if !solver.is_active(lit.var()) {
    //             debug!("failed literal {} is not active anymore", lit);
    //         }
    //     }
    // }

    // Compute rho:
    info!("Computing rho for {} vars: {}", vars.len(), display_slice(&vars));
    let num_total = 1u64 << vars.len();
    let num_hard = if args.tree {
        info!("Using tree-based propcheck");
        let vars_external: Vec<i32> = vars.iter().map(|var| var.to_external() as i32).collect();
        let mut hard = Vec::new();
        let mut easy = Vec::new();
        let num_hard = solver
            .0
            .propcheck_all_tree_via_internal(&vars_external, 0, Some(&mut hard), Some(&mut easy));
        info!("hard = {}", hard.len());
        info!("easy = {}", easy.len());
        assert_eq!(num_hard, hard.len() as u64);
        num_hard
    } else {
        info!("Using naive propcheck");

        let mut hard = Vec::new();
        let mut easy = Vec::new();

        let pb = ProgressBar::new(num_total);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed}] [{bar:40.cyan/white}] {pos:>6}/{len} (ETA: {eta}) {msg}")?
                .progress_chars("#>-"),
        );
        pb.set_message("propcheck");
        let n = vars.len();
        for i in 0..num_total {
            pb.inc(1);
            let cube: Vec<Lit> = vars
                .iter()
                .enumerate()
                .map(|(j, &var)| {
                    let bit = (i >> (n - 1 - j)) & 1;
                    Lit::new(var, bit == 0)
                })
                .collect();
            // pb.println(format!("cube = {}", display_slice(&cube)));
            let (res, _) = solver.propcheck(&cube);
            if res {
                hard.push(cube);
            } else {
                // pb.println(format!("easy cube: {}", display_slice(&cube)));
                easy.push(cube);
            }
        }
        pb.finish_and_clear();

        info!("hard = {}", hard.len());
        info!("easy = {}", easy.len());
        hard.len() as u64
    };
    let rho = 1.0 - (num_hard as f64 / num_total as f64);
    info!("rho = {:.3} ({} / {})", rho, num_total - num_hard, num_total);

    // if args.compute_cores {
    //     let vars_external: Vec<i32> = backdoor.iter().map(|var| var.to_external() as i32).collect();
    //     for &v in vars_external.iter() {
    //         assert!(solver.0.is_active(v), "var {} is not active", v);
    //     }
    //     let mut hard = Vec::new();
    //     let mut easy = Vec::new();
    //     let res = searcher
    //         .solver
    //         .0
    //         .propcheck_all_tree_via_internal(&vars_external, 0, Some(&mut hard), Some(&mut easy));
    //     assert_eq!(hard.len(), res as usize);
    //
    //     let hard: Vec<Vec<Lit>> = hard.into_iter().map(clause_from_external).collect();
    //     debug!("Hard tasks: {}", hard.len());
    //     for (i, cube) in hard.iter().enumerate() {
    //         debug!("[{}/{}]: {}", i + 1, hard.len(), display_slice(cube));
    //     }
    //
    //     let easy: Vec<Vec<Lit>> = easy.into_iter().map(clause_from_external).collect();
    //     debug!("Easy tasks: {}", easy.len());
    //
    //     let mut easy_cores: Vec<Vec<Lit>> = Vec::new();
    //     for (i, cube) in easy.iter().enumerate() {
    //         let (res, _) = solver.propcheck_save_core(&cube);
    //         if res {
    //             panic!("Unexpected SAT on cube = {}", display_slice(&cube));
    //         } else {
    //             let core = solver.0.propcheck_get_core();
    //             assert!(!core.is_empty());
    //             let mut core = clause_from_external(core);
    //             core.sort_by_key(|lit| lit.inner());
    //             debug!(
    //                 "{}/{}: core = {} for cube = {}",
    //                 i + 1,
    //                 easy.len(),
    //                 display_slice(&core),
    //                 display_slice(cube)
    //             );
    //             assert_eq!(
    //                 core.last().unwrap(),
    //                 cube.last().unwrap(),
    //                 "core.last() = {}, cube.last() = {}",
    //                 core.last().unwrap(),
    //                 cube.last().unwrap()
    //             );
    //
    //             if false {
    //                 let lemma = core.iter().map(|&lit| -lit).collect_vec();
    //                 let lits = &lemma;
    //
    //                 solver.0.internal_backtrack(0);
    //
    //                 let res = solver.0.internal_propagate();
    //                 assert!(res);
    //
    //                 let lits = clause_to_external(lits).collect_vec();
    //                 if lits.len() >= 2 {
    //                     for lit in lits {
    //                         assert!(solver.0.is_active(lit), "lit {} is not active", lit);
    //                         solver.0.add_derived(lit);
    //                     }
    //                     solver.0.add_derived(0);
    //                 } else {
    //                     let lit = lits[0];
    //                     if solver.0.is_active(lit) {
    //                         solver.0.add_unit_clause(lit);
    //                         assert!(!solver.0.is_active(lit));
    //                     } else {
    //                         log::warn!("unit {} is not active", lit);
    //                     }
    //                 }
    //
    //                 let res = solver.0.internal_propagate();
    //                 assert!(res);
    //             }
    //
    //             easy_cores.push(core);
    //         }
    //     }
    //
    //     let easy_cores: HashSet<Vec<Lit>> = easy_cores.into_iter().collect();
    //     debug!("Unique easy cores: {}", easy_cores.len());
    //     for (i, core) in easy_cores.iter().enumerate() {
    //         debug!("[{}/{}]: {}", i + 1, easy_cores.len(), display_slice(core));
    //     }
    //     if args.add_cores {
    //         info!("Adding lemmas from {} cores...", easy_cores.len());
    //         for core in easy_cores.iter() {
    //             let lemma = core.iter().map(|&lit| -lit).collect_vec();
    //             solver.0.add_derived_clause(clause_to_external(&lemma));
    //         }
    //     }
    // }

    // // Derive clauses from the best backdoor:
    // if args.derive {
    //     // TODO: handle the case when `hard.len() == 1`
    //
    //     let time_derive = Instant::now();
    //     let derived_clauses = derive_clauses(&hard, args.derive_ternary);
    //     let time_derive = time_derive.elapsed();
    //     info!(
    //         "Derived {} clauses ({} units, {} binary, {} other) for backdoor in {:.1}s",
    //         derived_clauses.len(),
    //         derived_clauses.iter().filter(|c| c.len() == 1).count(),
    //         derived_clauses.iter().filter(|c| c.len() == 2).count(),
    //         derived_clauses.iter().filter(|c| c.len() > 2).count(),
    //         time_derive.as_secs_f64()
    //     );
    //     // debug!("[{}]", derived_clauses.iter().map(|c| display_slice(c)).join(", "));
    //
    //     // if args.probe_derived {
    //     //     info!("Checking (probing) derived clauses...");
    //     //     for clause in derived_clauses.iter() {
    //     //         let cube = clause.iter().map(|&lit| -lit).collect_vec();
    //     //         let (res, _) = solver.propcheck_save_core(&cube);
    //     //         if res {
    //     //             info!("clause {} is not RUP", display_slice(clause));
    //     //         } else {
    //     //             info!("clause {} has RUP", display_slice(clause));
    //     //         }
    //     //     }
    //     // }
    //
    //     let mut new_clauses = Vec::new();
    //     for mut lemma in derived_clauses {
    //         lemma.sort_by_key(|lit| lit.inner());
    //         if all_clauses.insert(lemma.clone()) {
    //             if let Some(f) = &mut file_derived_clauses {
    //                 write_clause(f, &lemma)?;
    //             }
    //             solver.add_clause(&lemma);
    //             new_clauses.push(lemma.clone());
    //             all_derived_clauses.push(lemma);
    //         }
    //     }
    //     solver.0.limit("conflicts", 0);
    //     solver.solve();
    //     debug!(
    //         "Derived {} new clauses ({} units, {} binary, {} other)",
    //         new_clauses.len(),
    //         new_clauses.iter().filter(|c| c.len() == 1).count(),
    //         new_clauses.iter().filter(|c| c.len() == 2).count(),
    //         new_clauses.iter().filter(|c| c.len() > 2).count()
    //     );
    //     // debug!("[{}]", new_clauses.iter().map(|c| display_slice(c)).join(", "));
    // }

    Ok(())
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let start_time = Instant::now();
    let args = Cli::parse();
    info!("args = {:?}", args);

    _main(&args)?;

    info!("All done in {:.3} s", start_time.elapsed().as_secs_f64());
    Ok(())
}
