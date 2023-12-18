use std::fs::File;
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use clap::Parser;
use itertools::Itertools;
use log::{debug, info};

// Run this example:
// cargo run --release -p backdoor --bin php-gen -- -n 10 -o data/php/php_11_10.cnf

#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// Number of holes for (n+1) pigeons.
    #[arg(short = 'n', value_name = "INT")]
    holes: u32,

    /// Path to a file with output CNF.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    path_output: Option<PathBuf>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    debug!("args = {:?}", args);

    let time_encoding = Instant::now();
    let n = args.holes as i32; // number of holes for (n+1) pigeons
    info!("Encoding PHP({}, {})...", n + 1, n);

    let mut clauses = Vec::new();

    // n+1 clauses which say that a pigeon `i` has to be placed in some hole `j`:
    for i in 1..=(n + 1) {
        let mut clause = Vec::new();
        for j in 1..=n {
            clause.push(n * (i - 1) + j);
        }
        debug!("clause = {:?}", clause);
        clauses.push(clause);
    }

    // For each hole (`j`) we have a set of binary clauses ensuring
    // that only one single pigeon (`i`/`k`) is placed into that hole:
    for j in 1..=n {
        for i in 1..=n {
            for k in (i + 1)..=(n + 1) {
                let clause = vec![-(n * (i - 1) + j), -(n * (k - 1) + j)];
                debug!("clause = {:?}", clause);
                clauses.push(clause);
            }
        }
    }

    let num_vars = n * (n + 1);
    info!("Number of variables: {}", num_vars);
    info!("Number of clauses: {}", clauses.len());

    if let Some(path_output) = &args.path_output {
        info!("Writing to: {}", path_output.display());
        let f = File::create(path_output)?;
        let mut f = LineWriter::new(f);

        writeln!(f, "p cnf {} {}", num_vars, clauses.len())?;
        for clause in clauses {
            writeln!(f, "{} 0", clause.iter().map(|lit| format!("{}", lit)).join(" "))?;
        }
    };

    let time_encoding = time_encoding.elapsed();
    info!("Done encoding PHP({}, {}) in {:.1}s", n + 1, n, time_encoding.as_secs_f64());

    Ok(())
}
