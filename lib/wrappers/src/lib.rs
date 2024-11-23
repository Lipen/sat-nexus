#[path = "wrap_ipasir.rs"]
pub mod ipasir;

#[path = "wrap_cadical.rs"]
pub mod cadical;

#[cfg(feature = "cadical-static")]
#[path = "wrap_cadical-static.rs"]
pub mod cadical_static;

#[path = "wrap_minisat.rs"]
pub mod minisat;

#[path = "wrap_kissat.rs"]
pub mod kissat;

#[cfg(feature = "kissat-static")]
#[path = "wrap_kissat-static.rs"]
pub mod kissat_static;

#[path = "wrap_simple-sat.rs"]
pub mod simplesat;

pub mod dispatch;
