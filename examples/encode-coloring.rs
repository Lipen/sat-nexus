//! Graph coloring example.

use std::collections::HashMap;
use std::time::Instant;

use itertools::Itertools;

use sat_nexus_core::encoder::CnfEncoder;
use sat_nexus_core::lit::Lit;
use sat_nexus_core::map::Map;
use sat_nexus_core::solver::{LitValue, SolveResponse, Solver};
use sat_nexus_wrappers::cadical_static::CadicalStaticSolver as Cadical;

type DirectVar<T> = Map<T, Lit>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Vertex(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Color(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Edge(Vertex, Vertex);

impl Edge {
    pub fn new(a: usize, b: usize) -> Self {
        if a <= b {
            Self(Vertex(a), Vertex(b))
        } else {
            Self(Vertex(b), Vertex(a))
        }
    }
}

#[allow(dead_code)]
struct Coloring {
    num_vertices: usize,
    num_colors: usize,
    color: Map<Vertex, DirectVar<Color>>,
}

impl Coloring {
    pub fn encode(encoder: &mut CnfEncoder, edges: &[Edge], num_vertices: usize, num_colors: usize) -> Self {
        assert!(!edges.is_empty(), "No edges");
        assert!(num_vertices > 0, "Number of vertices must be positive");
        assert!(num_colors > 0, "Number of colors must be positive");

        let time_encode = Instant::now();
        println!(
            "=> Encoding graph coloring problem: {} vertices, {} edges, {} colors",
            num_vertices,
            edges.len(),
            num_colors
        );

        println!("=> Declaring variables...");

        let mut color_vars = Map::default();
        for v in (1..=num_vertices).map(Vertex) {
            let possible_colors = (1..=num_colors).map(Color).collect_vec();
            let color_var = encoder.new_direct(possible_colors);
            encoder.exactly_one(color_var.values());
            color_vars.add(v, color_var);
        }

        println!("=> Declaring constraints...");

        // For each edge (a, b) and each color c:
        //   (color[a] = c) -> (color[b] != c)
        for &Edge(a, b) in edges.iter() {
            for color in (1..=num_colors).map(Color) {
                let ca = color_vars[a][color];
                let cb = color_vars[b][color];
                encoder.add_clause(vec![-ca, -cb]);
            }
        }

        // The first vertex is colored with the first color:
        // (color[1] = 1)
        {
            let v1 = Vertex(1);
            let c1 = Color(1);
            let c = color_vars[v1][c1];
            encoder.add_clause(vec![c]);
        }

        let time_encode = time_encode.elapsed();
        println!(
            "=> Encoded using {} variables and {} clauses in {:.1} s",
            encoder.num_vars,
            encoder.clauses.len(),
            time_encode.as_secs_f64()
        );

        Self {
            num_vertices,
            num_colors,
            color: color_vars,
        }
    }
}

pub fn decode_onehot<'a, T>(var: &'a Map<T, Lit>, solver: &Cadical) -> Option<&'a T> {
    var.iter()
        .find_map(|(key, &t)| if solver.value(t) == LitValue::True { Some(key) } else { None })
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    println!("==> Graph coloring example");
    let time_total = Instant::now();

    let num_vertices = 10;
    let num_colors = 3;
    let edges = vec![
        Edge::new(1, 3),
        Edge::new(3, 5),
        Edge::new(5, 2),
        Edge::new(2, 4),
        Edge::new(4, 1),
        Edge::new(1, 6),
        Edge::new(2, 7),
        Edge::new(3, 8),
        Edge::new(4, 9),
        Edge::new(5, 10),
        Edge::new(6, 7),
        Edge::new(7, 8),
        Edge::new(8, 9),
        Edge::new(9, 10),
        Edge::new(10, 6),
    ];

    let mut encoder = CnfEncoder::default();
    let coloring = Coloring::encode(&mut encoder, &edges, num_vertices, num_colors);

    println!("=> Initializing the solver...");
    let mut solver = Cadical::default();
    println!("solver = {}", solver);

    println!("=> Adding {} clauses to solver...", encoder.clauses.len());
    for clause in encoder.clauses.iter() {
        solver.add_clause(clause);
    }

    println!("=> Solving...");
    let time_solve = Instant::now();
    let res = solver.solve();
    let time_solve = time_solve.elapsed();
    println!("=> Solver returned {:?} in {:.1} s", res, time_solve.as_secs_f64());

    if res == SolveResponse::Sat {
        println!("=> Coloring: SAT");

        let mut color = HashMap::new();

        for v in (1..=num_vertices).map(Vertex) {
            let c = *decode_onehot(&coloring.color[v], &solver).unwrap();
            println!("{:?} is colored in {:?}", v, c);
            color.insert(v, c);
        }

        println!("=> Checking coloring...");
        for Edge(a, b) in edges.iter() {
            assert_ne!(
                color[a], color[b],
                "Vertices {:?} and {:?} have the same color: {:?} == {:?}",
                a, b, color[a], color[b]
            );
        }
        println!("=> Coloring is valid");
    } else {
        println!("=> Coloring: UNSAT");
    }

    let time_total = time_total.elapsed();
    println!("==> All done in {:.1} s", time_total.as_secs_f64());

    Ok(())
}
