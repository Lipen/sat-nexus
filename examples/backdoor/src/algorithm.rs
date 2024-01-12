use std::collections::HashSet;
use std::iter::zip;
use std::time::{Duration, Instant};

use ahash::AHashMap;
use log::{debug, info, trace};
use rand::distributions::{Bernoulli, Distribution};
use rand::prelude::*;

use simple_sat::var::Var;

use crate::fitness::Fitness;
use crate::instance::Instance;
use crate::solvers::SatSolver;

#[derive(Debug)]
pub struct Algorithm {
    pub solver: SatSolver,
    pub pool: Vec<Var>,
    pub rng: StdRng,
    pub cache: AHashMap<Vec<Var>, Fitness>,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub options: Options,
}

impl Algorithm {
    pub fn new(solver: SatSolver, pool: Vec<Var>, options: Options) -> Self {
        Self {
            solver,
            pool,
            rng: StdRng::seed_from_u64(options.seed),
            cache: AHashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            options,
        }
    }
}

#[derive(Debug)]
pub struct Options {
    pub seed: u64,
    pub ban_used_variables: bool,
}

pub const DEFAULT_OPTIONS: Options = Options {
    seed: 42,
    ban_used_variables: false,
};

impl Default for Options {
    fn default() -> Self {
        DEFAULT_OPTIONS
    }
}

#[derive(Debug)]
pub struct RunResult {
    pub best_iteration: usize,
    pub best_instance: Instance,
    pub best_fitness: Fitness,
    pub time: Duration,
    pub records: Vec<Record>,
}

#[derive(Debug)]
pub struct Record {
    pub iteration: usize,
    pub instance: Instance,
    pub fitness: Fitness,
}

impl Algorithm {
    pub fn run(
        &mut self,
        backdoor_size: usize,
        num_iter: usize,
        stagnation_limit: Option<usize>,
        max_rho: Option<f64>,
        min_iter: usize,
    ) -> RunResult {
        let start_time = Instant::now();

        info!("Running EA for {} iterations with backdoor size {}", num_iter, backdoor_size);

        debug!("solver.vars() = {}", self.solver.num_vars());

        // Ban already assigned variables:
        let mut active_vars = HashSet::new();
        for i in 0..self.solver.num_vars() {
            let var = Var::new(i as u32);
            if self.solver.is_active(var) {
                active_vars.insert(var);
            }
        }
        debug!("Active variables: {} / {}", active_vars.len(), self.solver.num_vars());
        self.pool.retain(|v| active_vars.contains(v));
        drop(active_vars);

        debug!("pool.len() = {}", self.pool.len());

        // Create an initial instance:
        let mut instance = self.initial_instance(backdoor_size);
        info!("Initial instance: {:#}", instance);

        // Evaluate the initial instance:
        let mut fitness = self.calculate_fitness(&instance, None);
        info!("Initial fitness: {:?}", fitness);

        // Store the best result:
        let mut best_iteration: usize = 0;
        let mut best_instance = instance.clone();
        let mut best_fitness = fitness.clone();

        // Store all results:
        let mut records = Vec::new();
        records.push(Record {
            iteration: 0,
            instance: instance.clone(),
            fitness: fitness.clone(),
        });

        // Count the stagnated iterations:
        let mut num_stagnation: usize = 0;

        for i in 1..=num_iter {
            let time_iter = Instant::now();

            // Break upon reaching the maximum required rho:
            if let Some(max_rho) = max_rho {
                if i > min_iter && best_fitness.rho >= max_rho {
                    debug!("Reached maximum required rho {} >= {}", best_fitness.rho, max_rho);
                    break;
                }
            }

            // Produce the new instance (either mutate the previous, or re-init):
            let mutated_instance = {
                let mut need_reinit = false;
                if let Some(stagnation_limit) = stagnation_limit {
                    if num_stagnation >= stagnation_limit {
                        need_reinit = true;
                    }
                }
                if need_reinit {
                    // Re-initialize:
                    num_stagnation = 0;
                    self.initial_instance(backdoor_size)
                } else {
                    // Mutate the instance:
                    let mut mutated_instance = instance.clone();
                    self.mutate(&mut mutated_instance);
                    mutated_instance
                }
            };

            // Evaluate the mutated instance:
            let mutated_fitness = self.calculate_fitness(&mutated_instance, Some(&best_fitness));

            // Save the record about the new instance:
            records.push(Record {
                iteration: i,
                instance: mutated_instance.clone(),
                fitness: mutated_fitness.clone(),
            });

            let time_iter = time_iter.elapsed();
            if i <= 10 || (i < 1000 && i % 100 == 0) || (i < 10000 && i % 1000 == 0) || i % 10000 == 0 {
                debug!(
                    "[{} / {}] rho={:.3}, hard={}, size={}, time={:.3} ms",
                    i,
                    num_iter,
                    mutated_fitness.rho,
                    mutated_fitness.num_hard,
                    backdoor_size,
                    time_iter.as_secs_f64() * 1000.0
                );
            }

            // Update the best:
            if mutated_fitness < best_fitness {
                best_iteration = i;
                best_instance = mutated_instance.clone();
                best_fitness = mutated_fitness.clone();
            } else {
                num_stagnation += 1;
                trace!("Stagnated for {} iterations", num_stagnation);
            }

            // (1+1) strategy:
            if mutated_fitness <= fitness {
                instance = mutated_instance;
                fitness = mutated_fitness;
            }
        }

        // Print the best result:
        info!("Best iteration: {} / {}", best_iteration, num_iter);
        info!("Best instance: {:#}", best_instance);
        info!("Best fitness: {:?}", best_fitness);

        debug!("cache hits: {}", self.cache_hits);
        debug!("cache misses: {}", self.cache_misses);

        // Ban used variables:
        if self.options.ban_used_variables {
            // Note: even if `best_instance.variables` is Vec (if not yet switched to HashSet),
            // it is OK to perform O(n) lookup since the instance size is generally very small.
            self.pool.retain(|v| !best_instance.variables.contains(v));
        }

        // Clear the cache:
        self.cache.clear();

        let elapsed_time = start_time.elapsed();
        info!("EA done in {:.3} s", elapsed_time.as_secs_f64());

        RunResult {
            best_iteration,
            best_instance,
            best_fitness,
            time: elapsed_time,
            records,
        }
    }

