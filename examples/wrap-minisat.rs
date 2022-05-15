use color_eyre::eyre::Result;

use sat_nexus_test_utils::run_test_1;
use sat_nexus_wrappers::minisat::MiniSatSolver;

fn main() -> Result<()> {
    color_eyre::install()?;

    let solver = MiniSatSolver::new();
    run_test_1(solver)?;

    Ok(())
}
