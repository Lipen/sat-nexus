use minisat_sys::dynamic::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    unsafe {
        let ffi = CMiniSatFFI::load("glucose-c");
        println!("ffi created");
        let ptr = ffi.init();
        println!("ptr = {:?}", ptr);

        println!("minisat_l_True = {:?}", ffi.minisat_l_true());
        println!("minisat_l_False = {:?}", ffi.minisat_l_false());
        println!("minisat_l_Undef = {:?}", ffi.minisat_l_undef());

        // Turn off variable elimination
        ffi.minisat_eliminate(ptr, true);

        let lit1 = ffi.minisat_newLit(ptr);
        let lit2 = ffi.minisat_newLit(ptr);
        println!("lit1 = {}", lit1);
        println!("lit2 = {}", lit2);

        // Add clause (1 or 2)
        ffi.minisat_addClause_begin(ptr);
        ffi.minisat_addClause_addLit(ptr, lit1);
        ffi.minisat_addClause_addLit(ptr, lit2);
        ffi.minisat_addClause_commit(ptr);

        let lit3 = ffi.minisat_newLit(ptr);
        let lit4 = ffi.minisat_newLit(ptr);
        println!("lit3 = {}", lit3);
        println!("lit4 = {}", lit4);

        // Add clause (3 or 4)
        ffi.minisat_addClause_begin(ptr);
        ffi.minisat_addClause_addLit(ptr, lit3);
        ffi.minisat_addClause_addLit(ptr, lit4);
        ffi.minisat_addClause_commit(ptr);

        let neg_lit1 = ffi.minisat_negate(lit1);
        let neg_lit2 = ffi.minisat_negate(lit2);
        println!("~lit1 = {}", neg_lit1);
        println!("~lit2 = {}", neg_lit2);

        // Add clause (-1, -2)
        ffi.minisat_addClause_begin(ptr);
        ffi.minisat_addClause_addLit(ptr, neg_lit1);
        ffi.minisat_addClause_addLit(ptr, neg_lit2);
        ffi.minisat_addClause_commit(ptr);

        let neg_lit3 = ffi.minisat_negate(lit3);
        let neg_lit4 = ffi.minisat_negate(lit4);
        println!("~lit3 = {}", neg_lit3);
        println!("~lit4 = {}", neg_lit4);

        // Add clause (-3, -4)
        ffi.minisat_addClause_begin(ptr);
        ffi.minisat_addClause_addLit(ptr, neg_lit3);
        ffi.minisat_addClause_addLit(ptr, neg_lit4);
        ffi.minisat_addClause_commit(ptr);

        // Solving: should be SAT
        ffi.minisat_solve_begin(ptr);
        let result = ffi.minisat_solve_commit(ptr);
        println!("result = {}", result);
        assert!(result);

        println!("Trying literals:");
        for lit in [lit1, lit2, lit3, lit4] {
            let value = ffi.minisat_modelValue_Lit(ptr, lit);
            println!("value_Lit({}) = {:?}", lit, value);
        }
        println!("Trying negative literals:");
        for neg_lit in [neg_lit1, neg_lit2, neg_lit3, neg_lit4] {
            let value = ffi.minisat_modelValue_Lit(ptr, neg_lit);
            println!("value_Lit({}) = {:?}", neg_lit, value);
        }

        // Assuming conflicting literals: should be UNSAT
        println!("begin...");
        ffi.minisat_solve_begin(ptr);
        println!("add lit 1");
        ffi.minisat_solve_addLit(ptr, lit1);
        println!("add lit 2");
        ffi.minisat_solve_addLit(ptr, lit2);
        println!("commit...");
        let result = ffi.minisat_solve_commit(ptr);
        println!("result = {}", result);
        assert!(!result);
    }

    Ok(())
}
