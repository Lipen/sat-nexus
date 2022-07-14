use sat_nexus_test_utils::run_test_1;
use sat_nexus_wrappers::cadical::CadicalSolver;
use sat_nexus_wrappers::dispatch::DispatchSolver;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let solver = DispatchSolver::new_delegate_wrap(CadicalSolver::new());
    run_test_1(solver)?;

    Ok(())
}
