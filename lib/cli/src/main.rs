use std::fmt::Display;
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::bail;
use elapsed::measure_time;
use itertools::Itertools;
use log::info;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

use sat_nexus_core::cnf::Cnf;
use sat_nexus_core::solver::simple::SimpleSolver;
use sat_nexus_core::solver::{SolveResponse, Solver};
use sat_nexus_core::utils::bootstrap_solver_from_cnf;
use sat_nexus_wrappers::cadical_dynamic::CadicalDynamicSolver;
use sat_nexus_wrappers::dispatch::DispatchSolver;
use sat_nexus_wrappers::kissat_dynamic::KissatDynamicSolver;
use sat_nexus_wrappers::minisat_dynamic::MiniSatDynamicSolver;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// CNF file
    #[arg(value_name = "FILE")]
    cnf: PathBuf,

    /// SAT solver
    #[arg(short, long, default_value = "cadical")]
    solver: String,
}

#[allow(dead_code)]
fn get_solver1() -> DispatchSolver {
    DispatchSolver::new_delegate_wrap(CadicalDynamicSolver::new())
}

#[allow(dead_code)]
fn get_solver2(name: &str) -> color_eyre::Result<Box<dyn SimpleSolver>> {
    let solver: Box<dyn SimpleSolver> = match name.to_ascii_lowercase().as_str() {
        "minisat" => MiniSatDynamicSolver::new().into(),
        "cadical" => CadicalDynamicSolver::new().into(),
        "kissat" => KissatDynamicSolver::new().into(),
        _ => bail!("Bad solver '{}'", name),
    };
    Ok(solver)
}

#[allow(dead_code)]
fn get_solver3(name: &str) -> DispatchSolver {
    DispatchSolver::by_name(name)
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)?;

    let args = Cli::parse();
    info!("args = {:?}", args);
    info!("args.cnf = {}", args.cnf.display());
    info!("args.solver = {}", args.solver);

    let solver = get_solver3(&args.solver);

    run(args, solver)
}

fn run<S>(args: Cli, mut solver: S) -> color_eyre::Result<()>
where
    S: Solver + Display,
{
    info!("solver = {}", solver);

    let cnf = Cnf::from_file(args.cnf);
    info!("cnf.max_var = {}", cnf.max_var);
    info!("cnf.clauses = {}", cnf.clauses.len());

    bootstrap_solver_from_cnf(&mut solver, &cnf);

    info!("Solving...");
    let (elapsed, result) = measure_time(|| solver.solve());
    info!("{} in {}", result, elapsed);

    if result == SolveResponse::Sat {
        let model = (1..=solver.num_vars()).map(|i| solver.value(i as i32)).collect_vec();
        let model_string = model.iter().map(|x| if x.bool() { "1" } else { "0" }).join("");
        info!("model: {}", model_string);
    }

    Ok(())
}
