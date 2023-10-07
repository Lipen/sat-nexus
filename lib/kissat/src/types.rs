/// Possible responses from a call to `solve`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SolveResponse {
    /// The solver found the input to be satisfiable.
    Sat = 10,
    /// The solver found the input to be unsatisfiable.
    Unsat = 20,
    /// The solver was interrupted.
    Interrupted = 0,
}

/// Possible literal values from a call to `val`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LitValue {
    True,
    False,
    Any,
}

// Into<bool>
impl From<LitValue> for bool {
    fn from(v: LitValue) -> Self {
        match v {
            LitValue::True => true,
            LitValue::False => false,
            LitValue::Any => panic!("Cannot convert LitValue::Any to bool"),
        }
    }
}
