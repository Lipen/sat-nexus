#![allow(unused)]

use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

use derive_more::Deref;
use ndarray::{Array, Array1, ArrayD};
use type_map::TypeMap;

use nexus_sat::context::Context;

mod ipasir;

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    use nexus_sat::solver::wrap::WrappedIpasirSolver;
    use nexus_sat::solver::GenericSolver;

    let mut solver = WrappedIpasirSolver::new_cadical();
    println!("Solver signature: {}", solver.signature());

    solver.add_clause(&[1, 2]);
    solver.add_clause(&[3, 4]);
    solver.add_clause(&[-1, -2]);
    solver.add_clause(&[-3, -4]);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    solver.assume(1);
    solver.assume(2);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    // solver.assume(0);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    Ok(())
}
