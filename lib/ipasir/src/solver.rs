use std::convert::TryInto;
use std::fmt;

use super::api::*;
use super::ffi::*;
use super::types::*;

pub struct IpasirSolver {
    ffi: &'static IpasirFFI,
    ptr: IpasirPtr,
}

impl IpasirSolver {
    pub fn new_custom(ffi: &'static IpasirFFI) -> Self {
        Self {
            ffi,
            ptr: unsafe { ffi.ipasir_init() },
        }
    }

    pub fn new_minisat() -> Self {
        Self::new_custom(IpasirFFI::instance_minisat())
    }
    pub fn new_glucose() -> Self {
        Self::new_custom(IpasirFFI::instance_glucose())
    }
    pub fn new_cadical() -> Self {
        Self::new_custom(IpasirFFI::instance_cadical())
    }
}

impl Drop for IpasirSolver {
    fn drop(&mut self) {
        self.release();
    }
}

impl From<&'static IpasirFFI> for IpasirSolver {
    fn from(ffi: &'static IpasirFFI) -> Self {
        Self::new_custom(ffi)
    }
}

impl fmt::Display for IpasirSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.signature())
    }
}

impl Ipasir for IpasirSolver {
    fn ffi(&self) -> &'static IpasirFFI {
        self.ffi
    }

    fn ptr(&self) -> IpasirPtr {
        self.ptr
    }

    fn reset(&mut self) {
        self.release();
        self.ptr = self.ffi.init();
    }

    fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe { self.ffi.ipasir_release(self.ptr) }
            self.ptr = std::ptr::null_mut();
        }
    }
}

impl IpasirSolver {
    pub fn add_clause<I, L>(&self, lits: I)
    where
        I: IntoIterator<Item = L>,
        L: Into<Lit>,
    {
        for lit in lits.into_iter() {
            self.add(lit.into().into());
        }
        self.add(0);
    }

    pub fn try_add_clause<I, L>(&self, lits: I) -> Result<(), <L as TryInto<Lit>>::Error>
    where
        I: IntoIterator<Item = L>,
        L: TryInto<Lit>,
    {
        let lits: Vec<Lit> = lits
            .into_iter()
            .map(|x| x.try_into())
            .collect::<Result<_, _>>()?;
        self.add_clause(lits);
        Ok(())
    }
}
