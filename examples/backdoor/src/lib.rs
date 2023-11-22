pub mod algorithm;
pub mod fitness;
pub mod instance;
pub mod utils;

#[cfg(feature = "minimization")]
pub mod minimization;

#[cfg(not(feature = "minimization"))]
pub mod minimization {
    use simple_sat::lit::Lit;

    pub fn minimize_backdoor(_cubes: &[Vec<Lit>]) -> Vec<Vec<Lit>> {
        panic!("Use 'minimization' feature!")
    }
}
