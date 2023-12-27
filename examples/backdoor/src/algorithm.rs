use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::{Duration, Instant};

use log::{debug, info, trace};
use rand::distributions::{Bernoulli, Distribution};
use rand::prelude::*;

use simple_sat::lbool::LBool;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::var::Var;

use crate::fitness::Fitness;
use crate::instance::Instance;

#[derive(Debug)]
pub struct Algorithm {
    pub solver: Solver,
    pub pool: Rc<Vec<Var>>,
    pub rng: StdRng,
    pub cache: HashMap<Instance, Fitness>,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub options: Options,
    pub derived_clauses: HashSet<Vec<Lit>>,
    pub learnt_clauses: HashSet<Vec<Lit>>,
}

impl Algorithm {
    pub fn new(mut solver: Solver, pool: Vec<Var>, options: Options) -> Self {
        // Reset the limits for reduceDB:
        solver.learning_guard.reset(solver.num_clauses());

        Self {
            solver,
            pool: Rc::new(pool),
            rng: StdRng::seed_from_u64(options.seed),
            cache: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            options,
            derived_clauses: HashSet::new(),
            learnt_clauses: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub struct Options {
    pub seed: u64,
    pub add_learnts_in_propcheck_all_tree: bool,
    pub ban_used_variables: bool,
}

pub const DEFAULT_OPTIONS: Options = Options {
    seed: 42,
    add_learnts_in_propcheck_all_tree: false,
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
        weight: usize,
        num_iter: usize,
        stagnation_limit: Option<usize>,
        max_rho: Option<f64>,
        min_iter: usize,
    ) -> RunResult {
        let start_time = Instant::now();

        info!(
            "Running EA for {} iterations with pool size {} and weight {}",
            num_iter,
            self.pool.len(),
            weight
        );

        debug!("solver.num_vars() = {}", self.solver.num_vars());
        debug!("solver.num_clauses() = {}", self.solver.num_clauses());
        debug!("solver.num_learnts() = {}", self.solver.num_learnts());

        // Ban already assigned variables:
        let mut already_assigned = HashSet::new();
        for i in 0..self.solver.num_vars() {
            let var = Var::new(i as u32);
            let value = self.solver.value_var(var);
            if value != LBool::Undef {
                trace!("Variable {} already assigned value {:?}", var, value);
                already_assigned.insert(var);
            }
        }
        self.pool = Rc::new(self.pool.iter().copied().filter(|v| !already_assigned.contains(v)).collect());

        // Create an initial instance:
        let mut instance = self.initial_instance(Rc::clone(&self.pool), weight);
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
                    self.initial_instance(Rc::clone(&self.pool), weight)
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
                    "[{} / {}] {:?} for weight={} in {:.3} ms",
                    i,
                    num_iter,
                    mutated_fitness,
                    mutated_instance.weight(),
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
            let used_variables: HashSet<Var> = best_instance.get_variables().into_iter().collect();
            self.pool = Rc::new(self.pool.iter().copied().filter(|v| !used_variables.contains(v)).collect());
        }

        // Clear the cache:
        self.cache.clear();

        let elapsed_time = start_time.elapsed();
        info!("Run done in {:.3} s", elapsed_time.as_secs_f64());

        RunResult {
            best_iteration,
            best_instance,
            best_fitness,
            time: elapsed_time,
            records,
        }
    }

    fn initial_instance(&mut self, pool: Rc<Vec<Var>>, weight: usize) -> Instance {
        let instance = Instance::new_random_with_weight(pool, weight, &mut self.rng);
        assert_eq!(instance.weight(), weight);
        instance
    }

    fn calculate_fitness(&mut self, instance: &Instance, best: Option<&Fitness>) -> Fitness {
        if let Some(fit) = self.cache.get(instance) {
            self.cache_hits += 1;
            fit.clone()
        } else {
            self.cache_misses += 1;

            let vars = instance.get_variables();
            assert!(vars.len() < 32);

            // Compute rho:
            let add_learnts = self.options.add_learnts_in_propcheck_all_tree;
            let mut learnts = Vec::new();
            // let limit = 0;
            let limit = best.map_or(0, |b| b.num_hard);
            let num_hard = self.solver.propcheck_all_tree(&vars, limit, add_learnts, &mut learnts);
            self.learnt_clauses.extend(learnts);
            let num_total = 1u64 << vars.len();
            let rho = 1.0 - (num_hard as f64 / num_total as f64);

            // Calculate the fitness value:
            let value = 1.0 - rho;

            let fit = Fitness { value, rho, num_hard };

            self.cache.insert(instance.clone(), fit.clone());
            fit
        }
    }

    fn mutate(&mut self, instance: &mut Instance) {
        let weight = instance.weight();
        let p = 1.0 / weight as f64;
        let d = Bernoulli::new(p).unwrap();

        // Determine the indices of ones and zeros in the instance's genome:
        let mut ones = Vec::new();
        let mut zeros = Vec::new();
        for (i, &b) in instance.genome.iter().enumerate() {
            if b {
                ones.push(i);
            } else {
                zeros.push(i);
            }
        }

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
