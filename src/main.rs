#![allow(unused)]

use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

use derive_more::Deref;
use ndarray::{Array, Array1, ArrayD};
use type_map::TypeMap;

use nexus_sat::context::Context;

mod ipasir;

// trait Var {
//     // fn get(&self, index: I) -> Lit {
//     //     *self.index(index)
//     // }
//
//     // fn get<Idx>(&self, index: Idx) -> Lit
//     // where
//     //     Self: Index<Idx, Output=Lit>,
//     // {
//     //     *self.index(index)
//     // }
// }
//
// // impl<V, I> Index<I> for V
// // where
// //     V: Var<I>
// // {
// //     type Output = V::Output;
// //
// //     fn index(&self, index: I) -> &Self::Output {
// //         todo!()
// //     }
// // }
//
// #[derive(Debug, Deref)]
// struct ColorVar {
//     data: Array1<Lit>,
// }
//
// impl ColorVar {
//     fn new<A: Into<Self>>(args: A) -> Self {
//         args.into()
//     }
// }
//
// impl From<Vec<Lit>> for ColorVar {
//     fn from(lits: Vec<Lit>) -> Self {
//         let data = Array1::from(lits);
//         ColorVar { data }
//     }
// }
//
// impl From<Vec<isize>> for ColorVar {
//     fn from(lits: Vec<isize>) -> Self {
//         let data = lits.into_iter().map(|i| Lit::new(i)).collect::<Vec<_>>();
//         ColorVar::from(data)
//     }
// }
//
// impl Var for ColorVar {}
//
// // impl_key!(ColorVar);
//
// // impl Key for ColorVar {
// //     type Value = Self;
// // }
//
// // impl Index<usize> for ColorVar {
// //     type Output = Lit;
// //
// //     fn index(&self, index: usize) -> &Self::Output {
// //         &self.data[index]
// //     }
// // }
//
// // impl Deref for ColorVar {
// //     type Target = Array1<Lit>;
// //
// //     fn deref(&self) -> &Self::Target {
// //         &self.data
// //     }
// // }
//
// // #[derive(Debug)]
// // struct MappingVar {
// //     data: Array2<Lit>,
// // }
// //
// // impl MappingVar {
// //     fn get(&self, v: usize, s: usize) -> Lit {
// //         self.data[[v, s]]
// //     }
// //
// //     fn intvar(&self, v: usize) -> MappingIntVar {
// //         let data = self.data.slice(ndarray::s![v, ..]);
// //         MappingIntVar { data }
// //     }
// // }
// //
// // #[derive(Debug)]
// // struct MappingIntVar<'a> {
// //     data: ArrayView1<'a, Lit>,
// // }
// //
// // impl<'a> MappingIntVar<'a> {
// //     fn eq(&self, s: usize) -> Lit {
// //         self.data[s]
// //     }
// // }
//
// #[derive(Debug, Clone, Deref)]
// struct TransitionVar {
//     data: ArrayD<Lit>,
// }
//
// impl TransitionVar {
//     fn from_shape_fn<F>(shape: (usize, usize), mut f: F) -> Self
//     where
//         F: FnMut(usize, usize) -> Lit,
//     {
//         let data = Array::from_shape_fn(shape, |(i, j)| f(i, j)).into_dyn();
//         TransitionVar { data }
//     }
// }
//
// impl Var for TransitionVar {}
//
// // impl_key!(TransitionVar);
//
// // impl Key for TransitionVar {
// //     type Value = Self;
// // }
//
// // impl Index<[usize; 2]> for TransitionVar {
// //     type Output = Lit;
// //
// //     fn index(&self, index: [usize; 2]) -> &Self::Output {
// //         self.data.index(index)
// //     }
// // }
//
// // impl Deref for TransitionVar {
// //     type Target = ArrayD<Lit>;
// //
// //     fn deref(&self) -> &Self::Target {
// //         &self.data
// //     }
// // }
//
// // impl FnOnce<(usize,)> for ColorVar {
// //     type Output = Lit;
// //
// //     extern "rust-call" fn call_once(self, args: (usize,)) -> Self::Output {
// //         eprintln!("ColorVar::call_once(args = {:?})", args);
// //         self.call(args)
// //     }
// // }
// //
// // impl FnMut<(usize,)> for ColorVar {
// //     extern "rust-call" fn call_mut(&mut self, args: (usize,)) -> Self::Output {
// //         eprintln!("ColorVar::call_mut(args = {:?})", args);
// //         self.call(args)
// //     }
// // }
// //
// // impl Fn<(usize,)> for ColorVar {
// //     extern "rust-call" fn call(&self, args: (usize,)) -> Self::Output {
// //         eprintln!("ColorVar::call(args = {:?})", args);
// //         let v = args.0;
// //         self.data[v]
// //     }
// // }
//
// #[derive(Debug, Deref)]
// struct SatVar {
//     data: ArrayD<Lit>,
// }
//
// #[derive(Debug, Deref)]
// struct MyFirstVar(SatVar);
//
// #[derive(Debug, Deref)]
// struct MySecondVar(SatVar);
//
// // struct StorageKey<T>(PhantomData<T>);
// //
// // impl<T:Any> Key for StorageKey<T>  {
// //     type Value = T;
// // }
// //
// // struct NamedStorageKey<T>(PhantomData<T>);
// //
// // impl<T: Any> Key for NamedStorageKey<T> {
// //     type Value = HashMap<String, T>;
// // }
//
// trait GenericSolver {
//     fn context(&self) -> &Context;
//     fn num_vars(&self) -> usize;
//     fn num_clauses(&self) -> usize;
//     fn new_lit(&mut self) -> Lit;
//     fn add_clause<I, L>(&mut self, lits: I)
//     where I: IntoIterator<Item = L>,
//           L: Into<Lit>;
//     // fn add_unit_clause(&mut self, lit: Lit) {
//     //     self.add_clause(&[lit]);
//     // }
//     // fn add_binary_clause(&mut self, lit1: Lit, lit2: Lit);
//     // fn add_ternary_clause(&mut self, lit1: Lit, lit2: Lit, lit3: Lit);
// }
//
// type Clause = Vec<Lit>;
//
// #[derive(Debug)]
// struct TheSolver {
//     context: Context,
//     nvars: usize,
//     nclauses: usize,
//     clauses: Vec<Clause>,
// }
//
// impl TheSolver {
//     fn new() -> Self {
//         TheSolver {
//             context: Context::new(),
//             nvars: 0,
//             nclauses: 0,
//             clauses: Vec::new(),
//         }
//     }
// }
//
// impl GenericSolver for TheSolver {
//     fn context(&self) -> &Context {
//         &self.context
//     }
//
//     fn num_vars(&self) -> usize {
//         self.nvars
//     }
//
//     fn num_clauses(&self) -> usize {
//         self.nclauses
//     }
//
//     fn new_lit(&mut self) -> Lit {
//         self.nvars += 1;
//         Lit::new(self.nvars)
//     }
//
//     fn add_clause<I, L>(&mut self, lits: I)
//     where I: IntoIterator<Item = L>,
//           L: Into<Lit>
//     {
//         self.nclauses += 1;
//         // self.clauses.push(lits.into().to_vec());
//     }
// }
//
// fn main() {
//     let color = ColorVar::new(vec![1, 2, 3]);
//     println!("color = {:?}", color);
//     println!("color.get(0) = {:?}", color.get(0));
//     println!("color[1] = {:?}", color[1]);
//
//     let map = TypeMap::new();
//     let mut context = Context::from(map);
//     context.insert(color);
//     // println!("context = {:?}", context);
//     // let extracted = map.get::<ColorVar>();
//     // let extracted = context.storage.get::<ColorVar>();
//     // println!("extracted = {:?}", extracted);
//     let extracted = context.get::<ColorVar>();
//     println!("extracted = {:?}", extracted);
//     let extracted: &ColorVar = context.extract();
//     println!("extracted = {:?}", extracted);
//
//     let mut i: usize = 0;
//     let transition = TransitionVar::from_shape_fn((3, 2), |_, _| {
//         i += 1;
//         Lit::new(i)
//     });
//     println!("transition:\n{:?}", transition);
//     println!("transition.get([0, 1]) = {:?}", transition.get([0, 1]));
//     // println!("transition.gett([0, 1]) = {:?}", transition.gett([0,1]));
//     println!("transition[[0,1]] = {:?}", transition[[0, 1]]);
//     println!("transition.data[[0,1]] = {:?}", transition.data[[0, 1]]);
//     context.insert(transition.clone());
//     let extracted = context.extract::<TransitionVar>();
//     println!("extracted:\n{:?}", extracted);
//     let extracted: &TransitionVar = context.extract();
//     println!("extracted:\n{:?}", extracted);
//     context.insert_named("transition".to_owned(), transition.clone());
//     let extracted = context.get_named::<TransitionVar, _>("transition");
//     println!("extracted (named):\n{:?}", extracted);
//
//     let vv = MyFirstVar(SatVar {
//         data: transition.data,
//     });
//     println!("vv = {:?}", vv);
//     context.insert(vv);
//     let extracted = context.get::<MyFirstVar>();
//     println!("extracted:\n{:?}", extracted.unwrap());
//     println!("extracted:\n{:?}", extracted.unwrap().0);
//
//     let mut solver = TheSolver::new();
//     println!("solver = {:?}", solver);
//     println!("new_lit = {:?}", solver.new_lit());
//     println!("new_lit = {:?}", solver.new_lit());
//     // solver.add_clause(&[Lit::new(1), Lit::new(2)]);
//     solver.add_clause(vec![1, 2]);
//     println!("new_lit = {:?}", solver.new_lit());
//     println!("solver = {:?}", solver);
//
//     // let mapping = MappingVar {
//     //     data: arr2(&[
//     //         [Lit::new(1), Lit::new(2), Lit::new(3)],
//     //         [Lit::new(4), Lit::new(5), Lit::new(6)],
//     //     ]),
//     // };
//     // println!("mapping = {:?}", mapping);
//     // println!("mapping[0,1] = {:?}", mapping.get(0, 1));
//     // println!("mapping[1,2] = {:?}", mapping.data[[1, 2]]);
//     // let intvar = mapping.intvar(1);
//     // println!("intvar = {:?}", intvar);
//     // println!("intvar.eq(1) = {:?}", intvar.eq(1));
//     //
//     // let dynarray = Array::from_shape_fn((5, 2, 3), |(i, j, k)| i * 10 + j * 2 + 3 * k).into_dyn();
//     // println!("dynarray:\n{}", dynarray);
//     // println!("dynarray[1,0,2] = {}", dynarray[[1, 0, 2]]);
//     // println!("dynarray[2,1,1] = {}", dynarray[[2, 1, 1]]);
// }

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    use nexus_sat::solver::wrap::WrappedIpasirSolver;
    use nexus_sat::solver::GenericSolver;

    let mut solver = WrappedIpasirSolver::new_cadical();
    println!("Solver signature: {}", solver.signature());

    solver.add_clause(&[1, 2]);
    solver.add_clause(&[3, 4]);
    solver.add_clause(&[-1, -2]);
    solver.add_clause(&[-3, -4]);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    solver.assume(1);
    solver.assume(2);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    // solver.assume(0);
    let response = solver.solve();
    println!("Solver returned: {:?}", response);

    Ok(())
}
