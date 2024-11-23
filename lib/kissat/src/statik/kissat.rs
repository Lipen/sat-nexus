use std::fmt::{Debug, Display, Formatter};

use ffi_utils::cstr2str;

pub use crate::common::*;

use super::ffi::*;

/// Kissat solver.
pub struct Kissat {
    ptr: KissatPtr,
}

impl Kissat {
    pub fn new() -> Self {
        let ptr = unsafe { kissat_init() };
        Self { ptr }
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

/// Kissat IPASIR interface.
impl Kissat {
    pub fn signature(&self) -> &'static str {
        unsafe { cstr2str(kissat_signature()) }
    }

    pub fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe { kissat_release(self.ptr) }
            self.ptr = std::ptr::null_mut();
        }
    }

    pub fn add(&self, lit_or_zero: i32) {
        unsafe { kissat_add(self.ptr, lit_or_zero) }
    }

    pub fn solve(&self) -> SolveResponse {
        match unsafe { kissat_solve(self.ptr) } {
            0 => SolveResponse::Interrupted,
            10 => SolveResponse::Sat,
            20 => SolveResponse::Unsat,
            invalid => panic!("Invalid response from 'kissat_solve': {}", invalid),
        }
    }

    pub fn value(&self, lit: i32) -> LitValue {
        debug_assert_ne!(lit, 0, "Literal must be non-zero");
        match unsafe { kissat_value(self.ptr, lit) } {
            0 => LitValue::Any,
            p if p == lit => LitValue::True,
            n if n == -lit => LitValue::False,
            invalid => panic!("Invalid response from 'kissat_value(lit = {})': {}", lit, invalid),
        }
    }

    // TODO: set_terminate
}

/// Kissat additional API.
impl Kissat {
    pub fn terminate(&self) {
        unsafe { kissat_terminate(self.ptr) }
    }

    pub fn reserve(&self, max_var: i32) {
        unsafe { kissat_reserve(self.ptr, max_var) }
    }

    pub fn get_option(&self, name: &str) -> i32 {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe { kissat_get_option(self.ptr, name.as_ptr()) }
    }

    pub fn set_option(&self, name: &str, value: i32) -> i32 {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe { kissat_set_option(self.ptr, name.as_ptr(), value) }
    }

    pub fn has_configuration(name: &str) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe { kissat_has_configuration(name.as_ptr()) != 0 }
    }

    pub fn set_configuration(&self, name: &str) -> bool {
        let name = std::ffi::CString::new(name).unwrap();
        unsafe { kissat_set_configuration(self.ptr, name.as_ptr()) != 0 }
    }

    pub fn set_conflict_limit(&self, limit: u32) {
        unsafe { kissat_set_conflict_limit(self.ptr, limit) }
    }

    pub fn set_decision_limit(&self, limit: u32) {
        unsafe { kissat_set_decision_limit(self.ptr, limit) }
    }

    pub fn print_statistics(&self) {
        unsafe { kissat_print_statistics(self.ptr) }
    }
}

/// Kissat build information.
impl Kissat {
    pub fn id(&self) -> &'static str {
        unsafe { cstr2str(kissat_id()) }
    }

    pub fn version(&self) -> &'static str {
        unsafe { cstr2str(kissat_version()) }
    }

    pub fn compiler(&self) -> &'static str {
        unsafe { cstr2str(kissat_compiler()) }
    }
}

/// Extra methods.
impl Kissat {
    pub fn reset(&mut self) {
        self.release();
        self.ptr = unsafe { kissat_init() };
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
