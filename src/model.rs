#![allow(dead_code)]

use std::ops::Index;

use crate::ipasir::Var;

// trait Model {
//     fn get(lit: Lit) -> bool;
// }

struct Model {
    data: Vec<bool>,
}

impl Model {
    fn get(&self, var: Var) -> bool {
        self.data[var.0 as usize]
    }
}

impl Index<Var> for Model {
    type Output = bool;

    fn index(&self, var: Var) -> &Self::Output {
        &self.data[var.0 as usize]
    }
}
