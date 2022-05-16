use std::convert::TryInto;
use std::fmt::{Debug, Display, Formatter};

use crate::ffi::*;
use crate::types::*;

pub struct Ipasir {
    ffi: &'static IpasirFFI,
    ptr: IpasirPtr,
}

impl Ipasir {
    pub fn new_custom(ffi: &'static IpasirFFI) -> Self {
        let ptr = ffi.init();
        Self { ffi, ptr }
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

impl Drop for Ipasir {
    fn drop(&mut self) {
        self.release();
    }
}

impl From<&'static IpasirFFI> for Ipasir {
    fn from(ffi: &'static IpasirFFI) -> Self {
        Self::new_custom(ffi)
    }
}

impl Debug for Ipasir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ipasir").field("ptr", &self.ptr).finish()
    }
}

impl Display for Ipasir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.signature())
    }
}

// IPASIR interface
impl Ipasir {
    pub fn reset(&mut self) {
        self.release();
        self.ptr = self.ffi.init();
    }

    pub fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe { self.ffi.ipasir_release(self.ptr) }
            self.ptr = std::ptr::null_mut();
        }
    }

    pub fn signature(&self) -> &'static str {
        self.ffi.signature()
    }

    pub fn add(&self, lit_or_zero: i32) {
        unsafe { self.ffi.ipasir_add(self.ptr, lit_or_zero) }
    }

    pub fn assume(&self, lit: Lit) {
        unsafe { self.ffi.ipasir_assume(self.ptr, lit.into()) }
    }

    pub fn solve(&self) -> Result<SolveResponse> {
        match unsafe { self.ffi.ipasir_solve(self.ptr) } {
            0 => Ok(SolveResponse::Interrupted),
            10 => Ok(SolveResponse::Sat),
            20 => Ok(SolveResponse::Unsat),
            invalid => Err(IpasirError::InvalidResponseSolve { value: invalid }),
        }
    }

    pub fn val(&self, lit: Lit) -> Result<LitValue> {
        match unsafe { self.ffi.ipasir_val(self.ptr, lit.into()) } {
            0 => Ok(LitValue::DontCare),
            p if p == lit.get() => Ok(LitValue::True),
            n if n == -lit.get() => Ok(LitValue::False),
            invalid => Err(IpasirError::InvalidResponseVal { lit, value: invalid }),
        }
    }

    pub fn failed(&self, lit: Lit) -> Result<bool> {
        match unsafe { self.ffi.ipasir_failed(self.ptr, lit.into()) } {
            0 => Ok(true),
            1 => Ok(false),
            invalid => Err(IpasirError::InvalidResponseFailed { lit, value: invalid }),
        }
    }
}

// Additional fluent interface
impl Ipasir {
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
        let lits: Vec<Lit> = lits.into_iter().map(|x| x.try_into()).collect::<Result<_, _>>()?;
        self.add_clause(lits);
        Ok(())
    }
}
