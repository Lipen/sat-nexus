use std::fmt;

use snafu::Snafu;

use super::ffi::*;
use super::Lit;

pub type Result<T, E = SolverError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[allow(clippy::enum_variant_names)]
pub enum SolverError {
    #[snafu(display("Invalid response from `solve()`: {}", value))]
    InvalidResponseSolve { value: i32 },

    #[snafu(display("Invalid response from `val({})`: {}", lit, value))]
    InvalidResponseVal { lit: Lit, value: i32 },

    #[snafu(display("Invalid response from `failed({})`: {}", lit, value))]
    InvalidResponseFailed { lit: Lit, value: i32 },
}

/// Possible responses from a call to `ipasir_solve`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SolveResponse {
    /// The solver found the input to be satisfiable.
    Sat = 10,
    /// The solver found the input to be unsatisfiable.
    Unsat = 20,
    /// The solver was interrupted.
    Interrupted = 0,
}

/// The assignment of a literal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LitValue {
    /// Any assignment is okay.
    DontCare,
    /// The literal is `true`.
    True,
    /// The literal is `false`.
    False,
}

impl LitValue {
    pub fn bool(&self) -> bool {
        match self {
            LitValue::True => true,
            LitValue::False => false,
            LitValue::DontCare => panic!("DontCare can't be converted to bool!"),
        }
    }
}

impl fmt::Display for LitValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LitValue::*;
        match self {
            DontCare => write!(f, "X"),
            True => write!(f, "1"),
            False => write!(f, "0"),
        }
    }
}

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

    fn solve(&self) -> Result<SolveResponse> {
        match unsafe { self.ffi().ipasir_solve(self.ptr()) } {
            0 => Ok(SolveResponse::Interrupted),
            10 => Ok(SolveResponse::Sat),
            20 => Ok(SolveResponse::Unsat),
            invalid => Err(SolverError::InvalidResponseSolve { value: invalid }),
        }
    }

    fn val(&self, lit: Lit) -> Result<LitValue> {
        match unsafe { self.ffi().ipasir_val(self.ptr(), lit.into()) } {
            0 => Ok(LitValue::DontCare),
            p if p == lit.get() => Ok(LitValue::True),
            n if n == -lit.get() => Ok(LitValue::False),
            invalid => Err(SolverError::InvalidResponseVal { lit, value: invalid }),
        }
    }

    fn failed(&self, lit: Lit) -> Result<bool> {
        match unsafe { self.ffi().ipasir_failed(self.ptr(), lit.into()) } {
            0 => Ok(true),
            1 => Ok(false),
            invalid => Err(SolverError::InvalidResponseFailed { lit, value: invalid }),
        }
    }
}
