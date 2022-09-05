use crate::utils::luby;

pub struct RestartStrategy {
    is_luby: bool,
    restart_init: usize,
    restart_inc: f64,
}

impl RestartStrategy {
    pub fn new(is_luby: bool) -> Self {
        Self {
            is_luby,
            restart_init: 100, // MiniSat: 100
            restart_inc: 2.0,  // MiniSat: 2.0
        }
    }
}

impl RestartStrategy {
    pub fn num_confl(&self, restarts: usize) -> usize {
        let restart_base = if self.is_luby {
            luby(self.restart_inc, restarts as u32)
        } else {
            self.restart_inc.powi(restarts as i32)
        };

        (restart_base * self.restart_init as f64) as usize
    }
}
