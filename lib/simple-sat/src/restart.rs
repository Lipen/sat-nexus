use crate::utils::luby;

#[derive(Debug)]
pub struct RestartStrategy {
    pub is_luby: bool,
    pub restart_init: usize,
    pub restart_inc: f64,
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
