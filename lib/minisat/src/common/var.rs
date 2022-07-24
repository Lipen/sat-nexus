/// MiniSat variable.
///
/// **Note**: variables are 0-based integers, internally used as indices.
#[derive(Debug, Copy, Clone)]
pub struct Var(u32);

impl Var {
    pub fn new(var: u32) -> Self {
        Var(var)
    }

    pub(crate) fn get(self) -> u32 {
        self.0
    }
}
