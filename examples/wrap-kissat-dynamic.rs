use sat_nexus_test_utils::run_test_1;
use sat_nexus_wrappers::kissat_dynamic::KissatDynamicSolver;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let solver = KissatDynamicSolver::new();
    run_test_1(solver)?;

    Ok(())
}
