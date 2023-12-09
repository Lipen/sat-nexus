use std::collections::HashMap;
use std::fs::File;
use std::io::{LineWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use itertools::Itertools;
use log::{debug, info, trace};

use backdoor::algorithm::{Algorithm, Options, DEFAULT_OPTIONS};
use backdoor::utils::partition_tasks;
use simple_sat::lit::Lit;
use simple_sat::solver::Solver;
use simple_sat::utils::DisplaySlice;
use simple_sat::var::Var;

// Run this example:
// cargo run -p backdoor --bin search -- data/mult/lec_CvK_12.cnf --backdoor-size 10 --num-iters 1000

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    #[arg(value_name = "CNF")]
    path_cnf: PathBuf,

    /// Backdoor size.
    #[arg(long, value_name = "INT")]
    backdoor_size: usize,

    /// Number of EA iterations.
    #[arg(long, value_name = "INT")]
    num_iters: usize,

    /// Number of EA runs.
    #[arg(long, value_name = "INT", default_value_t = 1)]
    num_runs: usize,

    /// Path to a file with results.
    #[arg(long = "results", value_name = "FILE")]
    path_results: Option<PathBuf>,

    /// Random seed.
    #[arg(long, value_name = "INT", default_value_t = DEFAULT_OPTIONS.seed)]
    seed: u64,

    /// Do ban variables used in best backdoors on previous runs?
    #[arg(long)]
    ban_used: bool,

    /// Comma-separated list of banned variables (1-based indices).
    #[arg(long, value_name = "INT...")]
    bans: Option<String>,

    /// Number of stagnated iterations before re-initialization.
    #[arg(long, value_name = "INT")]
    stagnation_limit: Option<usize>,

    /// Maximum required rho value (break EA upon reaching).
    #[arg(long, value_name = "FLOAT", default_value_t = 1.0)]
    max_rho: f64,

    /// Minimum number of EA iterations.
    #[arg(long, value_name = "INT", default_value_t = 0)]
    min_iter: usize,

    /// Do dump records for each EA run?
    #[arg(long)]
    dump_records: bool,

    /// Do add learnts after analyzing conflicts in `propcheck_all_tree`?
    #[arg(long)]
    add_learnts: bool,

    /// Do dump learnts after each EA run?
    #[arg(long)]
    dump_intermediate_learnts: bool,

    /// Do dump all learnts after all EA runs?
    #[arg(long)]
    dump_learnts: bool,

    /// Do derive clauses from backdoors?
    #[arg(long)]
    derive: bool,

    /// Do dump derived clauses after each EA run?
    #[arg(long)]
    dump_derived: bool,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,simple_sat::solver=info")).init();

    let start_time = Instant::now();
    let args = Cli::parse();
    info!("args = {:?}", args);

    // Initialize the SAT solver:
    let mut solver = Solver::default();
    solver.init_from_file(&args.path_cnf);

    // Setup the evolutionary algorithm:
    let options = Options {
        seed: args.seed,
        add_learnts_in_propcheck_all_tree: args.add_learnts,
        ban_used_variables: args.ban_used,
        ..DEFAULT_OPTIONS
    };
    let mut algorithm = Algorithm::new(solver, options);

    // Bans some variables:
    if let Some(bans) = args.bans {
        let bans = parse_comma_separated_intervals(&bans);
        trace!("bans = {:?}", bans);
        for i in bans {
            assert!(i > 0);
            algorithm.banned[i - 1] = true;
        }
    }

    // Create and open the file with results:
    let mut file_results = if let Some(path_results) = &args.path_results {
        let f = File::create(path_results)?;
        let f = LineWriter::new(f);
        Some(f)
    } else {
        None
    };

    let mut file_derived_clauses = if args.dump_derived {
        let f = File::create("derived_clauses.txt")?;
        let f = LineWriter::new(f);
        Some(f)
    } else {
        None
    };

    for run_number in 1..=args.num_runs {
        info!("EA run {} / {}", run_number, args.num_runs);

        debug!("algorithm.derived_clauses.len() = {}", algorithm.derived_clauses.len());
        debug!("algorithm.learnt_clauses.len() = {}", algorithm.learnt_clauses.len());

        // Run the evolutionary algorithm:
        let result = algorithm.run(
            args.backdoor_size,
            args.num_iters,
            args.stagnation_limit,
            Some(args.max_rho),
            args.min_iter,
        );

        assert!(result.best_fitness.num_hard > 0, "Found strong backdoor?!..");

        // Derive clauses from the best backdoor:
        if args.derive {
            let backdoor = result.best_instance.get_variables();
            let (hard, easy) = partition_tasks(&backdoor, &mut algorithm.solver);
            debug!("Backdoor has {} hard and {} easy tasks", hard.len(), easy.len());

            let mut derived_clauses = Vec::new();

            // count :: {Var: (pos, neg)}
            let mut count = HashMap::<Var, (u64, u64)>::new();
            for cube in hard.iter() {
                for &lit in cube.iter() {
                    let e = count.entry(lit.var()).or_default();
                    if lit.negated() {
                        (*e).1 += 1;
                    } else {
                        (*e).0 += 1;
                    }
                }
            }
            for (&var, &(pos, neg)) in count.iter() {
                info!("Count (pos/neg) for {} is {} / {}", var, pos, neg);
            }
            for (&var, &(pos, neg)) in count.iter() {
                if pos == 0 {
                    info!("variable {} is always negative", var);
                    derived_clauses.push(vec![Lit::new(var, true)]);
                }
                if neg == 0 {
                    info!("variable {} is always positive", var);
                    derived_clauses.push(vec![Lit::new(var, false)]);
                }
            }

            // count_pair :: {(a, b): (+a+b, +a-b, -a+b, -a-b)}
            let mut count_pair = HashMap::<(Var, Var), (u64, u64, u64, u64)>::new();
            for cube in hard.iter() {
                for i in 0..cube.len() {
                    let a = cube[i];
                    if count[&a.var()].0 == 0 || count[&a.var()].1 == 0 {
                        continue;
                    }
                    for j in (i + 1)..cube.len() {
                        let b = cube[j];
                        if count[&b.var()].0 == 0 || count[&b.var()].1 == 0 {
                            continue;
                        }
                        let e = count_pair.entry((a.var(), b.var())).or_default();
                        match (a.negated(), b.negated()) {
                            (false, false) => (*e).0 += 1, // pos-pos
                            (false, true) => (*e).1 += 1,  // pos-neg
                            (true, false) => (*e).2 += 1,  // neg-pos
                            (true, true) => (*e).3 += 1,   // neg-neg
                        }
                    }
                }
            }
            for (&(a, b), &(pp, pn, np, nn)) in count_pair.iter() {
                info!("Count (pp/pn/np/nn) for {}-{} is {} / {} / {} / {}", a, b, pp, pn, np, nn);
            }
            for (&(a, b), &(pp, pn, np, nn)) in count_pair.iter() {
                if pp == 0 {
                    info!(
                        "pair {}-{} is never pos-pos |= clause ({}, {})",
                        a,
                        b,
                        Lit::new(a, true),
                        Lit::new(b, true)
                    );
                    derived_clauses.push(vec![Lit::new(a, true), Lit::new(b, true)]);
                }
                if pn == 0 {
                    info!(
                        "pair {}-{} is never pos-neg |= clause ({}, {})",
                        a,
                        b,
                        Lit::new(a, true),
                        Lit::new(b, false)
                    );
                    derived_clauses.push(vec![Lit::new(a, true), Lit::new(b, false)]);
                }
                if np == 0 {
                    info!(
                        "pair {}-{} is never neg-pos |= clause ({}, {})",
                        a,
                        b,
                        Lit::new(a, false),
                        Lit::new(b, true)
                    );
                    derived_clauses.push(vec![Lit::new(a, false), Lit::new(b, true)]);
                }
                if nn == 0 {
                    info!(
                        "pair {}-{} is never neg-neg |= clause ({}, {})",
                        a,
                        b,
                        Lit::new(a, false),
                        Lit::new(b, false)
                    );
                    derived_clauses.push(vec![Lit::new(a, false), Lit::new(b, false)]);
                }
            }

            debug!(
                "Total {} derived clauses: [{}]",
                derived_clauses.len(),
                derived_clauses.iter().map(|c| DisplaySlice(c)).join(", ")
            );

            // Add the derived clauses as learnts:
            for lemma in derived_clauses.iter() {
                algorithm.solver.add_learnt(lemma);

                if let Some(f) = &mut file_derived_clauses {
                    for lit in lemma.iter() {
                        write!(f, "{} ", lit)?;
                    }
                    writeln!(f, "0")?;
                }
            }

            for mut lemma in derived_clauses {
                lemma.sort_by_key(|lit| lit.var().0);
                algorithm.derived_clauses.insert(lemma);
            }
        }

        // Dump learnts:
        if args.dump_intermediate_learnts {
            let f = File::create(format!("learnts_{}.txt", run_number))?;
            let mut f = LineWriter::new(f);
            for learnt in algorithm.solver.learnts_iter() {
                for lit in learnt.iter() {
                    write!(f, "{} ", lit)?;
                }
                writeln!(f, " 0")?;
            }
        }

        // Write the best found backdoor into the resulting file:
        if let Some(f) = &mut file_results {
            // Note: variables in backdoors are reported 1-based.
            writeln!(
                f,
                "Backdoor [{}] of size {} on iter {} with fitness = {}, rho = {}, hard = {} in {:.3} ms",
                result.best_instance.get_variables().iter().map(|v| v.0 + 1).join(", "),
                result.best_instance.weight(),
                result.best_iteration,
                result.best_fitness.value,
                result.best_fitness.rho,
                result.best_fitness.num_hard,
                result.time.as_secs_f64() * 1000.0
            )?;
        }

        // Write the run records:
        if args.dump_records {
            let mut writer = csv::Writer::from_path(format!("run_{}.csv", run_number))?;
            writer.write_record(&["iteration", "instance", "fitness", "num_hard", "rho"])?;
            for record in result.records {
                writer.serialize((
                    record.iteration,
                    record.instance.get_variables().iter().map(|v| v.to_external()).join(","),
                    record.fitness.value,
                    record.fitness.num_hard,
                    record.fitness.rho,
                ))?;
            }
        }
    }

    // Dump all learnts:
    if args.dump_learnts {
        let f = File::create("learnt_clauses.txt")?;
        let mut f = LineWriter::new(f);
        for lemma in algorithm.learnt_clauses.iter() {
            for lit in lemma.iter() {
                write!(f, "{} ", lit)?;
            }
            writeln!(f, "0")?;
        }
    }

    let elapsed = Instant::now() - start_time;
    println!("\nAll done in {:.3} s", elapsed.as_secs_f64());
    Ok(())
}

fn parse_comma_separated_intervals(input: &str) -> Vec<usize> {
    let mut result = Vec::new();
    for part in input.split(',') {
        let range_parts: Vec<&str> = part.splitn(2, "-").collect();
        if range_parts.len() == 2 {
            let start: usize = range_parts[0].parse().unwrap();
            let end: usize = range_parts[1].parse().unwrap();
            if start <= end {
                result.extend(start..=end);
            } else {
                result.extend((end..=start).rev());
            }
        } else {
            let single: usize = part.parse().unwrap();
            result.push(single);
        }
    }
    result
}
