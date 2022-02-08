use crate::ffi::*;
use crate::types::*;

pub trait Ipasir {
    fn ffi(&self) -> &'static IpasirFFI;
    fn ptr(&self) -> IpasirPtr;

    fn reset(&mut self);
    fn release(&mut self);

    fn signature(&self) -> &'static str {
        self.ffi().signature()
    }

    fn add(&self, lit_or_zero: i32) {
        unsafe { self.ffi().ipasir_add(self.ptr(), lit_or_zero) }
    }

    fn assume(&self, lit: Lit) {
        unsafe { self.ffi().ipasir_assume(self.ptr(), lit.into()) }
    }

    fn solve(&self) -> crate::Result<SolveResponse> {
        match unsafe { self.ffi().ipasir_solve(self.ptr()) } {
            0 => Ok(SolveResponse::Interrupted),
            10 => Ok(SolveResponse::Sat),
            20 => Ok(SolveResponse::Unsat),
            invalid => Err(SolverError::InvalidResponseSolve { value: invalid }),
        }
    }

    fn val(&self, lit: Lit) -> crate::Result<LitValue> {
        match unsafe { self.ffi().ipasir_val(self.ptr(), lit.into()) } {
            0 => Ok(LitValue::DontCare),
            p if p == lit.get() => Ok(LitValue::True),
            n if n == -lit.get() => Ok(LitValue::False),
            invalid => Err(SolverError::InvalidResponseVal { lit, value: invalid }),
        }
    }

    fn failed(&self, lit: Lit) -> crate::Result<bool> {
        match unsafe { self.ffi().ipasir_failed(self.ptr(), lit.into()) } {
            0 => Ok(true),
            1 => Ok(false),
            invalid => Err(SolverError::InvalidResponseFailed { lit, value: invalid }),
        }
    }
}
