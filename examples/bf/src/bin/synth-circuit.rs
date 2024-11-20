use cadical::statik::Cadical;
use cadical::SolveResponse;
use sat_nexus_core::encoder::CnfEncoder;

use bf::encoding_circuit::encode_circuit_synthesis;
use bf::table::TruthTable;
use bf::utils::*;

fn main() -> color_eyre::Result<()> {
    let n = 3;
    println!("n = {}", n);

    // f = (a & b) | c
    let mut table = TruthTable::new(n);
    table.add_row(vec![false, false, false], false);
    table.add_row(vec![false, false, true], true);
    table.add_row(vec![false, true, false], false);
    table.add_row(vec![false, true, true], true);
    table.add_row(vec![true, false, false], false);
    table.add_row(vec![true, false, true], true);
    table.add_row(vec![true, true, false], true);
    table.add_row(vec![true, true, true], true);
    println!("{:?}", table);
    println!("{}", table.display_simple());

    let tables = vec![table];

    let num_nodes = 2;
    println!("num_nodes = {}", num_nodes);

    println!("Encoding...");
    let mut encoder = CnfEncoder::default();
    let vars = encode_circuit_synthesis(&mut encoder, num_nodes, &tables);
    println!("Encoded using {} variables and {} clauses", encoder.num_vars, encoder.clauses.len());

    // println!("DIMACS:");
    // println!("{}", encoder.to_dimacs());

    println!("Initializing SAT solver...");
    let solver = Cadical::new();

    println!("Adding clauses...");
    for clause in encoder.clauses.iter().cloned() {
        solver.add_clause(clause);
    }

    println!("Solving...");
    let res = solver.solve()?;
    println!("res = {}", res);

    if res == SolveResponse::Sat {
        println!("GATE_TYPE:");
        for (&gate, gate_type_var) in vars.gate_type.iter() {
            let gate_type = decode_onehot(gate_type_var, &solver);
            println!("gate_type[{:?}] = {:?}", gate, gate_type);
        }
        println!("PIN_PARENT:");
        for (&pin, pin_parent_var) in vars.pin_parent.iter() {
            let pin_parent = decode_onehot(pin_parent_var, &solver);
            println!("pin_parent[{:?}] = {:?}", pin, pin_parent);
        }
        println!("VALUE:");
        for (&pin, pin_value_var) in vars.pin_value.iter() {
            for (i, cube) in vars.unique_cubes.iter().enumerate() {
                let pin_value = solver.val(pin_value_var[i].get())?;
                println!(
                    "value[{:?}][{}] = {:?}",
                    pin,
                    cube.iter().map(|&b| if b { '1' } else { '0' }).collect::<String>(),
                    pin_value
                );
            }
        }

        let circuit = vars.build_circuit(&solver);
        println!("CIRCUIT:");
        println!("{:?}", circuit);
        println!("{}", circuit);

        // Write DOT to file "circuit.dot"
        std::fs::write("circuit.dot", circuit.to_dot() + "\n")?;

        // Run "dot -Tpdf -O circuit.dot
        let status = std::process::Command::new("dot")
            .arg("-Tpdf")
            .arg("-O")
            .arg("circuit.dot")
            .status()?;
        println!("{}", status);
    }

    Ok(())
}
