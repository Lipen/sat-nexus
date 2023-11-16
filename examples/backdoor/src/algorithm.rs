use std::collections::HashMap;
use std::time::Instant;

use itertools::{Either, Itertools};
use log::{debug, info};
use rand::distributions::{Bernoulli, Distribution};
use rand::prelude::*;

use simple_sat::solver::Solver;

use crate::fitness::Fitness;
use crate::instance::Instance;

#[derive(Debug)]
pub struct Algorithm {
    solver: Solver,
    rng: StdRng,
    cache: HashMap<Instance, Fitness>,
    cache_hits: usize,
    cache_misses: usize,
}

impl Algorithm {
    pub fn new(solver: Solver, seed: u64) -> Self {
        Self {
            solver,
            rng: StdRng::seed_from_u64(seed),
            cache: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

impl Algorithm {
    pub fn run(&mut self, weight: usize, num_iter: usize) {
        let start_time = Instant::now();

        info!("Running EA for {} iterations with weight = {}", num_iter, weight);

        // Determine genome size:
        let genome_size = self.solver.num_vars();
        info!("Genome size: {}", genome_size);

        // Create an initial instance:
        let mut instance = self.initial_instance(genome_size, weight);
        info!("Initial instance: {:#}", instance);

        let mut fitness = self.calculate_fitness(&instance);
        info!("Initial fitness: {:?}", fitness);

        let mut best_iteration: usize = 0;
        let mut best_instance = instance.clone();
        let mut best_fitness = fitness.clone();

        for i in 1..=num_iter {
            // debug!("--- Iteration #{}", i);
            let start_time_iter = Instant::now();

            // Mutate the instance:
            let mut mutated_instance = instance.clone();
            self.mutate(&mut mutated_instance);
            let mutated_instance = mutated_instance;

            // Evaluate the mutated instance:
            let mutated_fitness = self.calculate_fitness(&mutated_instance);

            let elapsed_time_iter = Instant::now() - start_time_iter;

            if i <= 10 || (i < 1000 && i % 100 == 0) || (i < 10000 && i % 1000 == 0) || i % 10000 == 0 {
                debug!(
                    "[{} / {}] {:?} for weight={} in {:.3} ms",
                    i,
                    num_iter,
                    mutated_fitness,
                    mutated_instance.weight(),
                    elapsed_time_iter.as_secs_f64() * 1000.0
                );
            }

            // Update the best:
            if mutated_fitness < best_fitness {
                best_iteration = i;
                best_instance = mutated_instance.clone();
                best_fitness = mutated_fitness.clone();
            }

            // (1+1) strategy:
            if mutated_fitness <= fitness {
                instance = mutated_instance;
                fitness = mutated_fitness;
            }
        }

        info!("Best iteration: {} / {}", best_iteration, num_iter);
        info!("Best instance: {:#}", best_instance);
        info!("Best fitness: {:?}", best_fitness);

        debug!("cache hits: {}", self.cache_hits);
        debug!("cache misses: {}", self.cache_misses);

        let elapsed_time = Instant::now() - start_time;
        info!("Run done in {:.3} s", elapsed_time.as_secs_f64());
    }

    fn initial_instance(&mut self, size: usize, weight: usize) -> Instance {
        Instance::new_random_with_weight(size, weight, &mut self.rng)
    }

    fn calculate_fitness(&mut self, instance: &Instance) -> Fitness {
        if let Some(fit) = self.cache.get(&instance) {
            self.cache_hits += 1;
            fit.clone()
        } else {
            self.cache_misses += 1;
            let fit = instance.calculate_fitness(&mut self.solver);
            self.cache.insert(instance.clone(), fit.clone());
            fit
        }
    }

    fn mutate(&mut self, instance: &mut Instance) {
        let weight = instance.weight();
        let p = 1.0 / weight as f64;
        let d = Bernoulli::new(p).unwrap();

        // Determine the indices of ones and zeros in the instance's genome:
        let (ones, zeros): (Vec<_>, Vec<_>) =
            instance
                .genome
                .iter()
                .enumerate()
                .partition_map(|(i, &b)| if b { Either::Left(i) } else { Either::Right(i) });

        // Flip some (`Binomial(1/p)` distributed) number of 1's to 0's:
        let mut successes = 0;
        for i in ones {
            if d.sample(&mut self.rng) {
                instance[i] = false;
                successes += 1;
            }
        }

        // Flip the exact number of original 0's to 1's:
        for &j in zeros.choose_multiple(&mut self.rng, successes) {
            instance[j] = true;
        }

        // At the end, the Hamming weight must not change:
        assert_eq!(instance.weight(), weight);
    }
}
