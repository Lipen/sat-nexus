use std::marker::PhantomData;

use crate::formula::expr::Expr;

struct Simplifier<T> {
    changed: bool,
    _phantom: PhantomData<T>,
}

impl<T> Simplifier<T> {
    pub fn new() -> Self {
        Self {
            changed: false,
            _phantom: PhantomData,
        }
    }

    pub fn step(&mut self, expr: Expr<T>) -> Expr<T> {
        match expr {
            Expr::Not { arg: outer } => match *outer {
                Expr::Const(b) => Expr::Const(!b),
                Expr::Not { arg: inner } => {
                    // Double negation: ~~A |- A
                    self.changed = true;
                    *inner
                }
                Expr::And { args } => self.de_morgan_and(args),
                Expr::Or { args } => self.de_morgan_or(args),
                _ => Expr::Not { arg: outer },
            },
            Expr::And { args } => self.consolidate_and(args),
            Expr::Or { args } => self.consolidate_or(args),
            e => e,
        }
    }

    // Consolidate: AND(x1,AND(x2, x3)) |- AND(x1,x2,x3)
    fn consolidate_and(&mut self, args: Vec<Expr<T>>) -> Expr<T> {
        let mut new_args = Vec::new();
        for arg in args {
            match arg {
                Expr::And { args: sub_args } => {
                    self.changed = true;
                    new_args.extend(sub_args.into_iter().map(|x| self.step(x)));
                }
                _ => new_args.push(self.step(arg)),
            }
        }
        Expr::and(new_args)
    }

    // Consolidate: OR(x1,OR(x2,x3)) |- OR(x1,x2,x3)
    fn consolidate_or(&mut self, args: Vec<Expr<T>>) -> Expr<T> {
        let mut new_args = Vec::new();
        for arg in args {
            match arg {
                Expr::Or { args: sub_args } => {
                    self.changed = true;
                    new_args.extend(sub_args.into_iter().map(|x| self.step(x)));
                }
                _ => new_args.push(self.step(arg)),
            }
        }
        Expr::or(new_args)
    }

    // De Morgan: ~(A and B) |- (~A or ~B)
    fn de_morgan_and(&mut self, args: Vec<Expr<T>>) -> Expr<T> {
        self.changed = true;
        Expr::or(args.into_iter().map(|x| Expr::not(self.step(x))))
    }

    // De Morgan: ~(A or B) |- (~A and ~B)
    fn de_morgan_or(&mut self, args: Vec<Expr<T>>) -> Expr<T> {
        self.changed = true;
        Expr::and(args.into_iter().map(|x| Expr::not(self.step(x))))
    }
}

pub fn simplify<T>(mut expr: Expr<T>) -> Expr<T> {
    let mut simplifier = Simplifier::new();
    simplifier.changed = true;
    while simplifier.changed {
        simplifier.changed = false;
        expr = simplifier.step(expr);
    }
    expr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_and() {
        let e: Expr<i32> = Expr::And {
            args: vec![
                Expr::Terminal(1),
                Expr::Or {
                    args: vec![Expr::Terminal(2), Expr::Terminal(3)],
                },
                Expr::And {
                    args: vec![Expr::Terminal(4), Expr::Terminal(5)],
                },
                Expr::Terminal(6),
                Expr::And {
                    args: vec![Expr::Terminal(7)],
                },
            ],
        };
        println!("e = {:?}", e);
        let e = simplify(e);
        println!("After simplify: e = {:?}", e);
        assert_eq!(
            e,
            Expr::And {
                args: vec![
                    Expr::Terminal(1),
                    Expr::Or {
                        args: vec![Expr::Terminal(2), Expr::Terminal(3)]
                    },
                    Expr::Terminal(4),
                    Expr::Terminal(5),
                    Expr::Terminal(6),
                    Expr::Terminal(7),
                ]
            }
        );
    }

    #[test]
    fn test_simplify_or() {
        let e: Expr<i32> = Expr::Or {
            args: vec![
                Expr::Terminal(1),
                Expr::And {
                    args: vec![Expr::Terminal(2), Expr::Terminal(3)],
                },
                Expr::Or {
                    args: vec![Expr::Terminal(4), Expr::Terminal(5)],
                },
                Expr::Terminal(6),
                Expr::Or {
                    args: vec![Expr::Terminal(7)],
                },
            ],
        };
        println!("e = {:?}", e);
        let e = simplify(e);
        println!("After simplify: e = {:?}", e);
        assert_eq!(
            e,
            Expr::Or {
                args: vec![
                    Expr::Terminal(1),
                    Expr::And {
                        args: vec![Expr::Terminal(2), Expr::Terminal(3)]
                    },
                    Expr::Terminal(4),
                    Expr::Terminal(5),
                    Expr::Terminal(6),
                    Expr::Terminal(7),
                ]
            }
        );
    }

    #[test]
    fn test_nested_simplify() {
        let e: Expr<i32> = Expr::Or {
            args: vec![
                Expr::Terminal(1),
                Expr::Or {
                    args: vec![
                        Expr::Terminal(2),
                        Expr::Or {
                            args: vec![
                                Expr::Terminal(3),
                                Expr::Or {
                                    args: vec![
                                        Expr::Terminal(4),
                                        Expr::Not {
                                            arg: Box::new(Expr::And {
                                                args: vec![
                                                    Expr::Not {
                                                        arg: Box::new(Expr::Terminal(5)),
                                                    },
                                                    Expr::Not {
                                                        arg: Box::new(Expr::Terminal(6)),
                                                    },
                                                ],
                                            }),
                                        },
                                    ],
                                },
                            ],
                        },
                    ],
                },
            ],
        };
        println!("e = {:?}", e);
        let e = simplify(e);
        println!("After simplify: e = {:?}", e);
        assert_eq!(
            e,
            Expr::Or {
                args: vec![
                    Expr::Terminal(1),
                    Expr::Terminal(2),
                    Expr::Terminal(3),
                    Expr::Terminal(4),
                    Expr::Terminal(5),
                    Expr::Terminal(6),
                ]
            }
        );
    }
}
