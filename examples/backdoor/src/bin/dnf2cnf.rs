use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::Instant;

use backdoor::utils::bdd_cnf_encode;

use bdd_rs::bdd::Bdd;
use simple_sat::lit::Lit;

use clap::{Parser, ValueEnum};
use itertools::Itertools;
use log::info;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// Input file with cubes in DNF.
    #[arg(long = "cubes", value_name = "FILE")]
    path_cubes: PathBuf,

    /// Output file with clauses in resulting CNF.
    #[arg(long = "clauses", value_name = "FILE")]
    path_clauses: Option<PathBuf>,

    /// Encoding to use for converting DNF to CNF.
    #[arg(long, default_value = "tseytin")]
    encoding: Encoding,

    /// Number of variables in the original CNF.
    #[arg(long)]
    num_vars: Option<u64>,

    /// Path to a file with intermediate results.
    #[arg(long = "results", value_name = "FILE")]
    path_results: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Copy, Clone)]
enum Encoding {
    Naive,
    Tseytin,
    BddPaths,
    BddTseytin,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug,bdd_rs::bdd=info")).init();

    let start_time = Instant::now();
    let args = Cli::parse();
    info!("args = {:?}", args);

    info!("Reading cubes from '{}'...", args.path_cubes.display());
    let cubes = read_cubes(args.path_cubes);
    info!("Total {} cubes", cubes.len());

    // let mut clauses = Vec::new();

    // if false {
    //     let mut hard2_clauses = Vec::new();
    //     for cube in hard2_cubes.iter() {
    //         num_vars += 1;
    //         let aux = Lit::from_external(num_vars);
    //         hard2_clauses.extend(encode_clause_tseytin(aux, cube));
    //     }
    //     println!(
    //         "Tseytin-encoded {} hard cubes in the second part into {} clauses with {} aux vars",
    //         hard2_cubes.len(),
    //         hard2_clauses.len(),
    //         num_vars - max_var
    //     );
    //     clauses.extend(hard2_clauses);
    // }
    // if false {
    //     let path_output = "data/2024-08-07/out_orig_with_cubes2_tseytin.cnf";
    //     println!(
    //         "Writing CNF with {} vars and {} clauses to '{}'",
    //         num_vars,
    //         clauses.len(),
    //         path_output
    //     );
    //     let file = File::create(path_output)?;
    //     let mut writer = BufWriter::new(&file);
    //     writeln!(writer, "p cnf {} {}", num_vars, clauses.len())?;
    //     for clause in clauses.iter() {
    //         for &lit in clause.iter() {
    //             write!(writer, "{} ", lit)?;
    //         }
    //         writeln!(writer, "0")?;
    //     }
    // }
    //
    // if true {
    //     let bdd = Bdd::new(24);
    //     let mut hard2_cubes_bdds = Vec::new();
    //     for cube in hard2_cubes.iter() {
    //         let c = bdd.cube(cube.iter().map(|lit| lit.to_external()));
    //         hard2_cubes_bdds.push(c);
    //     }
    //     let f = bdd.apply_or_many(hard2_cubes_bdds.iter().copied());
    //     println!("f = {} of size {}", f, bdd.size(f));
    //     let hard2_clauses = bdd_cnf_encode(&bdd, f);
    //     println!(
    //         "BDD-encoded {} hard cubes in the second part into {} clauses",
    //         hard2_cubes.len(),
    //         hard2_clauses.len()
    //     );
    //     clauses.extend(hard2_clauses);
    // }

    // let path_output = "data/2024-08-07/out.cnf";
    // if true {
    //     println!(
    //         "Writing CNF with {} vars and {} clauses to '{}'",
    //         num_vars,
    //         clauses.len(),
    //         path_output
    //     );
    //     let file = File::create(path_output)?;
    //     let mut writer = BufWriter::new(&file);
    //     writeln!(writer, "p cnf {} {}", num_vars, clauses.len())?;
    //     for clause in clauses.iter() {
    //         write_clause(&mut writer, clause)?;
    //     }
    // }

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
            if let Some(&0) = cube.last() {
                assert_eq!(cube.pop().unwrap(), 0);
            }
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
