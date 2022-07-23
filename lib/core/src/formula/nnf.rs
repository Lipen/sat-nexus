use std::fmt::Debug;

use itertools::Itertools;
use log::debug;
use tap::Tap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NNF {
    Literal(i32),
    Conjunction(Vec<NNF>),
    Disjunction(Vec<NNF>),
}

impl NNF {
    pub fn and<I>(args: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<NNF>,
    {
        let args = args.into_iter().map_into::<NNF>().collect_vec();
        NNF::Conjunction(args)
    }

    pub fn or<I>(args: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<NNF>,
    {
        let args = args.into_iter().map_into::<NNF>().collect_vec();
        NNF::Disjunction(args)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CNF(pub(crate) Conjunction<Disjunction<i32>>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Conjunction<T>(pub(crate) Vec<T>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Disjunction<T>(pub(crate) Vec<T>);

impl NNF {
    pub fn to_cnf<F>(&self, new_var: &mut F) -> CNF
    where
        F: FnMut() -> i32,
    {
        debug!("NNF::to_cnf(self = {:?})", self);
        match self {
            NNF::Literal(lit) => CNF(Conjunction(vec![Disjunction(vec![*lit])])),
            NNF::Conjunction(args) => {
                debug_assert_ne!(args.len(), 0, "Empty args");
                let clauses = args.iter().map(|arg| arg.to_cnf(new_var)).flat_map(|cnf| cnf.0 .0).collect_vec();
                CNF(Conjunction(clauses))
            }
            NNF::Disjunction(args) => match args.len() {
                0 => panic!("Empty args"),
                1 => args[0].to_cnf(new_var),
                _ => {
                    let mut clauses = Vec::new();
                    let mut clause = Vec::new();
                    for arg in args.iter() {
                        let (v, cls) = arg.reify(new_var);
                        clauses.extend(cls.into_iter().map(|cl| Disjunction(cl)));
                        clause.push(v);
                    }
                    clauses.push(Disjunction(clause));
                    CNF(Conjunction(clauses))
                }
            },
        }
    }

    pub fn reify<F>(&self, new_var: &mut F) -> (i32, Vec<Vec<i32>>)
    where
        F: FnMut() -> i32,
    {
        debug!("NNF::reify(self = {:?})", self);
        match self {
            NNF::Literal(lit) => (*lit, vec![]),
            NNF::Conjunction(args) | NNF::Disjunction(args) => {
                match args.len() {
                    0 => panic!("Empty args"),
                    1 => args[0].reify(new_var),
                    _ => {
                        // Reify args
                        let mut clauses = Vec::new();
                        let mut lits = Vec::new();
                        for term in args.iter() {
                            let (u, cls) = term.reify(new_var);
                            clauses.extend(cls);
                            lits.push(u);
                        }

                        let z = new_var();
                        match self {
                            NNF::Conjunction(_) => {
                                // Tseytin-encode 'z <=> AND(lits)'
                                // Add clause (z, -x1, -x2, ..., -xn), where xi \in lits
                                clauses.push({
                                    let mut cl = Vec::with_capacity(lits.len() + 1);
                                    cl.push(z);
                                    for lit in lits.iter().copied() {
                                        cl.push(-lit);
                                    }
                                    cl
                                });
                                // Add clauses (-z, xi) for each xi \in lits
                                for lit in lits.iter().copied() {
                                    clauses.push(vec![-z, lit]);
                                }
                            }
                            NNF::Disjunction(_) => {
                                // Tseytin-encode 'z <=> OR(lits)'
                                // Add clause (-z, -x1, -x2, ..., -xn), where xi \in lits
                                clauses.push({
                                    let mut cl = Vec::with_capacity(lits.len() + 1);
                                    cl.push(-z);
                                    for lit in lits.iter().copied() {
                                        cl.push(-lit);
                                    }
                                    cl
                                });
                                // Add clauses (z, xi) for each xi \in lits
                                for lit in lits.iter().copied() {
                                    clauses.push(vec![z, lit]);
                                }
                            }
                            _ => unreachable!(),
                        }

                        debug!("reify: z = {z}, lits = {lits:?}, clauses = {clauses:?}");
                        (z, clauses)
                    }
                }
            }
        }
        .tap(|x| {
            debug!("NNF::reify({self:?}) -> {x:?}");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nnf_reify1() {
        let mut nvars = 0usize;
        let mut new_var = || {
            nvars += 1;
            nvars as i32
        };
        let x1 = NNF::Literal(new_var());
        let x2 = NNF::Literal(new_var());
        let x3 = NNF::Literal(new_var());
        let x4 = NNF::Literal(new_var());
        let expr = NNF::or([x1, NNF::and([x2, x3, x4])]);
        println!("expr = {:?}", expr);
        let cnf = expr.to_cnf(&mut new_var);
        println!("cnf = {:?}", cnf);
        println!("nvars = {}", nvars);
        println!("nclauses = {}", cnf.0 .0.len());
        for clause in cnf.0 .0.iter() {
            println!(". {:?}", clause);
        }
        assert_eq!(nvars, 5);
        assert_eq!(cnf.0 .0.len(), 5);
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn test_nnf_reify2() {
        let mut nvars = 0usize;
        let mut new_var = || {
            nvars += 1;
            nvars as i32
        };
        let x1 = NNF::Literal(new_var());
        let x2 = NNF::Literal(new_var());
        let x3 = NNF::Literal(new_var());
        let expr = NNF::or([
            NNF::and([x1.clone(), x2.clone(), x3.clone()]),
            NNF::and([x1.clone(), x2.clone(), x3.clone()]),
        ]);
        println!("expr = {:?}", expr);
        let cnf = expr.to_cnf(&mut new_var);
        println!("cnf = {:?}", cnf);
        println!("nvars = {}", nvars);
        println!("nclauses = {}", cnf.0 .0.len());
        for clause in cnf.0 .0.iter() {
            println!(". {:?}", clause);
        }
        assert_eq!(nvars, 5);
        assert_eq!(cnf.0 .0.len(), 9);
    }

    #[test]
    fn test_nnf_reify3() {
        let mut nvars = 0usize;
        let mut new_var = || {
            nvars += 1;
            nvars as i32
        };
        let x1 = NNF::Literal(new_var());
        let x2 = NNF::Literal(new_var());
        let x3 = NNF::Literal(new_var());
        let expr = NNF::or([NNF::or([NNF::or([NNF::and([x1, x2, x3])])])]);
        println!("expr = {:?}", expr);
        let cnf = expr.to_cnf(&mut new_var);
        println!("cnf = {:?}", cnf);
        println!("nvars = {}", nvars);
        println!("nclauses = {}", cnf.0 .0.len());
        for clause in cnf.0 .0.iter() {
            println!(". {:?}", clause);
        }
        assert_eq!(nvars, 3);
        assert_eq!(cnf.0 .0.len(), 3);
    }
}
