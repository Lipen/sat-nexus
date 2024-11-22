use kissat::dynamic::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let solver = Kissat::new();
    // static FFI: OnceLock<KissatFFI> = OnceLock::new();
    // let ffi = FFI.get_or_init(|| unsafe { KissatFFI::new("C:/lib/kissat.dll").unwrap() });
    // let solver = Kissat::new_custom(ffi);
    println!("Solver signature: {}", solver.signature());
    println!("solver = {}", solver);

    solver.add_clause([1, 2]);
    solver.add_clause(vec![3, 4]);
    solver.add_clause([-1, -2]);
    solver.add_clause(vec![-3, -4]);
    solver.add_clause([5, -5]);

    let response = solver.solve();
    println!("solve() = {:?}", response);
    assert!(matches!(response, SolveResponse::Sat));

    for i in 1..=5 {
        println!("value({}) = {:?}", i, solver.value(i));
    }

    Ok(())
}
