use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver, Select, Sender};
use log::{debug, trace};

use cadical::statik::Cadical;
use cadical::SolveResponse;
use simple_sat::lit::Lit;
use simple_sat::utils::{parse_dimacs, DisplaySlice};

use crate::utils::clause_to_external;

#[derive(Debug)]
pub struct Task {
    pub cube: Vec<Lit>,
}

impl Task {
    pub fn new(cube: Vec<Lit>) -> Self {
        Self { cube }
    }
}

impl Task {
    pub fn solve_with(&self, solver: &Cadical) -> SolveResponse {
        for &lit in self.cube.iter() {
            solver.assume(lit.to_external()).unwrap();
        }
        // solver.limit("conflicts", 1000);
        let res = solver.solve().unwrap();
        // if res == SolveResponse::Unsat {
        //     let mut lemma = Vec::new();
        //     for &lit in self.cube.iter() {
        //         if solver.failed(lit.to_external()).unwrap() {
        //             lemma.push(-lit);
        //         }
        //     }
        //     if lemma.len() < self.cube.len() {
        //         // lemma.sort_by_key(|lit| lit.inner());
        //         // debug!("new lemma from unsat core: {}", DisplaySlice(&lemma));
        //         solver.add_clause(clause_to_external(&lemma));
        //     }
        // }
        res
    }
}

pub struct SolverActor {
    handle: JoinHandle<()>,
    result_receiver: Receiver<(Task, SolveResponse, Duration)>,
}

impl SolverActor {
    pub fn new<F>(task_receiver: Receiver<Task>, init: F) -> Self
    where
        F: Fn() -> Cadical + Send + Sync + 'static,
    {
        let (result_sender, result_receiver) = unbounded();
        let handle = thread::spawn(move || {
            let solver = init();
            for task in task_receiver {
                let time_solve = Instant::now();
                let res = task.solve_with(&solver);
                let time_solve = time_solve.elapsed();
                trace!("{:?} in {:.1}s for {}", res, time_solve.as_secs_f64(), DisplaySlice(&task.cube));
                if result_sender.send((task, res, time_solve)).is_err() {
                    break;
                }
            }
            debug!(
                "finished {:?}: conflicts = {}, decisions = {}, propagations = {}",
                thread::current().id(),
                solver.conflicts(),
                solver.decisions(),
                solver.propagations(),
            );
        });
        Self { handle, result_receiver }
    }
}

pub struct SolverPool {
    pool: Vec<SolverActor>,
    task_sender: Sender<Task>,
}

impl SolverPool {
    pub fn new_with<F>(size: usize, init: F) -> Self
    where
        F: Fn(usize) -> Cadical + Send + Sync + 'static,
    {
        debug!("Initializing pool of size {}", size);
        let (task_sender, task_receiver) = unbounded::<Task>();
        let init = Arc::new(init);
        let pool = (0..size)
            .map(|i| {
                let receiver = task_receiver.clone();
                let init = Arc::clone(&init);
                SolverActor::new(receiver, move || init(i))
            })
            .collect();
        Self { pool, task_sender }
    }

    pub fn new_from<P>(size: usize, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let clauses: Vec<Vec<Lit>> = parse_dimacs(path).collect();
        Self::new_with(size, move |_| {
            let solver = Cadical::new();
            for clause in clauses.iter() {
                solver.add_clause(clause_to_external(clause));
            }
            solver
        })
    }
}

impl SolverPool {
    pub fn submit(&self, task: Task) {
        self.task_sender.send(task).unwrap();
    }

    pub fn results(&self) -> impl IntoIterator<Item = (Task, SolveResponse, Duration)> + '_ {
        self.pool.iter().flat_map(|s| s.result_receiver.try_iter())
    }

    pub fn join(&self) -> impl IntoIterator<Item = (Task, SolveResponse, Duration)> + '_ {
        let receivers: Vec<_> = self.pool.iter().map(|s| &s.result_receiver).collect();
        std::iter::from_fn(move || {
            let mut sel = Select::new();
            let mut num_receivers = 0;
            for r in receivers.iter() {
                sel.recv(r);
                num_receivers += 1;
            }
            while num_receivers > 0 {
                let index = sel.ready();
                match receivers[index].try_recv() {
                    Ok(res) => return Some(res),
                    Err(_) => {
                        sel.remove(index);
                        num_receivers -= 1;
                    }
                }
            }
            None
        })
    }

    pub fn finish(self) {
        drop(self.task_sender);
        for s in self.pool {
            s.handle.join().unwrap();
        }
        // TODO: collect the remaining results in pool's receivers
    }
}
