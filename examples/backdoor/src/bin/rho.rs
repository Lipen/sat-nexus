use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use log::{debug, info};

use backdoor::solver::Solver;
use backdoor::utils::*;

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

    /// Deduplicate input variables.
    #[arg(long)]
    dedup: bool,

    /// Sort input variables.
    #[arg(long)]
    sort: bool,

    /// Use tree-based propcheck.
    #[arg(long)]
    tree: bool,

    /// Freeze all variables.
    #[arg(long)]
    freeze_all: bool,

    /// Freeze backdoor variables.
    #[arg(long)]
    freeze: bool,

    /// Budget (in conflicts) for pre-solve.
    #[arg(long, value_name = "INT", default_value_t = 0)]
    budget_presolve: u64,

    /// Show hard cubes.
    #[arg(long)]
    show_hard: bool,

    /// Show easy cubes.
    #[arg(long)]
    show_easy: bool,
}

fn _main(args: &Cli) -> color_eyre::Result<()> {
    // Parse input variables:
    let vars = parse_comma_separated_intervals(&args.vars_str);
    let vars: Vec<Var> = vars.into_iter().map(|i| Var::from_external(i as u32)).collect();
    info!("Got {} variables: {}", vars.len(), display_slice(&vars));

    assert!(!vars.is_empty(), "no variables");

    // Deduplicate variables:
    let vars = if args.dedup {
        vars.into_iter().unique().collect()
    } else {
        assert!(vars.iter().all_unique(), "duplicate variables");
        vars
    };

    // Sort variables:
    let vars = if args.sort {
        let mut vars = vars;
        vars.sort();
        vars
    } else {
        vars
    };

    assert!(vars.len() < 64, "too many variables");

    // Initialize SAT solver:
    info!("Initializing SAT solver...");
    let cadical = Cadical::new();
    info!("Reading CNF from '{}'...", args.path_cnf.display());
    for clause in parse_dimacs(&args.path_cnf) {
        cadical.add_clause(lits_to_external(&clause));
    }
    info!("vars = {}, clauses = {}", cadical.vars(), cadical.irredundant());
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

    let solver = Solver::new(cadical);
    debug!("solver = {:?}", solver);

    if args.budget_presolve > 0 {
        info!("Pre-solving with {} conflicts budget...", args.budget_presolve);
        solver.0.limit("conflicts", args.budget_presolve as i32);
        let time_solve = Instant::now();
        let res = solver.0.solve()?;
        let time_solve = time_solve.elapsed();
        match res {
            SolveResponse::Interrupted => {
                info!("UNKNOWN in {:.1} s", time_solve.as_secs_f64());
                solver.0.internal_backtrack(0);
                // do nothing more
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

    // Ensure all variables are active:
    for &v in vars.iter() {
        assert!(solver.is_active(v), "var {} is not active", v);
    }

    // Compute rho:
    info!("Computing rho for {} vars: {}", vars.len(), display_slice(&vars));
    let num_total = 1u64 << vars.len();
    let num_hard = if args.tree {
        info!("Using tree-based propcheck");
        let vars_external = vars_to_external(&vars);
        let mut hard = Vec::new();
        let mut easy = Vec::new();
        let num_hard = solver
            .0
            .propcheck_all_tree_via_internal(&vars_external, 0, Some(&mut hard), Some(&mut easy));
        info!("hard = {}", hard.len());
        if args.show_hard {
            for (i, cube) in hard.iter().enumerate() {
                info!("hard {}/{} = {}", i + 1, hard.len(), display_slice(&cube));
            }
        }
        info!("easy = {}", easy.len());
        if args.show_easy {
            for (i, cube) in easy.iter().enumerate() {
                solver.0.propcheck(&cube, false, false, true);
                let core = solver.0.propcheck_get_core();
                info!(
                    "easy {}/{} = {}, core = {}",
                    i + 1,
                    easy.len(),
                    display_slice(&cube),
                    display_slice(&core)
                );
            }
        }
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
        if args.show_hard {
            for (i, cube) in hard.iter().enumerate() {
                info!("hard {}/{} = {}", i + 1, hard.len(), display_slice(&cube));
            }
        }
        info!("easy = {}", easy.len());
        if args.show_easy {
            for (i, cube) in easy.iter().enumerate() {
                solver.propcheck_save_core(&cube);
                let core = solver.0.propcheck_get_core();
                info!(
                    "easy {}/{} = {}, core = {}",
                    i + 1,
                    easy.len(),
                    display_slice(&cube),
                    display_slice(&core)
                );
            }
        }
        hard.len() as u64
    };
    let rho = 1.0 - (num_hard as f64 / num_total as f64);
    info!("rho = {:.3} ({} / {})", rho, num_total - num_hard, num_total);

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
