//! Graph coloring example.

use color_eyre::eyre::Result;
use itertools::Itertools;
use ndarray::ArrayD;

use sat_nexus::core::context::Context;
use sat_nexus::core::domainvar::DomainVar;
use sat_nexus::core::op::ops::Ops;
use sat_nexus::core::solver::*;
use sat_nexus::wrappers::ipasir::IpasirSolver;
use sat_nexus_core::solver::delegate::DelegateSolver;

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

fn declare_variables<S>(solver: &mut S, context: &mut Context, num_vertices: usize, num_colors: usize, edges: &[Edge])
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
}

fn declare_constraints<S>(solver: &mut S, context: &mut Context)
where
    S: Solver,
{
    println!("=> Declaring constraints...");

    let num_vertices = *context.extract_named::<usize, _>("num_vertices");
    let num_colors = *context.extract_named::<usize, _>("num_colors");
    let edges = context.extract_named::<Vec<Edge>, _>("edges");
    let color = context.extract_named::<ColorArray, _>("color");

    println!("num_vertices = {}", num_vertices);
    println!("num_colors = {}", num_colors);
    println!("edges = {:?}", edges);
    println!("color = {}", color);

    // (color[a] = c) -> (color[b] != c)
    for &Edge(a, b) in edges.iter() {
        for c in 1..=num_colors {
            solver.imply(color[[a - 1]].eq(c), color[[b - 1]].neq(c));
        }
    }

    // [aux]
    // (color[1] = 1)
    solver.add_clause_(&[color[[0]].eq(1)])
}

fn main() -> Result<()> {
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

    let mut solver = DelegateSolver::new(IpasirSolver::new_cadical());
    println!("solver = {}", solver);

    let mut context = Context::new();
    println!("context = {:?}", context);

    declare_variables(&mut solver, &mut context, num_vertices, num_colors, &edges);
    declare_constraints(&mut solver, &mut context);

    println!("=> Declared {} variables and {} clauses", solver.num_vars(), solver.num_clauses());

    println!("=> Solving...");
    let response = solver.solve();
    println!("=> Solver returned: {:?}", response);

    if matches!(response, SolveResponse::Sat) {
        let color = context.extract_named::<ArrayD<DomainVar<usize>>, _>("color");

        assert!(matches!(solver.eval(&color[[0]].eq(1)), LitValue::True));

        println!("color = {}", solver.eval(color));
        for v in 1..=num_vertices {
            println!("color[{: >2}] = {}", v, color[[v - 1]].eval(&solver));
        }
    }

    Ok(())
}
