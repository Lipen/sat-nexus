use color_eyre::eyre::Result;

use sat_nexus_test_utils::run_test_1;
use sat_nexus_wrappers::dispatch::DispatchingSolver;

fn main() -> Result<()> {
    color_eyre::install()?;

    let solver = DispatchingSolver::new_cadical();
    run_test_1(solver)?;

    Ok(())
}