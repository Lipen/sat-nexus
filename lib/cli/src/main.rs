use std::fmt::Display;
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::bail;
use elapsed::measure_time;
use itertools::Itertools;
use log::info;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

use sat_nexus_core::cnf::Cnf;
use sat_nexus_core::solver::simple::BoxDynSimpleSolverExt;
use sat_nexus_core::solver::simple::SimpleSolver;
use sat_nexus_core::solver::Solver;
use sat_nexus_core::utils::bootstrap_solver_from_cnf;
use sat_nexus_wrappers::cadical::CadicalSolver;
use sat_nexus_wrappers::dispatch::DispatchSolver;
use sat_nexus_wrappers::minisat::MiniSatSolver;

mod parsing;

#[derive(Parser, Debug)]
#[clap(author, version)]
struct Cli {
    /// CNF file
    #[clap(parse(from_os_str), value_name = "FILE")]
    cnf: PathBuf,

    /// SAT solver
    #[clap(short, long, default_value = "cadical")]
    solver: String,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto)?;

    let args = Cli::parse();
    info!("args = {:?}", args);
    info!("args.cnf = {}", args.cnf.display());
    info!("args.solver = {}", args.solver);

    let mut solver = DispatchSolver::new_delegate_wrap(CadicalSolver::new());
    info!("solver1 = {}", solver);
    solver.reset();

    let mut solver: Box<dyn SimpleSolver> = match args.solver.to_ascii_lowercase().as_str() {
        "minisat" => Box::new(MiniSatSolver::new()),
        "cadical" => Box::new(CadicalSolver::new()),
        _ => bail!("Bad solver '{}'", &args.solver),
    };
    info!("solver2 = {}", solver.display());
    solver.reset();

    let mut solver = DispatchSolver::by_name(&args.solver);
    info!("solver3 = {}", solver);
    solver.reset();

    run(args, solver)
}

fn run<S>(args: Cli, mut solver: S) -> color_eyre::Result<()>
where
    S: Solver + Display,
{
    info!("solver = {}", solver);

    let cnf = Cnf::from_file(&args.cnf);
    info!("cnf.max_var = {}", cnf.max_var);
    info!("cnf = {}", cnf);

    bootstrap_solver_from_cnf(&mut solver, &cnf);

    info!("Solving...");
    let (elapsed, result) = measure_time(|| solver.solve());
    info!("{} in {}", result, elapsed);

    let model = (1..=solver.num_vars()).map(|i| solver.value(i)).collect_vec();
    let model_string = model.iter().map(|x| if x.bool() { "1" } else { "0" }).join("");
    info!("model: {}", model_string);

    Ok(())
}
