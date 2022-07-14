use minisat_sys::statik::bindings::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    unsafe {
        let ptr = minisat_new();
        println!("ptr = {:?}", ptr);

        println!("minisat_l_True = {:?}", minisat_l_True);
        println!("minisat_l_False = {:?}", minisat_l_False);
        println!("minisat_l_Undef = {:?}", minisat_l_Undef);

        // Turn off variable elimination
        minisat_eliminate(ptr, true);

        let lit1 = minisat_newLit(ptr);
        let lit2 = minisat_newLit(ptr);
        println!("lit1 = {}", lit1);
        println!("lit2 = {}", lit2);

        // Add clause (1 or 2)
        minisat_addClause_begin(ptr);
        minisat_addClause_addLit(ptr, lit1);
        minisat_addClause_addLit(ptr, lit2);
        minisat_addClause_commit(ptr);

        let lit3 = minisat_newLit(ptr);
        let lit4 = minisat_newLit(ptr);
        println!("lit3 = {}", lit3);
        println!("lit4 = {}", lit4);

        // Add clause (3 or 4)
        minisat_addClause_begin(ptr);
        minisat_addClause_addLit(ptr, lit3);
        minisat_addClause_addLit(ptr, lit4);
        minisat_addClause_commit(ptr);

        let neg_lit1 = minisat_negate(lit1);
        let neg_lit2 = minisat_negate(lit2);
        println!("~lit1 = {}", neg_lit1);
        println!("~lit2 = {}", neg_lit2);

        // Add clause (-1, -2)
        minisat_addClause_begin(ptr);
        minisat_addClause_addLit(ptr, neg_lit1);
        minisat_addClause_addLit(ptr, neg_lit2);
        minisat_addClause_commit(ptr);

        let neg_lit3 = minisat_negate(lit3);
        let neg_lit4 = minisat_negate(lit4);
        println!("~lit3 = {}", neg_lit3);
        println!("~lit4 = {}", neg_lit4);

        // Add clause (-3, -4)
        minisat_addClause_begin(ptr);
        minisat_addClause_addLit(ptr, neg_lit3);
        minisat_addClause_addLit(ptr, neg_lit4);
        minisat_addClause_commit(ptr);

        // Solving: should be SAT
        minisat_solve_begin(ptr);
        let result = minisat_solve_commit(ptr);
        println!("result = {}", result);
        assert!(result);

        println!("Trying literals:");
        for lit in [lit1, lit2, lit3, lit4] {
            let value = minisat_modelValue_Lit(ptr, lit);
            println!("value_Lit({}) = {:?}", lit, value);
        }
        println!("Trying negative literals:");
        for neg_lit in [neg_lit1, neg_lit2, neg_lit3, neg_lit4] {
            let value = minisat_modelValue_Lit(ptr, neg_lit);
            println!("value_Lit({}) = {:?}", neg_lit, value);
        }

        // Assuming conflicting literals: should be UNSAT
        minisat_solve_begin(ptr);
        minisat_solve_addLit(ptr, lit1);
        minisat_solve_addLit(ptr, lit2);
        let result = minisat_solve_commit(ptr);
        println!("result = {}", result);
        assert!(!result);
    }
    Ok(())
}
