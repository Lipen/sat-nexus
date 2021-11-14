use std::ffi::CStr;
use std::fmt;
use std::os::raw::c_int;

use super::ffi::*;

pub struct CadicalSolver2 {
    ffi: &'static CCadicalFFI,
    ptr: CCadicalPtr,
}

impl CadicalSolver2 {
    pub fn new() -> Self {
        Self::new_custom(CCadicalFFI::instance())
    }

    pub fn new_custom(ffi: &'static CCadicalFFI) -> Self {
        CadicalSolver2 {
            ffi,
            ptr: unsafe { ffi.ccadical_init() },
        }
    }
}

impl Default for CadicalSolver2 {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CadicalSolver2 {
    fn drop(&mut self) {
        self.release()
    }
}

impl fmt::Display for CadicalSolver2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.signature())
    }
}

/// Cadical interface methods.
impl CadicalSolver2 {
    pub fn signature(&self) -> &'static str {
        let c_chars = unsafe { self.ffi.ccadical_signature() };
        let c_str = unsafe { CStr::from_ptr(c_chars) };
        c_str
            .to_str()
            .expect("The implementation returned invalid UTF-8.")
    }

    pub fn reset(&mut self) {
        self.release();
        self.ptr = unsafe { self.ffi.ccadical_init() };
    }

    pub fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe { self.ffi.ccadical_release(self.ptr) }
            self.ptr = std::ptr::null_mut();
        }
    }

    pub fn add(&self, lit: c_int) {
        unsafe { self.ffi.ccadical_add(self.ptr, lit) }
    }

    pub fn assume(&self, lit: c_int) {
        unsafe { self.ffi.ccadical_assume(self.ptr, lit) }
    }

    pub fn solve(&self) -> i32 {
        unsafe { self.ffi.ccadical_solve(self.ptr) }
    }

    pub fn val(&self, lit: c_int) -> i32 {
        unsafe { self.ffi.ccadical_val(self.ptr, lit) }
    }

    pub fn failed(&self, lit: c_int) -> i32 {
        unsafe { self.ffi.ccadical_failed(self.ptr, lit) }
    }

    pub fn set_option(&self, _name: *const i8, _val: i32) {
        todo!()
    }

    pub fn limit(&self, _name: *const i8, _limit: i32) {
        todo!()
    }

    pub fn get_option(&self, _name: *const i8) -> i32 {
        todo!()
    }

    pub fn print_statistics(&self) {
        unsafe { self.ffi.ccadical_print_statistics(self.ptr) }
    }

    pub fn active(&self) -> i64 {
        unsafe { self.ffi.ccadical_active(self.ptr) }
    }

    pub fn irredundant(&self) -> i64 {
        unsafe { self.ffi.ccadical_irredundant(self.ptr) }
    }

    pub fn fixed(&self, lit: c_int) -> c_int {
        unsafe { self.ffi.ccadical_fixed(self.ptr, lit) }
    }

    pub fn terminate(&self) {
        unsafe { self.ffi.ccadical_terminate(self.ptr) }
    }

    pub fn freeze(&self, lit: c_int) {
        unsafe { self.ffi.ccadical_freeze(self.ptr, lit) }
    }

    pub fn frozen(&self, lit: c_int) -> c_int {
        unsafe { self.ffi.ccadical_frozen(self.ptr, lit) }
    }

    pub fn melt(&self, lit: c_int) {
        unsafe { self.ffi.ccadical_melt(self.ptr, lit) }
    }

    pub fn simplify(&self) -> c_int {
        unsafe { self.ffi.ccadical_simplify(self.ptr) }
    }
}

/// Additional CadicalSolver methods.
impl CadicalSolver2 {
    pub fn add_clause<I, L>(&self, lits: I)
    where
        I: IntoIterator<Item = L>,
        L: Into<c_int>,
    {
        for lit in lits.into_iter() {
            self.add(lit.into());
        }
        self.add(0);
    }

    // pub fn add_clause_unwrap<I, L>(&self, lits: I)
    // where
    //     I: IntoIterator<Item = L>,
    //     L: TryInto<c_int>,
    //     <L as TryInto<c_int>>::Error: std::fmt::Debug,
    // {
    //     self.add_clause(lits.into_iter().map(|x| x.try_into().unwrap()))
    // }
}

// impl From<ipasir::Lit> for Lit {
//     fn from(lit: ipasir::Lit) -> Self {
//         Lit(lit.to_ffi() as i32)
//     }
// }
//
// impl ipasir::Ipasir for CadicalSolver2 {
//     fn signature(&self) -> &'static str {
//         self.signature()
//     }
//
//     fn add_clause<I, L>(&mut self, lits: I)
//     where
//         I: IntoIterator<Item = L>,
//         L: Into<ipasir::Lit>,
//     {
//         for lit in lits.into_iter() {
//             Self::add(self, lit.into().into());
//         }
//         Self::add(self, Lit(0));
//     }
//
//     fn assume(&mut self, lit: ipasir::Lit) {
//         Self::assume(self, lit.into())
//     }
//
//     fn solve(&mut self) -> ipasir::Result<ipasir::SolveResponse> {
//         match Self::solve(self) {
//             0 => Ok(ipasir::SolveResponse::Interrupted),
//             10 => Ok(ipasir::SolveResponse::Sat),
//             20 => Ok(ipasir::SolveResponse::Unsat),
//             invalid => Err(ipasir::SolverError::ResponseSolve { value: invalid }),
//         }
//     }
//
//     fn val(&self, lit: ipasir::Lit) -> ipasir::Result<ipasir::LitValue> {
//         match Self::val(self, lit.into()) {
//             0 => Ok(ipasir::LitValue::DontCare),
//             p if p == lit.to_ffi() => Ok(ipasir::LitValue::True),
//             n if n == -lit.to_ffi() => Ok(ipasir::LitValue::False),
//             invalid => Err(ipasir::SolverError::ResponseVal { value: invalid }),
//         }
//     }
//
//     fn failed(&self, lit: ipasir::Lit) -> ipasir::Result<bool> {
//         match Self::failed(self, lit.into()) {
//             0 => Ok(true),
//             1 => Ok(false),
//             invalid => Err(ipasir::SolverError::ResponseFailed { value: invalid }),
//         }
//     }
// }
