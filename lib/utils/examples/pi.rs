use sat_nexus_utils::pool::Pool;
use std::time::Instant;

use indicatif::{ProgressBar, ProgressIterator};
use rand::prelude::*;

// Example of using the Pool to estimate the value of pi using the Monte Carlo method.
fn main() {
    const NUM_THREADS: usize = 4;
    const NUM_TASKS: usize = 1024;
    const NUM_SAMPLES: usize = 1_000_000;

    let time_start = Instant::now();

    // Create a pool with NUM_THREADS threads
    let pool = Pool::<usize, u64>::new_with(NUM_THREADS, |i, rx, tx| {
        // input is the number of samples
        // output is the number of points inside the circle

        let mut rng = StdRng::seed_from_u64(1_000_000 + i as u64);

        for num_samples in rx {
            let mut num_inside = 0;
            for _ in 0..num_samples {
                let x = rng.gen_range(-1.0..=1.0);
                let y = rng.gen_range(-1.0..=1.0);
                if x * x + y * y <= 1.0 {
                    num_inside += 1;
                }
            }
            tx.send(num_inside).unwrap();
        }
    });

    // Submit NUM_TASKS tasks to the pool, each with NUM_SAMPLES samples
    println!("Submitting {} tasks, each with {} samples...", NUM_TASKS, NUM_SAMPLES);
    for _ in 0..NUM_TASKS {
        pool.submit(NUM_SAMPLES);
    }

    // Wait for all submitted tasks to finish
    println!("Waiting for {} tasks to finish...", NUM_TASKS);
    let pb = ProgressBar::new(NUM_TASKS as u64);
    let num_inside = pool.join().take(NUM_TASKS).progress_with(pb).sum::<u64>();
    println!("Number of points inside the circle: {}", num_inside);

    // Estimate the value of pi
    let pi = 4.0 * num_inside as f64 / (NUM_SAMPLES as u64 * NUM_TASKS as u64) as f64;
    println!("Estimated value of pi: {}", pi);

    println!("All done in {:.3} s", time_start.elapsed().as_secs_f64());
}
