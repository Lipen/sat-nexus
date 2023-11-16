use crate::fitness::Fitness;
use itertools::Itertools;
use rand::prelude::*;
use simple_sat::solver::Solver;
use simple_sat::var::Var;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Instance {
    pub(crate) genome: Vec<bool>,
}

impl Instance {
    pub fn new(genome: Vec<bool>) -> Self {
        Self { genome }
    }

    pub fn new_random<R: Rng + ?Sized>(size: usize, rng: &mut R) -> Self {
        let genome = (0..size).map(|_| rng.gen()).collect();
        Self::new(genome)
    }

    pub fn new_random_with_weight<R: Rng + ?Sized>(size: usize, weight: usize, rng: &mut R) -> Self {
        assert!(weight <= size);
        let mut genome = Vec::with_capacity(size);
        genome.resize_with(weight, || true);
        genome.resize_with(size, || false);
        genome.shuffle(rng);
        Self::new(genome)
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
            write!(f, "{{{}}}", self.get_variables().iter().map(|v| v.0).join(", "))
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

impl Instance {
    pub fn bitstring(&self) -> String {
        self.genome.iter().map(|&b| if b { '1' } else { '0' }).collect()
    }

    pub fn weight(&self) -> usize {
        self.genome.iter().filter(|&&b| b).count()
    }

    pub fn indices_true(&self) -> impl Iterator<Item = usize> + '_ {
        self.genome.iter().enumerate().filter_map(|(i, &b)| if b { Some(i) } else { None })
    }
    pub fn indices_false(&self) -> impl Iterator<Item = usize> + '_ {
        self.genome.iter().enumerate().filter_map(|(i, &b)| if !b { Some(i) } else { None })
    }

    pub fn get_variables(&self) -> Vec<Var> {
        self.indices_true().map(|i| Var::new(i as u32)).collect()
    }

    pub(crate) fn calculate_fitness(&self, solver: &mut Solver) -> Fitness {
        // Extract the set of variables an instance represents:
        let vars = self.get_variables();
        assert!(vars.len() < 32);

        // Compute rho:
        let num_hard = solver.propcheck_all_tree(&vars);
        let num_total = 1u64 << vars.len();
        let rho = 1.0 - (num_hard as f64 / num_total as f64);

        // Calculate the fitness value:
        let fitness = 1.0 - rho;

        Fitness {
            value: fitness,
            rho,
            num_hard,
        }
    }
}
