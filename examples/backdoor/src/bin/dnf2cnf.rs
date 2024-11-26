use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use backdoor::utils::*;

use bdd_rs::bdd::Bdd;
use simple_sat::lit::Lit;

use clap::{Parser, ValueEnum};
use itertools::Itertools;
use log::{debug, info};

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// Input file with cubes in DNF.
    #[arg(long = "cubes", value_name = "FILE")]
    path_cubes: PathBuf,

    /// Output file with clauses in resulting CNF.
    #[arg(long = "clauses", value_name = "FILE")]
    path_clauses: Option<PathBuf>,

    /// Path to a file with intermediate results.
    #[arg(long = "results", value_name = "FILE")]
    path_results: Option<PathBuf>,

    /// Encoding to use for converting DNF to CNF.
    #[arg(long, value_name = "ENCODING", value_enum)]
    encoding: Encoding,

    /// Number of variables in the original CNF.
    #[arg(long, value_name = "INT")]
    num_vars: Option<u64>,

    /// Number of bits to use for storage in BDD.
    #[arg(long, value_name = "INT", default_value_t = 20)]
    storage_bits: usize,
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
    debug!("args = {:?}", args);

    info!("Reading cubes from '{}'...", args.path_cubes.display());
    let cubes = read_cubes(args.path_cubes);
    info!("Total {} cubes", cubes.len());

    let encoded_clauses: Vec<Vec<Lit>>;

    match args.encoding {
        Encoding::Naive => todo!(),

        Encoding::Tseytin => {
            info!("Using Tseytin method");

            let num_vars_original = args
                .num_vars
                .expect("Number of variables in the original CNF must be provided (via `--num-vars <INT>`) when using Tseytin encoding");
            let mut num_vars = num_vars_original;

            let mut clauses = Vec::new();
            info!("Building CNF via Tseytin-encoding...");
            let mut aux_vars = Vec::new();
            for cube in cubes.iter() {
                num_vars += 1;
                let aux = Lit::from_external(num_vars as i32);
                aux_vars.push(aux);
                clauses.extend(encode_cube_tseytin(aux, cube));
            }
            // Add clause (aux1, aux2, ..., auxN)
            clauses.push(aux_vars.iter().copied().collect());
            info!("Total {} clauses and {} new vars", clauses.len(), aux_vars.len());
            encoded_clauses = clauses;
        }

        Encoding::BddPaths => {
            info!("Using BDD-paths method");

            let bdd = Bdd::new(args.storage_bits);

            let mut cubes_bdd = Vec::new();
            info!("Building BDDs for {} cubes...", cubes.len());
            for cube in cubes.iter() {
                let c = bdd.cube(lits_to_external(cube));
                cubes_bdd.push(c);
            }
            debug!("bdd = {:?}", bdd);

            info!("Merging BDDs for {} cubes via Apply(OR)...", cubes_bdd.len());
            let f = bdd.apply_or_many(cubes_bdd);
            info!("f = {} of size {}", f, bdd.size(f));
            debug!("bdd = {:?}", bdd);

            info!("Building CNF via enumerating all paths to 0...");
            let clauses = bdd_cnf_encode(&bdd, f);
            info!("Total {} clauses", clauses.len());
            encoded_clauses = clauses;
        }

        Encoding::BddTseytin => {
            info!("Using BDD-Tseytin method");

            let num_vars_original = args.num_vars.expect(
                "Number of variables in the original CNF must be provided (via `--num-vars <INT>`) when using BDD-Tseytin encoding",
            );

            let bdd = Bdd::new(args.storage_bits);

            let mut cubes_bdd = Vec::new();
            info!("Building BDDs for {} cubes...", cubes.len());
            for cube in cubes.iter() {
                let c = bdd.cube(lits_to_external(cube));
                cubes_bdd.push(c);
            }
            debug!("bdd = {:?}", bdd);

            info!("Merging BDDs for {} cubes via Apply(OR)...", cubes_bdd.len());
            let f = bdd.apply_or_many(cubes_bdd);
            info!("f = {} of size {}", f, bdd.size(f));
            debug!("bdd = {:?}", bdd);

            info!("Building CNF via Tseytin-encoding the BDD...");
            let (clauses, extra_vars) = bdd_tseytin_encode_ite(&bdd, f, num_vars_original);
            info!("Total {} clauses and {} new vars", clauses.len(), extra_vars.len());
            encoded_clauses = clauses;
        }
    }

    if let Some(path_clauses) = args.path_clauses {
        info!("Writing {} clauses into '{}'...", encoded_clauses.len(), path_clauses.display());
        let file = File::create(path_clauses)?;
        let mut file = LineWriter::new(file);
        for clause in encoded_clauses.iter() {
            for lit in clause.iter() {
                write!(&mut file, "{} ", lit)?;
            }
            writeln!(&mut file, "0")?;
        }
    } else {
        info!("If you want to save the resulting CNF, use `--clauses <FILE>` option!");
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
        if line.starts_with("c") {
            continue;
        }
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

fn encode_cube_tseytin(aux: Lit, cube: &Vec<Lit>) -> Vec<Vec<Lit>> {
    let mut clauses = Vec::with_capacity(cube.len() + 1);

    // Binary clauses (lit, -aux)
    for &lit in cube {
        clauses.push(vec![lit, -aux]);
    }

    // Clause (-lit1, -lit2, ..., -litN, aux)
    clauses.push(cube.iter().map(|&lit| -lit).chain([aux]).collect());

    clauses
}
