use std::env;
use std::path::Path;
use std::time::Instant;

use simple_sat::solver::Solver;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        panic!("Too few arguments");
    }

    let path = &args[1];
    // let path = "data/coloring.cnf";
    // let path = "data/sgen/sgen_sat_n50_s42.cnf";
    println!("path = {}", path);

    let time_start = Instant::now();
    let mut solver = Solver::from_file(Path::new(path));
    let time_create = time_start.elapsed();
    let res = solver.solve();
    println!("Solver returned: {:?}", res);
    println!("vars:         {}", solver.num_vars());
    println!("clauses:      {}", solver.num_clauses());
    println!("learnts:      {}", solver.num_learnts());
    println!("decisions:    {}", solver.num_decisions());
    println!("propagations: {}", solver.num_propagations());
    println!("conflicts:    {}", solver.num_conflicts());
    println!("restarts:     {}", solver.num_restarts());
    let time_total = time_start.elapsed();
    println!("time total:      {:?}", time_total);
    println!(
        "time create:     {:?} ({:.2}%)",
        time_create,
        100.0 * time_create.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time search:     {:?} ({:.2}%)",
        solver.time_search,
        100.0 * solver.time_search.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time propagate:  {:?} ({:.2}%)",
        solver.time_propagate,
        100.0 * solver.time_propagate.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time analyze:    {:?} ({:.2}%)",
        solver.time_analyze,
        100.0 * solver.time_analyze.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time backtrack:  {:?} ({:.2}%)",
        solver.time_backtrack,
        100.0 * solver.time_backtrack.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time learn:      {:?} ({:.2}%)",
        solver.time_learn,
        100.0 * solver.time_learn.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time restart:    {:?} ({:.2}%)",
        solver.time_restart,
        100.0 * solver.time_restart.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time pick_dec:   {:?} ({:.2}%)",
        solver.time_pick_decision_var,
        100.0 * solver.time_pick_decision_var.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time decision:   {:?} ({:.2}%)",
        solver.time_decision,
        100.0 * solver.time_decision.as_secs_f64() / time_total.as_secs_f64(),
    );
    println!(
        "time insert_var_order: {:?} ({:.2}%) [{} times]",
        solver.time_insert_var_order,
        100.0 * solver.time_insert_var_order.as_secs_f64() / time_total.as_secs_f64(),
        solver.num_insert_var_order,
    );
    println!(
        "time update_var_order: {:?} ({:.2}%) [{} times]",
        solver.time_update_var_order,
        100.0 * solver.time_update_var_order.as_secs_f64() / time_total.as_secs_f64(),
        solver.num_update_var_order,
    );

    println!("All done in {:?}", time_start.elapsed());
}
