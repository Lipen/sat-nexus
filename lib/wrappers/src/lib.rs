#[path = "wrap_ipasir.rs"]
pub mod ipasir;

#[path = "wrap_minisat-dynamic.rs"]
pub mod minisat_dynamic;

#[path = "wrap_cadical-dynamic.rs"]
pub mod cadical_dynamic;

#[cfg(feature = "cadical-static")]
#[path = "wrap_cadical-static.rs"]
pub mod cadical_static;

#[path = "wrap_kissat-dynamic.rs"]
pub mod kissat_dynamic;

#[cfg(feature = "kissat-static")]
#[path = "wrap_kissat-static.rs"]
pub mod kissat_static;

#[path = "wrap_simple-sat.rs"]
pub mod simplesat;

pub mod dispatch;
