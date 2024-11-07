use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct Worker {
    handle: JoinHandle<()>,
}

impl Worker {
    pub fn new<F>(init: F) -> Self
    where
        F: FnOnce(),
        F: Send + 'static,
    {
        Self {
            handle: thread::spawn(init),
        }
    }
}

pub struct Pool<Input, Output> {
    workers: Vec<Worker>,
    task_sender: Sender<Input>,
    task_receiver: Receiver<Input>,
    result_sender: Sender<Output>,
    result_receiver: Receiver<Output>,
}

impl<Input, Output> Pool<Input, Output>
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    pub fn new() -> Self {
        let (task_sender, task_receiver) = unbounded::<Input>();
        let (result_sender, result_receiver) = unbounded::<Output>();
        Self {
            workers: Vec::new(),
            task_sender,
            task_receiver,
            result_sender,
            result_receiver,
        }
    }

    pub fn new_with<F>(size: usize, f: F) -> Self
    where
        F: Fn(usize, Receiver<Input>, Sender<Output>),
        F: Send + Sync + 'static,
    {
        let (task_sender, task_receiver) = unbounded::<Input>();
        let (result_sender, result_receiver) = unbounded::<Output>();
        let f = Arc::new(f);
        let workers = (0..size)
            .map(|i| {
                let f = Arc::clone(&f);
                let receiver = task_receiver.clone();
                let sender = result_sender.clone();
                Worker::new(move || f(i, receiver, sender))
            })
            .collect();
        Self {
            workers,
            task_sender,
            task_receiver,
            result_sender,
            result_receiver,
        }
    }

    pub fn add_worker<F>(&mut self, f: F)
    where
        F: FnOnce(usize, Receiver<Input>, Sender<Output>),
        F: Send + 'static,
    {
        let index = self.workers.len();
        let receiver = self.task_receiver.clone();
        let sender = self.result_sender.clone();
        let worker = Worker::new(move || f(index, receiver, sender));
        self.workers.push(worker);
    }

    pub fn submit(&self, task: Input) {
        self.task_sender.send(task).unwrap();
    }

    pub fn results(&self) -> impl Iterator<Item = Output> + '_ {
        self.result_receiver.try_iter()
    }

    pub fn join(&self) -> impl Iterator<Item = Output> + '_ {
        self.result_receiver.iter()
    }

    /// Stop the workers and return the iterator of remaining tasks.
    pub fn finish(self) -> impl Iterator<Item = Output> {
        drop(self.task_sender);
        // TODO: do we also need `drop(self.task_receiver)` here?
        for w in self.workers {
            w.handle.join().unwrap()
        }
        drop(self.result_sender);
        self.result_receiver.into_iter()
    }

    /// Stop the workers and optionally discard the remaining tasks.
    /// - If `discard` is `true`, the remaining tasks are exhausted and dropped.
    /// - If `discard` is `false`, the remaining tasks are ignored.
    pub fn stop(self, discard: bool) {
        let remaining = self.finish();
        if discard {
            remaining.for_each(drop);
        }
    }
}
