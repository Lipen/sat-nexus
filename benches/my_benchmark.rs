use criterion::{criterion_group, criterion_main, Criterion};

use cadical::Cadical;
use cadical_sys::statik;
use minisat::statik::MiniSat;

fn ms_solve(minisat: &MiniSat) {
    minisat.solve();
}

fn cadical_solve(cadical: &Cadical) {
    cadical.solve().unwrap();
}

fn ccadical_solve(ptr: *mut statik::CCaDiCaL) {
    unsafe {
        statik::ccadical_solve(ptr);
    }
}

fn my_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("Solvers");

    let minisat = MiniSat::new();
    group.bench_with_input("MiniSat solve", &minisat, |b, minisat| b.iter(|| ms_solve(minisat)));

    let cadical = Cadical::new();
    group.bench_with_input("Cadical solve", &cadical, |b, cadical| b.iter(|| cadical_solve(cadical)));

    let ptr = unsafe { statik::ccadical_init() };
    group.bench_with_input("CCaDiCaL solve", &ptr, |b, &ptr| b.iter(|| ccadical_solve(ptr)));

    group.finish();
}

criterion_group!(benches, my_benches);
criterion_main!(benches);
