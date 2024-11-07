use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{bounded, unbounded, Receiver, Select, Sender, TryRecvError};

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

    pub fn join(self) {
        self.handle.join().unwrap();
    }
}

pub struct Mediator<Input> {
    worker: Worker,
    job: JoinHandle<()>,
    tx_extra: Sender<Input>,
}

impl<Input> Mediator<Input> {
    pub fn new(worker: Worker, rx: Receiver<Input>, tx: Sender<Input>) -> Self
    where
        Input: Send + 'static,
    {
        let (tx_extra, rx_extra) = unbounded::<Input>();
        let job = thread::spawn(move || {
            let rxs = [&rx, &rx_extra];
            let mut sel = Select::new();
            for r in rxs {
                sel.recv(r);
            }
            loop {
                let index = sel.ready();
                match rxs[index].try_recv() {
                    Ok(msg) => {
                        tx.send(msg).unwrap();
                    }
                    Err(TryRecvError::Empty) => {
                        continue;
                    }
                    Err(TryRecvError::Disconnected) => {
                        assert!(index == 0 || index == 1);
                        let other = 1 - index;
                        for msg in rxs[other] {
                            tx.send(msg).unwrap();
                        }
                        break;
                    }
                }
            }
        });
        Self { worker, job, tx_extra }
    }

    pub fn join(self) {
        drop(self.tx_extra);
        self.job.join().unwrap();
        self.worker.join();
    }
}

// Input: pool -> workers
//           .-> mediator --> worker
//          /
// pool -->*--> mediator --> worker
//          \
//           `-> mediator --> worker
//
// Output: workers -> pool
//           .<- worker
//          /
// pool <--*<-- worker
//          \
//           `<- worker
//
pub struct Pool<Input, Output> {
    mediators: Vec<Mediator<Input>>,
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
            mediators: Vec::new(),
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
        let mediators = (0..size)
            .map(|i| {
                let f = Arc::clone(&f);
                let (task_sender_worker, task_receiver_worker) = bounded::<Input>(0);
                let result_sender = result_sender.clone();
                let worker = Worker::new(move || f(i, task_receiver_worker, result_sender));
                Mediator::new(worker, task_receiver.clone(), task_sender_worker)
            })
            .collect();
        Self {
            mediators,
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
        let (task_sender_worker, task_receiver_worker) = bounded::<Input>(0);
        let index = self.mediators.len();
        let result_sender = self.result_sender.clone();
        let worker = Worker::new(move || f(index, task_receiver_worker, result_sender));
        let mediator = Mediator::new(worker, self.task_receiver.clone(), task_sender_worker);
        self.mediators.push(mediator);
    }

    pub fn submit(&self, task: Input) {
        self.task_sender.send(task).unwrap();
    }

    pub fn broadcast(&self, task: Input)
    where
        Input: Clone,
    {
        for m in self.mediators.iter() {
            m.tx_extra.send(task.clone()).unwrap();
        }
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
        for m in self.mediators {
            m.join();
        }
        // TODO: do we also need `drop(self.task_receiver)` here?
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex() {
        const NUM_THREADS: usize = 4;
        const N: i32 = 1_000;

        let pool = Pool::<i32, Vec<i32>>::new_with(NUM_THREADS, |index, input_rx, tx_output| {
            let mut data = Vec::new();
            for x in input_rx {
                // println!("worker #{} got item {}", index, x);
                data.push(x);
            }
            println!("worker #{} captured {} items", index, data.len());
            tx_output.send(data).unwrap();
        });

        for i in 1..=N {
            pool.submit(i);
        }

        pool.broadcast(0);

        let mut total = 0;
        for data in pool.finish() {
            assert!(data.contains(&0));
            total += data.len() as u64;
        }
        assert_eq!(total, N as u64 + NUM_THREADS as u64);
    }
}
