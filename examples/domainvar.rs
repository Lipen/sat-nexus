use ndarray::ArrayD;

use sat_nexus_core::context::Context;
use sat_nexus_core::domainvar::DomainVar;
use sat_nexus_core::solver::ext::SolverExt;
use sat_nexus_core::solver::*;
use sat_nexus_wrappers::cadical_dynamic::CadicalDynamicSolver;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut solver = CadicalDynamicSolver::new();
    let mut context = Context::new();

    let num_states = 5;
    let num_trans = 3;
    let myvar = solver.new_domain_var_array_dyn([num_states, num_trans], |_| 0..=num_states);
    println!("myvar (Debug):\n{:?}", myvar);
    context.insert(myvar);

    let myvar = context.get::<ArrayD<DomainVar<usize>>>()?;
    println!("myvar:\n{}", myvar);

    solver.add_clause([myvar[[0, 0]].eq(3)]);
    solver.add_unit(myvar[[0, 1]].eq(2));
    solver.add_unit(myvar[[0, 2]].eq(4));
    solver.add_unit(myvar[[2, 1]].eq(2));
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    println!("model:\n{}", myvar.map(|x| x.eval(&solver)));

    Ok(())
}
