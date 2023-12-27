use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

use itertools::{equal, Itertools};
use rand::prelude::*;

use simple_sat::var::Var;

#[derive(Debug, Clone)]
pub struct Instance {
    pub(crate) genome: Vec<bool>,
    pub(crate) pool: Rc<RefCell<Vec<Var>>>,
}

impl Instance {
    pub fn new(genome: Vec<bool>, pool: Rc<RefCell<Vec<Var>>>) -> Self {
        Self { genome, pool }
    }

    pub fn new_random<R: Rng + ?Sized>(pool: Rc<RefCell<Vec<Var>>>, rng: &mut R) -> Self {
        let size = pool.borrow().len();
        let genome = (0..size).map(|_| rng.gen()).collect();
        Self::new(genome, pool)
    }

    pub fn new_random_with_weight<R: Rng + ?Sized>(pool: Rc<RefCell<Vec<Var>>>, weight: usize, rng: &mut R) -> Self {
        let size = pool.borrow().len();
        assert!(weight <= size);
        let mut genome = Vec::with_capacity(size);
        genome.resize_with(weight, || true);
        genome.resize_with(size, || false);
        genome.shuffle(rng);
        Self::new(genome, pool)
    }
}

// impl Debug for Instance {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.bitstring())
//     }
// }

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{{{}}}", self.get_variables().iter().map(|v| v.to_external()).join(", "))
        } else {
            write!(f, "{}", self.bitstring())
        }
    }
}

impl Index<usize> for Instance {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        self.genome.index(index)
    }
}

impl IndexMut<usize> for Instance {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.genome.index_mut(index)
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        equal(
            zip(self.genome.iter(), self.pool.borrow().iter()),
            zip(other.genome.iter(), other.pool.borrow().iter()),
        )
    }
}

impl Eq for Instance {}

impl Hash for Instance {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.genome.hash(state);
    }
}

impl Instance {
    // TODO: add 'key() -> ...' for using Instance as HashMap keys
    //   without actually storing the whole Instance itself as key.

    pub fn bitstring(&self) -> String {
        self.genome.iter().map(|&b| if b { '1' } else { '0' }).collect()
    }

    pub fn weight(&self) -> usize {
        self.genome.iter().filter(|&&b| b).count()
    }

    pub fn get_variables(&self) -> Vec<Var> {
        zip(self.genome.iter(), self.pool.borrow().iter())
            .filter_map(|(&b, &v)| if b { Some(v) } else { None })
            .collect()
    }
}
