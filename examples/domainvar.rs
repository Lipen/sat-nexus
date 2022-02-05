use color_eyre::eyre::Result;
use ndarray::ArrayD;

use sat_nexus::core::domainvar::DomainVar;
use sat_nexus::core::solver::Solver;
use sat_nexus::wrappers::ipasir::WrappedIpasirSolver;

fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    let mut solver = WrappedIpasirSolver::new_cadical();

    let num_states = 5;
    let num_trans = 3;
    // let myvar = solver.new_array(&[num_states, num_trans], |solver| {
    //     solver.new_domain_var(0..=num_states)
    // });
    let myvar = solver.new_domain_var_array_dyn([num_states, num_trans], |_| 0..=num_states);
    // .tap_mut(|it| it[[3, 0]].reverse_domain());
    println!("myvar (Debug):\n{:?}", myvar);

    let shared_context = solver.context();
    let mut context = shared_context.borrow_mut();
    context.insert(myvar);

    drop(context);
    // drop(shared_context);

    // let shared_context = solver.context();
    let context = shared_context.borrow();
    let myvar = context.extract::<ArrayD<DomainVar<usize>>>();
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
