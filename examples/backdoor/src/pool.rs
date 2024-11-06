use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{debug, trace};

use cadical::statik::Cadical;
use cadical::SolveResponse;
use simple_sat::lit::Lit;
use simple_sat::utils::parse_dimacs;

use crate::utils::lits_to_external;

#[derive(Debug)]
pub struct CubeTask {
    pub i: usize,
    pub cube: Vec<Lit>,
}

impl CubeTask {
    pub fn new(i: usize, cube: Vec<Lit>) -> Self {
        Self { i, cube }
    }
}

impl CubeTask {
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

type TaskResult<Task> = (Task, SolveResponse, Duration);

pub struct SolverActor {
    handle: JoinHandle<()>,
}

impl SolverActor {
    pub fn new<Task, F, S>(task_receiver: Receiver<Task>, result_sender: Sender<TaskResult<Task>>, init: F, solve: S) -> Self
    where
        Task: Debug + Send + 'static,
        F: Fn() -> Cadical + Send + 'static,
        S: Fn(&Task, &Cadical) -> SolveResponse + Send + 'static,
    {
        let handle = thread::spawn(move || {
            let solver = init();
            for task in task_receiver {
                let time_solve = Instant::now();
                let res = solve(&task, &solver);
                let time_solve = time_solve.elapsed();
                trace!("{:?} in {:.1}s for {:?}", res, time_solve.as_secs_f64(), task);
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
        Self { handle }
    }
}

pub struct SolverPool<Task> {
    pool: Vec<SolverActor>,
    task_sender: Sender<Task>,
    result_receiver: Receiver<TaskResult<Task>>,
}

impl<Task> SolverPool<Task>
where
    Task: Debug + Send + 'static,
{
    pub fn new_with<F, S>(size: usize, init: F, solve: S) -> Self
    where
        F: Fn(usize) -> Cadical + Send + Sync + 'static,
        S: Fn(&Task, &Cadical) -> SolveResponse + Send + Sync + 'static,
    {
        debug!("Initializing pool of size {}", size);
        let (result_sender, result_receiver) = unbounded::<TaskResult<Task>>();
        let (task_sender, task_receiver) = unbounded();
        let init = Arc::new(init);
        let solve = Arc::new(solve);
        let pool = (0..size)
            .map(|i| {
                let sender = result_sender.clone();
                let receiver = task_receiver.clone();
                let init = Arc::clone(&init);
                let solve = Arc::clone(&solve);
                SolverActor::new(receiver, sender, move || init(i), move |task, solver| solve(task, solver))
            })
            .collect();
        Self {
            pool,
            task_sender,
            result_receiver,
        }
    }

    pub fn new_from<P, S>(size: usize, path: P, solve: S) -> Self
    where
        P: AsRef<Path>,
        S: Fn(&Task, &Cadical) -> SolveResponse + Send + Sync + 'static,
    {
        let clauses: Vec<Vec<Lit>> = parse_dimacs(path).collect();
        Self::new_with(
            size,
            move |_| {
                let solver = Cadical::new();
                for clause in clauses.iter() {
                    solver.add_clause(lits_to_external(clause));
                }
                solver
            },
            solve,
        )
    }
}

impl<Task> SolverPool<Task> {
    pub fn submit(&self, task: Task) {
        self.task_sender.send(task).unwrap();
    }

    pub fn results(&self) -> impl Iterator<Item = TaskResult<Task>> + '_ {
        self.result_receiver.try_iter()
    }

    pub fn join(&self) -> impl Iterator<Item = TaskResult<Task>> + '_ {
        self.result_receiver.iter()
    }

    pub fn finish(self) -> impl Iterator<Item = TaskResult<Task>> {
        drop(self.task_sender);
        for s in self.pool {
            s.handle.join().unwrap();
        }
        self.result_receiver.into_iter()
    }
}
