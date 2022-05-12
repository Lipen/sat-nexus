#[path = "wrap_ipasir.rs"]
pub mod ipasir;

#[path = "wrap_cadical.rs"]
pub mod cadical;

#[path = "wrap_minisat.rs"]
pub mod minisat;

pub mod delegate;
pub mod dispatch;

pub mod wrap_cadical_simple;
pub mod wrap_ipasir_simple;
pub mod wrap_minisat_simple;
