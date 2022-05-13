use color_eyre::eyre::Result;

use sat_nexus::core::solver::delegate::DelegateSolver;
use sat_nexus::core::solver::mock::MockSolver;
use sat_nexus_test_utils::run_test_1;

fn main() -> Result<()> {
    color_eyre::install()?;

    let solver = DelegateSolver::new(MockSolver::new());
    run_test_1(solver)?;

    Ok(())
}
