use std::fmt::{Debug, Display, Formatter};

use ffi_utils::cstr2str;

pub use crate::common::*;

use super::ffi::*;

/// Kissat solver.
pub struct Kissat {
    ffi: &'static KissatFFI,
    ptr: KissatPtr,
}

impl Kissat {
    pub fn new() -> Self {
        Self::new_custom(KissatFFI::instance())
    }

    pub fn new_custom(ffi: &'static KissatFFI) -> Self {
        Kissat {
            ffi,
            ptr: unsafe { ffi.kissat_init() },
        }
    }
}

impl Default for Kissat {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Kissat {
    fn drop(&mut self) {
        self.release()
    }
}

impl Debug for Kissat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Kissat").field("ptr", &self.ptr).finish()
    }
}

impl Display for Kissat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.signature())
    }
}

/// Kissat interface.
impl Kissat {
    pub fn signature(&self) -> &'static str {
        unsafe { cstr2str(self.ffi.kissat_signature()) }
    }

    pub fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe { self.ffi.kissat_release(self.ptr) }
            self.ptr = std::ptr::null_mut();
        }
    }

    pub fn add(&self, lit_or_zero: i32) {
        unsafe { self.ffi.kissat_add(self.ptr, lit_or_zero) }
    }

    pub fn solve(&self) -> SolveResponse {
        match unsafe { self.ffi.kissat_solve(self.ptr) } {
            0 => SolveResponse::Interrupted,
            10 => SolveResponse::Sat,
            20 => SolveResponse::Unsat,
            invalid => panic!("Invalid response from 'kissat_solve': {}", invalid),
        }
    }

    pub fn value(&self, lit: i32) -> LitValue {
        debug_assert_ne!(lit, 0, "Literal must be non-zero");
        match unsafe { self.ffi.kissat_value(self.ptr, lit) } {
            0 => LitValue::Any,
            p if p == lit => LitValue::True,
            n if n == -lit => LitValue::False,
            invalid => panic!("Invalid response from 'kissat_value(lit = {})': {}", lit, invalid),
        }
    }
}

/// Additional methods
impl Kissat {
    pub fn reset(&mut self) {
        self.release();
        self.ptr = unsafe { self.ffi.kissat_init() };
    }

    pub fn add_clause<I>(&self, lits: I)
    where
        I: IntoIterator,
        I::Item: Into<i32>,
    {
        for lit in lits.into_iter() {
            self.add(lit.into());
        }
        self.add(0);
    }
}
