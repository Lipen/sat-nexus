use sat_nexus_test_utils::run_test_1;
use sat_nexus_wrappers::minisat_dynamic::MiniSatDynamicSolver;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let solver = MiniSatDynamicSolver::new();
    run_test_1(solver)?;

    Ok(())
}
