use snafu::Snafu;

use super::Lit;

pub type Result<T, E = SolverError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[allow(clippy::enum_variant_names)]
pub enum SolverError {
    #[snafu(display("Invalid response from `solve()`: {}", value))]
    InvalidResponseSolve { value: i32 },

    #[snafu(display("Invalid response from `val({})`: {}", lit, value))]
    InvalidResponseVal { lit: Lit, value: i32 },

    #[snafu(display("Invalid response from `failed({})`: {}", lit, value))]
    InvalidResponseFailed { lit: Lit, value: i32 },
}
