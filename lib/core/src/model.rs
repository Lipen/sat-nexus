#![allow(dead_code)]

use crate::lit::Lit;

pub struct Model {
    data: Vec<bool>,
}

impl Model {
    pub fn new(data: Vec<bool>) -> Self {
        Self { data }
    }
}

impl Model {
    fn get(&self, lit: Lit) -> bool {
        let value = self.data[lit.var() as usize];
        let is_pos = lit.get() < 0;
        value ^ is_pos
    }
}
