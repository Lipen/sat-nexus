use std::fmt;

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
