use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SolveResponse {
    Sat,
    Unsat,
    Unknown,
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
impl fmt::Display for LitValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LitValue::*;
        match self {
            True => write!(f, "1"),
            False => write!(f, "0"),
            DontCare => write!(f, "X"),
        }
    }
}
