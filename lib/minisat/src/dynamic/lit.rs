pub use crate::common::Lit;

use super::ffi::bindings::minisat_Lit;

pub(crate) fn lit_from_c(lit: minisat_Lit) -> Lit {
    Lit::new(lit as _)
}

pub(crate) fn lit_to_c(lit: Lit) -> minisat_Lit {
    lit.get() as _
}
