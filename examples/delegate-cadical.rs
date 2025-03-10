use sat_nexus_core::solver::delegate::DelegateSolver;
use sat_nexus_test_utils::run_test_1;
use sat_nexus_wrappers::cadical_dynamic::CadicalDynamicSolver;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let solver = DelegateSolver::wrap(CadicalDynamicSolver::new());
    run_test_1(solver)?;

    Ok(())
}
