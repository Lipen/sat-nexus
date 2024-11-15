use std::fmt::Write as _;

#[derive(Debug, Clone)]
pub struct TruthTable {
    pub variables: usize,
    pub rows: Vec<(Vec<bool>, bool)>,
}

impl TruthTable {
    /// Create a new truth table with the given number of variables.
    pub fn new(variables: usize) -> Self {
        TruthTable {
            variables,
            rows: Vec::new(),
        }
    }

    /// Add a row to the truth table.
    pub fn add_row(&mut self, inputs: Vec<bool>, output: bool) {
        self.rows.push((inputs, output));
    }

    /// Create a complete truth table with `n` variables.
    pub fn complete(n: usize) -> Self {
        let mut table = TruthTable::new(n);
        for i in 0..(1 << n) {
            let inputs = (0..n).map(|j| (i & (1 << j)) != 0).collect();
            table.add_row(inputs, false); // Placeholder output
        }
        table
    }

    /// Display the truth table in a simple format.
    ///
    /// ```txt
    /// 0 0 | 0
    /// 0 1 | 1
    /// 1 0 | 1
    /// 1 1 | 0
    /// ```
    pub fn display_simple(&self) -> String {
        let mut output = String::new();

        for (inputs, result) in self.rows.iter() {
            for &input in inputs.iter() {
                write!(output, "{} ", if input { "1" } else { "0" }).unwrap();
            }
            writeln!(output, "| {}", if *result { "1" } else { "0" }).unwrap();
        }

        output
    }
}
