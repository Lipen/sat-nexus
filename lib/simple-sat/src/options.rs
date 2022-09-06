#[derive(Debug)]
pub struct Options {
    // Restart:
    pub is_luby: bool,
    pub restart_init: usize,
    pub restart_inc: f64,
    // ReduceDB:
    pub learntsize_factor: f64,
    pub learntsize_inc: f64,
    pub learntsize_adjust_start: f64,
    pub learntsize_adjust_inc: f64,
}

pub const DEFAULT_OPTIONS: Options = Options {
    // Restart:
    is_luby: true,
    restart_init: 100,
    restart_inc: 2.0,
    // ReduceDB:
    learntsize_factor: 0.3,
    learntsize_inc: 1.1,
    learntsize_adjust_start: 100.0,
    learntsize_adjust_inc: 1.5,
};

impl Default for Options {
    fn default() -> Self {
        DEFAULT_OPTIONS
    }
}
