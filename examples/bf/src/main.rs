use bf::encoder::SatEncoder;
use bf::encoding_formula::encode_boolean_synthesis;
use cadical::statik::Cadical;
use cadical::SolveResponse;

use bf::table::TruthTable;
use bf::utils::decode_onehot;

fn main() -> color_eyre::Result<()> {
    let n = 3;
    println!("n = {}", n);

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

    let num_nodes = 5;
    println!("num_nodes = {}", num_nodes);

    println!("Encoding...");
    let mut encoder = SatEncoder::default();
    let vars = encode_boolean_synthesis(&mut encoder, num_nodes, &table);
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
        println!("NODE_TYPE:");
        for (&node, node_type_var) in vars.node_type.iter() {
            let node_type = decode_onehot(node_type_var, &solver);
            println!("node_type[{}] = {:?}", node, node_type);
        }
        println!("INDEX:");
        for (&node, index_var) in vars.index.iter() {
            let index = decode_onehot(index_var, &solver);
            println!("index[{}] = {:?}", node, index);
        }
        println!("PARENT:");
        for (&node, parent_var) in vars.parent.iter() {
            let parent = decode_onehot(parent_var, &solver);
            println!("parent[{}] = {:?}", node, parent);
        }
        println!("CHILD:");
        for (&node, child_var) in vars.child.iter() {
            let child = decode_onehot(child_var, &solver);
            println!("child[{}] = {:?}", node, child);
        }
        // println!("VALUE:");
        // for (&(node, row), &value_var) in vars.value.iter() {
        //     let &(ref inputs, _) = &table.rows[row];
        //     let value = solver.val(value_var)?;
        //     println!(
        //         "value[{}][{}:{}] = {:?}",
        //         node,
        //         row,
        //         inputs.iter().map(|&b| if b { '1' } else { '0' }).collect::<String>(),
        //         value
        //     );
        // }
    }

    let formula = vars.build_formula(&solver);
    println!("FORMULA:");
    println!("{:?}", formula);
    println!("{}", formula);

    Ok(())
}
