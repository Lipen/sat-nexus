use std::convert::TryInto;
use std::ffi::CStr;
use std::fmt;

use super::ffi::*;
use super::interface::*;

pub struct CadicalSolver {
    ffi: &'static CadicalFFI,
    ptr: CadicalPtr,
}

impl CadicalSolver {
    pub fn new() -> Self {
        Self::new_custom(&CADICAL)
    }

    pub fn new_custom(ffi: &'static CadicalFFI) -> Self {
        Self {
            ffi,
            ptr: unsafe { ffi.ccadical_init() },
        }
    }

    pub fn new_null(ffi: &'static CadicalFFI) -> Self {
        Self {
            ffi,
            ptr: std::ptr::null_mut(),
        }
    }
}

impl CadicalSolver {
    pub fn _i_really_want_ffi(&self) -> &'static CadicalFFI {
        self.ffi
    }
    pub fn _i_really_want_ptr(&self) -> CadicalPtr {
        self.ptr
    }
}

impl CadicalInterface for CadicalSolver {
    fn signature(&self) -> &'static str {
        let c_chars = unsafe { self.ffi.ccadical_signature() };
        let c_str = unsafe { CStr::from_ptr(c_chars) };
        c_str
            .to_str()
            .expect("The implementation returned invalid UTF-8.")
    }

    fn reset(&mut self) {
        self.release();
        self.ptr = unsafe { self.ffi.ccadical_init() };
    }

    fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe { self.ffi.ccadical_release(self.ptr) }
            self.ptr = std::ptr::null_mut();
        }
    }

    fn add(&self, lit_or_zero: i32) {
        unsafe { self.ffi.ccadical_add(self.ptr, lit_or_zero) }
    }

    fn assume(&self, lit: Lit) {
        unsafe { self.ffi.ccadical_assume(self.ptr, lit.to_ffi()) }
    }

    fn solve(&self) -> Result<SolveResponse> {
        match unsafe { self.ffi.ccadical_solve(self.ptr) } {
            0 => Ok(SolveResponse::Interrupted),
            10 => Ok(SolveResponse::Sat),
            20 => Ok(SolveResponse::Unsat),
            invalid => Err(SolverError::InvalidResponseSolve { value: invalid }),
        }
    }

    fn val(&self, lit: Lit) -> Result<LitValue> {
        match unsafe { self.ffi.ccadical_val(self.ptr, lit.to_ffi()) } {
            0 => Ok(LitValue::DontCare),
            p if p == lit.to_ffi() => Ok(LitValue::True),
            n if n == -lit.to_ffi() => Ok(LitValue::False),
            invalid => Err(SolverError::InvalidResponseVal {
                lit,
                value: invalid,
            }),
        }
    }

    fn failed(&self, lit: Lit) -> Result<bool> {
        match unsafe { self.ffi.ccadical_failed(self.ptr, lit.to_ffi()) } {
            0 => Ok(true),
            1 => Ok(false),
            invalid => Err(SolverError::InvalidResponseFailed { value: invalid }),
        }
    }

    fn set_option(&self, _name: &'static str, _val: i32) {
        todo!()
    }

    fn limit(&self, _name: &'static str, _limit: i32) {
        todo!()
    }

    fn get_option(&self, _name: &'static str) -> i32 {
        todo!()
    }

    fn print_statistics(&self) {
        unsafe { self.ffi.ccadical_print_statistics(self.ptr) }
    }

    fn active(&self) -> i64 {
        unsafe { self.ffi.ccadical_active(self.ptr) }
    }

    fn irredundant(&self) -> i64 {
        unsafe { self.ffi.ccadical_irredundant(self.ptr) }
    }

    fn fixed(&self, lit: Lit) -> Result<FixedValue> {
        match unsafe { self.ffi.ccadical_fixed(self.ptr, lit.to_ffi()) } {
            1 => Ok(FixedValue::Implied),
            -1 => Ok(FixedValue::Negation),
            0 => Ok(FixedValue::Unclear),
            invalid => Err(SolverError::InvalidResponseFixed { value: invalid }),
        }
    }

    fn terminate(&self) {
        unsafe { self.ffi.ccadical_terminate(self.ptr) }
    }

    fn freeze(&self, lit: Lit) {
        unsafe { self.ffi.ccadical_freeze(self.ptr, lit.to_ffi()) }
    }

    fn frozen(&self, lit: Lit) -> Result<bool> {
        match unsafe { self.ffi.ccadical_frozen(self.ptr, lit.to_ffi()) } {
            0 => Ok(true),
            1 => Ok(false),
            invalid => Err(SolverError::InvalidResponseFrozen { value: invalid }),
        }
    }

    fn melt(&self, lit: Lit) {
        unsafe { self.ffi.ccadical_melt(self.ptr, lit.to_ffi()) }
    }

    fn simplify(&self) -> Result<SimplifyResponse> {
        match unsafe { self.ffi.ccadical_simplify(self.ptr) } {
            0 => Ok(SimplifyResponse::Unknown),
            10 => Ok(SimplifyResponse::Sat),
            20 => Ok(SimplifyResponse::Unsat),
            invalid => Err(SolverError::InvalidResponseSimplify { value: invalid }),
        }
    }
}

impl Drop for CadicalSolver {
    fn drop(&mut self) {
        self.release()
    }
}

impl fmt::Display for CadicalSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.signature())
    }
}

impl CadicalSolver {
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
        Ok(self.add_clause(lits))
    }
}
