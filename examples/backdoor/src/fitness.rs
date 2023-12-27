use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Fitness {
    pub value: f64,
    pub rho: f64,
    pub num_hard: u64,
}

impl Eq for Fitness {}

impl PartialEq<Self> for Fitness {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Ord for Fitness {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.total_cmp(&other.value)
    }
}

impl PartialOrd<Self> for Fitness {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
