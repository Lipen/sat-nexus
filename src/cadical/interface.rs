use crate::ipasir::{Lit, LitValue, Result, SolveResponse};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FixedValue {
    /// The literal is implied by the formula.
    Implied,
    /// The negation of the literal is implied by the formula.
    Negation,
    /// It is unclear at this point whether the literal is implied by the formula.
    Unclear,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SimplifyResponse {
    Unknown = 0,
    Sat = 10,
    Unsat = 20,
}

pub trait CadicalInterface {
    fn signature(&self) -> &'static str;
    fn reset(&mut self);
    fn release(&mut self);
    fn add(&self, lit_or_zero: i32);
    fn assume(&self, lit: Lit);
    fn solve(&self) -> Result<SolveResponse>;
    fn val(&self, lit: Lit) -> Result<LitValue>;
    fn failed(&self, lit: Lit) -> Result<bool>;
    // TODO: set_terminate
    // TODO: set_learn
    fn set_option(&self, name: &'static str, val: i32);
    fn limit(&self, name: &'static str, limit: i32);
    fn get_option(&self, name: &'static str) -> i32;
    fn print_statistics(&self);
    fn active(&self) -> i64;
    fn irredundant(&self) -> i64;
    fn fixed(&self, lit: Lit) -> Result<FixedValue>;
    fn terminate(&self);
    fn freeze(&self, lit: Lit);
    fn frozen(&self, lit: Lit) -> Result<bool>;
    fn melt(&self, lit: Lit);
    fn simplify(&self) -> Result<SimplifyResponse>;
}

// Symbols:
//   ccadical_signature,
//   ccadical_init,
//   ccadical_release,
//   ccadical_add,
//   ccadical_assume,
//   ccadical_solve,
//   ccadical_val,
//   ccadical_failed,
//   ccadical_set_terminate,
//   ccadical_set_learn,
//   ccadical_set_option,
//   ccadical_limit,
//   ccadical_get_option,
//   ccadical_print_statistics,
//   ccadical_active,
//   ccadical_irredundant,
//   ccadical_fixed,
//   ccadical_terminate,
//   ccadical_freeze,
//   ccadical_frozen,
//   ccadical_melt,
//   ccadical_simplify,