    fn initial_instance(&mut self, size: usize) -> Instance {
        Instance::new_random(size, &self.pool, &mut self.rng)
        // let vars: Vec<Var> = [15, 482, 820, 1155, 2072, 2231, 2509, 3579, 4051, 4483]
        //     .into_iter()
        //     .map(|v| Var::from_external(v))
        //     .collect();
        // assert_eq!(vars.len(), size);
        // Instance::new(vars)
    }

    fn calculate_fitness(&mut self, instance: &Instance, best: Option<&Fitness>) -> Fitness {
        let key = instance.get_variables();
        if let Some(fit) = self.cache.get(&key) {
            self.cache_hits += 1;
            fit.clone()
        } else {
            self.cache_misses += 1;

            let vars = instance.get_variables();
            assert!(vars.len() < 32);

            // Compute rho:
            // let limit = 0;
            let limit = best.map_or(0, |b| b.num_hard + 1);
            let num_hard = self.solver.propcheck_all_tree(&vars, limit);
            let num_total = 1u64 << vars.len();
            let rho = 1.0 - (num_hard as f64 / num_total as f64);

            // Calculate the fitness value:
            let value = 1.0 - rho;

            let fit = Fitness { value, rho, num_hard };

            self.cache.insert(key, fit.clone());
            fit
        }
    }

    fn mutate(&mut self, instance: &mut Instance) {
        let n = instance.len();
        let p = 1.0 / n as f64;
        let d = Bernoulli::new(p).unwrap();

        let mut to_replace = Vec::new();
        for i in 0..n {
            if d.sample(&mut self.rng) {
                to_replace.push(i);
            }
        }

        let other_vars: Vec<Var> = self.pool.iter().filter(|v| !instance.variables.contains(v)).copied().collect();
        let substituted = other_vars.choose_multiple(&mut self.rng, to_replace.len());
        for (i, &v) in zip(to_replace, substituted) {
            instance.variables[i] = v;
        }

        // Instance size should stay the same:
        assert_eq!(instance.len(), n);
    }
}
