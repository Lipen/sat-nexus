//! Graph coloring example.

use itertools::Itertools;
use ndarray::ArrayD;

use sat_nexus_core::context::Context;
use sat_nexus_core::domainvar::DomainVar;
use sat_nexus_core::formula::constraint::add_constraint;
use sat_nexus_core::formula::expr::Expr;
use sat_nexus_core::formula::var::Var;
use sat_nexus_core::lit::Lit;
use sat_nexus_core::op::ops::Ops;
use sat_nexus_core::solver::ext::SolverExt;
use sat_nexus_core::solver::*;
use sat_nexus_wrappers::cadical_dynamic::CadicalDynamicSolver;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Edge(usize, usize);

impl Edge {
    fn normalize(self) -> Edge {
        let Edge(a, b) = self;
        if a <= b {
            Edge(a, b)
        } else {
            Edge(b, a)
        }
    }
}

type ColorArray = ArrayD<DomainVar<usize>>;

fn declare_variables<S>(
    solver: &mut S,
    context: &mut Context,
    num_vertices: usize,
    num_colors: usize,
    edges: &[Edge],
) -> color_eyre::Result<()>
where
    S: Solver,
{
    assert!(num_vertices > 0, "Number of vertices must be positive");
    assert!(num_colors > 0, "Number of colors must be positive");
    assert!(!edges.is_empty(), "No edges");

    println!("=> Declaring variables...");

    let edges = edges.iter().map(|&e| e.normalize()).unique().sorted().collect_vec();

    println!("num_vertices = {}", num_vertices);
    println!("num_colors = {}", num_colors);
    println!("edges = {:?}", edges);

    context.insert_named("num_vertices", num_vertices);
    context.insert_named("num_colors", num_colors);
    #[allow(clippy::redundant_clone)]
    context.insert_named("edges", edges.clone());

    let color: ColorArray = solver.new_domain_var_array_dyn([num_vertices], |_| 1..=num_colors);
    context.insert_named("color", color);

    Ok(())
}

fn declare_constraints<S>(solver: &mut S, context: &mut Context) -> color_eyre::Result<()>
where
    S: Solver,
{
    println!("=> Declaring constraints...");

    let num_vertices = *context.get_named::<usize, _>("num_vertices")?;
    let num_colors = *context.get_named::<usize, _>("num_colors")?;
    let edges = context.get_named::<Vec<Edge>, _>("edges")?;
    let color = context.get_named::<ColorArray, _>("color")?;

    println!("num_vertices = {}", num_vertices);
    println!("num_colors = {}", num_colors);
    println!("edges = {:?}", edges);
    println!("color = {}", color);

    static USE_CONSTRAINT: bool = true;

    fn lit_to_var(lit: Lit) -> Expr<Var> {
        let e = Expr::from(Var(lit.var()));
        if lit.get() < 0 {
            !e
        } else {
            e
        }
    }

    // (color[a] = c) -> (color[b] != c)
    for &Edge(a, b) in edges.iter() {
        for c in 1..=num_colors {
            if USE_CONSTRAINT {
                add_constraint(
                    solver,
                    Expr::imply(lit_to_var(color[[a - 1]].eq(c)), lit_to_var(color[[b - 1]].neq(c))),
                )
            } else {
                solver.imply(color[[a - 1]].eq(c), color[[b - 1]].neq(c));
            }
        }
    }

    // [aux]
    // (color[1] = 1)
    solver.add_unit(color[[0]].eq(1));

    Ok(())
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    println!("==> Graph coloring example");

    let num_vertices = 10;
    let num_colors = 3;
    let edges = vec![
        Edge(1, 3),
        Edge(3, 5),
        Edge(5, 2),
        Edge(2, 4),
        Edge(4, 1),
        Edge(1, 6),
        Edge(2, 7),
        Edge(3, 8),
        Edge(4, 9),
        Edge(5, 10),
        Edge(6, 7),
        Edge(7, 8),
        Edge(8, 9),
        Edge(9, 10),
        Edge(10, 6),
    ];

    let mut solver = CadicalDynamicSolver::new();
    println!("solver = {}", solver);

    let mut context = Context::new();
    println!("context = {:?}", context);

    declare_variables(&mut solver, &mut context, num_vertices, num_colors, &edges)?;
    declare_constraints(&mut solver, &mut context)?;

    println!("=> Declared {} variables and {} clauses", solver.num_vars(), solver.num_clauses());

    println!("=> Solving...");
    let response = solver.solve();
    println!("=> Solver returned: {:?}", response);

    if matches!(response, SolveResponse::Sat) {
        let color = context.get_named::<ArrayD<DomainVar<usize>>, _>("color")?;

        assert!(matches!(solver.eval(&color[[0]].eq(1)), LitValue::True));

        println!("color = {}", solver.eval(color));
        for v in 1..=num_vertices {
            println!("color[{: >2}] = {}", v, color[[v - 1]].eval(&solver));
        }

        println!("=> Checking coloring...");
        for &Edge(a, b) in edges.iter() {
            assert_ne!(color[[a - 1]].eval(&solver), color[[b - 1]].eval(&solver),);
        }
        println!("=> Coloring: OK");
    }

    Ok(())
}
