use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;
use std::time::Instant;

use backdoor::derivation::derive_clauses;
use backdoor::pool::{CubeTask, SolverPool};
use backdoor::utils::{bdd_cnf_encode, lits_to_external, write_clause};

use bdd_rs::bdd::Bdd;
use cadical::statik::Cadical;
use cadical::SolveResponse;
use simple_sat::lit::Lit;

use indicatif::{ProgressBar, ProgressIterator};
use itertools::Itertools;
use log::info;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,bdd_rs::bdd=info,backdoor::derivation=info")).init();

    let start_time = Instant::now();

    let path_original = "data/2024-08-07/original.cnf";
    println!("Reading original CNF from '{}'", path_original);
    let clauses_original = simple_sat::utils::parse_dimacs(path_original).collect_vec();
    println!("Original clauses: {}", clauses_original.len());
    let max_var = clauses_original
        .iter()
        .flat_map(|c| c.iter())
        .map(|l| l.to_external().abs())
        .max()
        .unwrap();
    println!("Number of variables: {}", max_var);

    let path_hard1 = "data/2024-08-07/cubes1.txt";
    let hard1_cubes = read_cubes(path_hard1);
    println!("Hard cubes in the first part: {}", hard1_cubes.len());

    println!(
        "Variables in the first part: [{}]",
        hard1_cubes[0].iter().map(|lit| lit.var()).join(", ")
    );

    let mut clauses = clauses_original.clone();
    let mut num_vars = max_var;

    println!("Deriving clauses from hard cubes in the first part...");
    let time_derive = Instant::now();
    let derived1 = derive_clauses(&hard1_cubes, false);
    let time_derive = time_derive.elapsed();
    println!(
        "Derived {} clauses ({} units, {} binary, {} ternary, {} other) for {} cubes in {:.1}s",
        derived1.len(),
        derived1.iter().filter(|c| c.len() == 1).count(),
        derived1.iter().filter(|c| c.len() == 2).count(),
        derived1.iter().filter(|c| c.len() == 3).count(),
        derived1.iter().filter(|c| c.len() > 2).count(),
        hard1_cubes.len(),
        time_derive.as_secs_f64()
    );
    clauses.extend(derived1);

    let path_hard2 = "data/2024-08-07/cubes2.txt";
    let hard2_cubes = read_cubes(path_hard2);
    println!("Hard cubes in the seconds part: {}", hard2_cubes.len());

    println!(
        "Variables in the second part: [{}]",
        hard2_cubes[0].iter().map(|lit| lit.var()).join(", ")
    );

    if false {
        let mut hard2_clauses = Vec::new();
        for cube in hard2_cubes.iter() {
            num_vars += 1;
            let aux = Lit::from_external(num_vars);
            hard2_clauses.extend(encode_clause_tseytin(aux, cube));
        }
        println!(
            "Tseytin-encoded {} hard cubes in the second part into {} clauses with {} aux vars",
            hard2_cubes.len(),
            hard2_clauses.len(),
            num_vars - max_var
        );
        clauses.extend(hard2_clauses);
    }
    if false {
        let path_output = "data/2024-08-07/out_orig_with_cubes2_tseytin.cnf";
        println!(
            "Writing CNF with {} vars and {} clauses to '{}'",
            num_vars,
            clauses.len(),
            path_output
        );
        let file = File::create(path_output)?;
        let mut writer = BufWriter::new(&file);
        writeln!(writer, "p cnf {} {}", num_vars, clauses.len())?;
        for clause in clauses.iter() {
            for &lit in clause.iter() {
                write!(writer, "{} ", lit)?;
            }
            writeln!(writer, "0")?;
        }
    }

    if true {
        let bdd = Bdd::new(24);
        let mut hard2_cubes_bdds = Vec::new();
        for cube in hard2_cubes.iter() {
            let c = bdd.cube(cube.iter().map(|lit| lit.to_external()));
            hard2_cubes_bdds.push(c);
        }
        let f = bdd.apply_or_many(hard2_cubes_bdds.iter().copied());
        println!("f = {} of size {}", f, bdd.size(f));
        let hard2_clauses = bdd_cnf_encode(&bdd, f);
        println!(
            "BDD-encoded {} hard cubes in the second part into {} clauses",
            hard2_cubes.len(),
            hard2_clauses.len()
        );
        clauses.extend(hard2_clauses);
    }

    let path_output = "data/2024-08-07/out.cnf";
    if true {
        println!(
            "Writing CNF with {} vars and {} clauses to '{}'",
            num_vars,
            clauses.len(),
            path_output
        );
        let file = File::create(path_output)?;
        let mut writer = BufWriter::new(&file);
        writeln!(writer, "p cnf {} {}", num_vars, clauses.len())?;
        for clause in clauses.iter() {
            write_clause(&mut writer, clause)?;
        }
    }

    // let path_cubes1 = "data/2024-08-07/cubes1_inv1.txt";
    // let cubes1 = read_cubes(path_cubes1);
    let cubes1 = hard1_cubes;
    println!("cubes1.len() = {}", cubes1.len());

    let is_parallel = false;

    if !is_parallel {
        println!("Spawning CaDiCaL");
        let solver = Cadical::new();

        for clause in clauses.iter() {
            solver.add_clause(lits_to_external(clause));
        }

        let num_conflicts = 1000000;
        println!(
            "Solving {} hard cubes in the first part using CaDiCaL with {} conflicts per task",
            cubes1.len(),
            num_conflicts
        );
        let mut total_unsat = 0;
        let mut total_unknown = 0;
        for (i, cube) in cubes1.iter().enumerate() {
            // println!("Checking hard cube {}/{}: {}", i+1, hard1_cubes.len(), display_slice(cube));
            solver.reset_assumptions();
            for &lit in cube.iter() {
                solver.assume(lit.to_external())?;
            }
            solver.limit("conflicts", num_conflicts);
            let num_conflicts_before = solver.conflicts() as u64;
            let time_solve = Instant::now();
            let res = solver.solve()?;
            let time_solve = time_solve.elapsed();
            let num_conflicts_after = solver.conflicts() as u64;
            let num_conflicts = num_conflicts_after - num_conflicts_before;
            match res {
                SolveResponse::Sat => panic!("Unexpected SAT"),
                SolveResponse::Unsat => total_unsat += 1,
                SolveResponse::Interrupted => total_unknown += 1,
            }
            println!(
                "[{}/{}]: {} in {:.3}s after {} conflicts. So far {} UNSAT, {} UNKNOWN",
                i + 1,
                cubes1.len(),
                res2str(res),
                time_solve.as_secs_f64(),
                num_conflicts,
                total_unsat,
                total_unknown
            );
        }
        println!("Checked {} cubes, {} UNSAT, {} UNKNOWN", cubes1.len(), total_unsat, total_unknown);
    }

    if is_parallel {
        let pool = SolverPool::<CubeTask>::new_from(32, path_output, |task, solver| task.solve_with(solver));
        let mut num_tasks = 0;
        for (i, cube) in cubes1.iter().enumerate() {
            pool.submit(CubeTask::new(i, cube.clone()));
            num_tasks += 1;
        }
        info!("Submitted {} tasks", num_tasks);

        info!("Joining...");
        let pb = ProgressBar::new(num_tasks as u64);
        let results: Vec<_> = pool.join().take(num_tasks).progress_with(pb).collect();
        info!("Got {} results", results.len());

        info!(
            "Total CPU time: {:.1}s",
            results.iter().map(|(_, _, time)| time.as_secs_f64()).sum::<f64>()
        );
        pool.finish();
    }

    let total_time = start_time.elapsed();
    println!("\nAll done in {:.3} s", total_time.as_secs_f64());
    Ok(())
}

fn read_cubes(path: impl AsRef<Path>) -> Vec<Vec<Lit>> {
    let file = File::open(path).expect("Unable to open file");
    let reader = BufReader::new(file);
    let mut cubes = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Unable to read line").trim().to_string();
        if line.starts_with("a") {
            let mut cube: Vec<i32> = line
                .split_whitespace()
                .skip(1)
                .map(|x| x.parse::<i32>().expect("Unable to parse integer"))
                .collect_vec();
            assert_eq!(cube.pop().expect("List must not be empty"), 0);
            cubes.push(cube.into_iter().map(Lit::from_external).collect());
        }
    }
    cubes
}

fn encode_clause_tseytin(aux: Lit, clause: &Vec<Lit>) -> Vec<Vec<Lit>> {
    let mut clauses = Vec::with_capacity(clause.len() + 1);
    for &lit in clause {
        clauses.push(vec![-lit, aux]);
    }
    clauses.push(clause.iter().copied().chain([-aux]).collect());
    clauses
}

fn res2str(res: SolveResponse) -> &'static str {
    match res {
        SolveResponse::Sat => "SAT",
        SolveResponse::Unsat => "UNSAT",
        SolveResponse::Interrupted => "UNKNOWN",
    }
}
