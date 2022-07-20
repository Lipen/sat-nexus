use crate::formula::expr::Expr;
use crate::formula::simplify::simplify;
use crate::formula::var::Var;
use crate::lit::Lit;
use crate::solver::Solver;

pub fn add_constraint<S>(solver: &mut S, expr: &Expr<Var>)
where
    S: Solver,
{
    let expr = simplify(expr.clone());
    let nnf = expr.to_nnf();
    let cnf = nnf.to_cnf(&mut || solver.new_var().get());
    for clause in cnf.0 .0 {
        solver.add_clause(clause.0.into_iter().map(|x| Lit::new(x)))
    }
}
