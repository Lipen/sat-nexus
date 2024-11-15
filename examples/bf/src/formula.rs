#![allow(clippy::should_implement_trait)]

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum BooleanFormula {
    Var(usize),
    Not(Box<BooleanFormula>),
    And(Box<BooleanFormula>, Box<BooleanFormula>),
    Or(Box<BooleanFormula>, Box<BooleanFormula>),
}

impl BooleanFormula {
    pub fn var(index: usize) -> Self {
        BooleanFormula::Var(index)
    }

    pub fn not(formula: BooleanFormula) -> Self {
        BooleanFormula::Not(Box::new(formula))
    }

    pub fn and(left: BooleanFormula, right: BooleanFormula) -> Self {
        BooleanFormula::And(Box::new(left), Box::new(right))
    }

    pub fn or(left: BooleanFormula, right: BooleanFormula) -> Self {
        BooleanFormula::Or(Box::new(left), Box::new(right))
    }
}

impl Display for BooleanFormula {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BooleanFormula::Var(index) => {
                write!(f, "x{}", index)
            }
            BooleanFormula::Not(formula) => {
                write!(f, "¬{}", formula)
            }
            BooleanFormula::And(left, right) => {
                write!(f, "({} ∧ {})", left, right)
            }
            BooleanFormula::Or(left, right) => {
                write!(f, "({} ∨ {})", left, right)
            }
        }
    }
}
