use tracing::debug;

#[derive(Debug)]
pub struct LearningStrategy {
    pub learntsize_factor: f64,
    pub learntsize_inc: f64,
    pub learntsize_adjust_start: f64,
    pub learntsize_adjust_inc: f64,
}

#[derive(Debug)]
pub struct LearningGuard {
    pub strategy: LearningStrategy,
    max_learnts: f64,
    learntsize_adjust_confl: f64,
    learntsize_adjust_cnt: u64,
}

impl LearningGuard {
    pub fn new(strategy: LearningStrategy) -> Self {
        Self {
            strategy,
            max_learnts: 0.0,
            learntsize_adjust_confl: 0.0,
            learntsize_adjust_cnt: 0,
        }
    }

    pub fn limit(&self, num_assigns: usize) -> usize {
        self.max_learnts as usize + num_assigns
    }

    pub fn reset(&mut self, num_clauses: usize) {
        self.max_learnts = num_clauses as f64 * self.strategy.learntsize_factor;
        self.learntsize_adjust_confl = self.strategy.learntsize_adjust_start;
        self.learntsize_adjust_cnt = self.learntsize_adjust_confl as _;
    }

    pub fn bump(&mut self) -> bool {
        self.learntsize_adjust_cnt -= 1;
        if self.learntsize_adjust_cnt == 0 {
            self.max_learnts *= self.strategy.learntsize_inc;
            self.learntsize_adjust_confl *= self.strategy.learntsize_adjust_inc;
            self.learntsize_adjust_cnt = self.learntsize_adjust_confl as _;
            debug!(
                "New max_learnts = {}, learntsize_adjust_cnt = {}",
                self.max_learnts as u64, self.learntsize_adjust_cnt
            );
            true
        } else {
            false
        }
    }
}
