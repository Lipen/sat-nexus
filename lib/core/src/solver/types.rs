use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SolveResponse {
    Sat,
    Unsat,
    Unknown,
}

impl Display for SolveResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use SolveResponse::*;
        match self {
            Sat => write!(f, "SAT"),
            Unsat => write!(f, "UNSAT"),
            Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LitValue {
    True,
    False,
    DontCare,
}

impl LitValue {
    pub fn bool(&self) -> bool {
        use LitValue::*;
        match self {
            True => true,
            False => false,
            DontCare => panic!("DontCare can't be converted to bool!"),
        }
    }
}

impl Display for LitValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use LitValue::*;
        match self {
            True => write!(f, "1"),
            False => write!(f, "0"),
            DontCare => write!(f, "X"),
        }
    }
}
