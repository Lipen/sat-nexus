use std::convert::TryInto;
use std::fmt;

use super::ffi::*;
use super::interface::*;
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

    pub fn new_cadical() -> Self {
        Self::new_custom(&CADICAL)
    }
    pub fn new_minisat() -> Self {
        Self::new_custom(&MINISAT)
    }
    pub fn new_glucose() -> Self {
        Self::new_custom(&GLUCOSE)
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
        self.ptr = unsafe { self.ffi.ipasir_init() };
    }

    fn release(&mut self) {
        if !self.ptr.is_null() {
            unsafe { self.ffi.ipasir_release(self.ptr) }
            self.ptr = std::ptr::null_mut();
        }
    }
}

impl Drop for IpasirSolver {
    fn drop(&mut self) {
        // unsafe { self.ffi.ipasir_release(self.ptr) }
        self.release();
    }
}

impl fmt::Display for IpasirSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.signature())
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

    pub fn add_clause_try<I, L>(&self, lits: I) -> Result<(), <L as TryInto<Lit>>::Error>
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

// pub struct IpasirSolver {
//     ffi: &'static IpasirFFI,
//     ptr: *mut c_void,
// }
//
// impl IpasirSolver {
//     pub fn new_custom(ffi: &'static IpasirFFI) -> Self {
//         IpasirSolver {
//             ffi,
//             ptr: unsafe { ffi.ipasir_init() },
//         }
//     }
//
//     pub fn new_cadical() -> Self {
//         Self::new_custom(&CADICAL)
//     }
//     pub fn new_minisat() -> Self {
//         Self::new_custom(&MINISAT)
//     }
//     pub fn new_glucose() -> Self {
//         Self::new_custom(&GLUCOSE)
//     }
// }
//
// impl fmt::Display for IpasirSolver {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.signature())
//     }
// }
//
// impl Ipasir for IpasirSolver {
//     fn signature(&self) -> &'static str {
//         let c_chars = unsafe { self.ffi.ipasir_signature() };
//         let c_str = unsafe { CStr::from_ptr(c_chars) };
//         c_str
//             .to_str()
//             .expect("The IPASIR implementation returned invalid UTF-8.")
//     }
//
//     fn add_clause<I, L>(&mut self, lits: I)
//     where
//         I: IntoIterator<Item = L>,
//         L: Into<Lit>,
//     {
//         for lit in lits.into_iter() {
//             unsafe { self.ffi.ipasir_add(self.ptr, lit.into().to_ffi()) }
//         }
//         unsafe { self.ffi.ipasir_add(self.ptr, 0) }
//     }
//
//     fn assume(&mut self, lit: Lit) {
//         // TODO: maybe use `lit.into()` everywhere instead of `lit.to_ffi()`
//         unsafe { self.ffi.ipasir_assume(self.ptr, lit.to_ffi()) }
//     }
//
//     fn solve(&mut self) -> Result<SolveResponse> {
//         match unsafe { self.ffi.ipasir_solve(self.ptr) } {
//             0 => Ok(SolveResponse::Interrupted),
//             10 => Ok(SolveResponse::Sat),
//             20 => Ok(SolveResponse::Unsat),
//             invalid => Err(SolverError::ResponseSolve { value: invalid }),
//         }
//     }
//
//     fn val(&self, lit: Lit) -> Result<LitValue> {
//         match unsafe { self.ffi.ipasir_val(self.ptr, lit.to_ffi()) } {
//             0 => Ok(LitValue::DontCare),
//             p if p == lit.to_ffi() => Ok(LitValue::True),
//             n if n == -lit.to_ffi() => Ok(LitValue::False),
//             invalid => Err(SolverError::ResponseVal { value: invalid }),
//         }
//     }
//
//     fn failed(&self, lit: Lit) -> Result<bool> {
//         match unsafe { self.ffi.ipasir_failed(self.ptr, lit.to_ffi()) } {
//             0 => Ok(true),
//             1 => Ok(false),
//             invalid => Err(SolverError::ResponseFailed { value: invalid }),
//         }
//     }
// }

impl IpasirExt for IpasirSolver {
    fn model(&self) -> Vec<bool> {
        todo!()
    }
}
