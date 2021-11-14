use std::rc::{Rc, Weak};

use color_eyre::eyre::Result;

use sat_nexus::lit::Lit;
use sat_nexus::solver::wrap_ipasir::WrappedIpasirSolver;
use sat_nexus::solver::Solver;
use std::cell::RefCell;

struct Store<S> {
    weak: Weak<RefCell<S>>,
}

impl<S> Store<S>
where
    S: Solver,
{
    fn declare_something(&mut self) -> Lit {
        let shared = self.weak.upgrade().unwrap();
        let res = shared.borrow_mut().new_var();
        res
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    let solver = WrappedIpasirSolver::new_cadical();
    let shared_solver = Rc::new(RefCell::new(solver));
    // let solver = shared_solver.borrow_mut();

    println!("Solver signature: {}", shared_solver.borrow().signature());

    let mut store = Store {
        weak: Rc::downgrade(&shared_solver),
    };
    let v = store.declare_something();
    println!("new var = {}", v);

    Ok(())
}
