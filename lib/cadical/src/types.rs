use snafu::Snafu;

pub type Result<T, E = CadicalError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum CadicalError {
    #[snafu(display("Literal must be non-zero"))]
    ZeroLiteral,

    #[snafu(display("Invalid response from `simplify()`: {}", value))]
    InvalidResponseSimplify { value: i32 },

    #[snafu(display("Invalid response from `solve()`: {}", value))]
    InvalidResponseSolve { value: i32 },

    #[snafu(display("Invalid response from `val({})`: {}", lit, value))]
    InvalidResponseVal { lit: i32, value: i32 },

    #[snafu(display("Invalid response from `failed({})`: {}", lit, value))]
    InvalidResponseFailed { lit: i32, value: i32 },

    #[snafu(display("Invalid response from `fixed({})`: {}", lit, value))]
    InvalidResponseFixed { lit: i32, value: i32 },

    #[snafu(display("Invalid response from `frozen({})`: {}", lit, value))]
    InvalidResponseFrozen { lit: i32, value: i32 },
}

/// Possible responses from a call to `simplify`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SimplifyResponse {
    Unknown = 0,
    Sat = 10,
    Unsat = 20,
}

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
}

// Into<bool>
impl From<LitValue> for bool {
    fn from(v: LitValue) -> Self {
        match v {
            LitValue::True => true,
            LitValue::False => false,
        }
    }
}

/// Possible responses from a call to `fixed`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FixedResponse {
    /// The literal is implied by the formula.
    Implied = 1,
    /// The negation of the literal is implied by the formula.
    Negation = -1,
    /// It is unclear at this point whether the literal is implied by the formula.
    Unclear = 0,
}

/// Possible responses from a call to `frozen`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FrozenResponse {
    Frozen,
    NotFrozen,
}
