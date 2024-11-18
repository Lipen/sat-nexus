use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Write as _;

use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct LogicGate {
    pub kind: String,
    pub num_inputs: usize,
    pub num_outputs: usize,
}

// Note: Gates are 0-based
// Note: Pins are 1-based

#[derive(Debug, Clone)]
pub struct BooleanCircuit {
    pub gates: Vec<LogicGate>,
    pub num_pins: usize,
    pub input_pins: Vec<usize>,                       // [pin]
    pub output_pins: Vec<usize>,                      // [pin]
    pub gate_input_pins: HashMap<usize, Vec<usize>>,  // gate: [pin]
    pub gate_output_pins: HashMap<usize, Vec<usize>>, // gate: [pin]
    pub connections: HashMap<usize, usize>,           // pin: parent
}

impl BooleanCircuit {
    pub fn new(num_inputs: usize, num_outputs: usize) -> Self {
        let num_pins = num_inputs + num_outputs;
        let input_pins = (1..=num_inputs).collect();
        let output_pins = (num_inputs + 1..=num_pins).collect();
        Self {
            gates: Vec::new(),
            num_pins,
            input_pins,
            output_pins,
            gate_input_pins: HashMap::new(),
            gate_output_pins: HashMap::new(),
            connections: HashMap::new(),
        }
    }
}

impl Display for BooleanCircuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BooleanCircuit {{")?;
        writeln!(
            f,
            "  gates: {{{}}}",
            self.gates.iter().enumerate().map(|(i, g)| format!("{}: {}", i, g.kind)).join(", ")
        )?;
        writeln!(f, "  num_pins: {}", self.num_pins)?;
        writeln!(f, "  input_pins: {:?}", self.input_pins)?;
        writeln!(f, "  output_pins: {:?}", self.output_pins)?;
        writeln!(
            f,
            "  gate_input_pins: {{{}}}",
            (0..self.gates.len())
                .map(|i| format!("{}: {:?}", i, self.gate_input_pins[&i]))
                .join(", ")
        )?;
        writeln!(
            f,
            "  gate_output_pins: {{{}}}",
            (0..self.gates.len())
                .map(|i| format!("{}: {:?}", i, self.gate_output_pins[&i]))
                .join(", ")
        )?;
        writeln!(
            f,
            "  connections: {{{}}}",
            self.output_pins
                .iter()
                .copied()
                .chain((0..self.gates.len()).flat_map(|i| self.gate_input_pins[&i].iter().copied()))
                .map(|p| format!("{}<-{}", p, self.connections.get(&p).copied().unwrap_or(0)))
                .join(", ")
        )?;
        write!(f, "}}")
    }
}

impl BooleanCircuit {
    pub fn add_input(&mut self) -> usize {
        self.num_pins += 1;
        self.input_pins.push(self.num_pins);
        self.num_pins
    }

    pub fn add_output(&mut self) -> usize {
        self.num_pins += 1;
        self.output_pins.push(self.num_pins);
        self.num_pins
    }

    pub fn add_gate(&mut self, gate: LogicGate) -> usize {
        let index = self.gates.len();
        let input_pins = (0..gate.num_inputs)
            .map(|_| {
                self.num_pins += 1;
                self.num_pins
            })
            .collect();
        if let Some(pins) = self.gate_input_pins.insert(index, input_pins) {
            panic!("Gate {} already has associated input pins: {:?}", index, pins);
        }
        let output_pins = (0..gate.num_outputs)
            .map(|_| {
                self.num_pins += 1;
                self.num_pins
            })
            .collect();
        if let Some(pins) = self.gate_output_pins.insert(index, output_pins) {
            panic!("Gate {} already has associated output pins: {:?}", index, pins);
        }
        self.gates.push(gate);
        index
    }

    pub fn connect(&mut self, pin: usize, parent: usize) {
        println!("Connecting pin {} to parent {}", pin, parent);
        if let Some(other) = self.connections.insert(pin, parent) {
            panic!("Pin {} is already connected to {}", pin, other)
        }
    }
}

impl BooleanCircuit {
    /// ### Example:
    ///
    /// ```dot
    /// digraph {
    ///   rankdir=LR;
    ///   { rank=source
    ///   node [shape=oval];
    ///   i1 [label="in1:pin1"];
    ///   i2 [label="in2:pin2"];
    ///   i3 [label="in3:pin3"];
    ///   }
    ///   { rank=sink
    ///   node [shape=oval];
    ///   o1 [label="out1:pin4"];
    ///   }
    ///   g1 [shape=Mrecord, label="{{ <x1> x1:pin5 | <x2> x2:pin6 } | { gate1 \n AND } | { <y1> y1:pin7 }}"];
    ///   g2 [shape=Mrecord, label="{{ <x1> x1:pin8 | <x2> x2:pin9 } | { gate2 \n OR } | { <y1> y1:pin10 }}"];
    ///   g1:y1 -> g2:x1;
    ///   i3 -> g2:x2;
    ///   g2:y1 -> o1;
    ///   i2 -> g1:x1;
    ///   i1 -> g1:x2;
    /// }
    /// ```
    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph {\n");
        dot.push_str("  rankdir=LR;\n");

        let mut pin2id = HashMap::new();

        dot.push_str("  { rank=source\n");
        dot.push_str("  node [shape=oval];\n");
        for (i, &pin) in self.input_pins.iter().enumerate() {
            let id = format!("i{}", i + 1);
            writeln!(dot, "  {} [label=\"in{}:pin{}\"];", id, i + 1, pin).unwrap();
            if let Some(id) = pin2id.insert(pin, id) {
                panic!("Pin {} is already connected to {}", pin, id)
            }
        }
        dot.push_str("  }\n");

        dot.push_str("  { rank=sink\n");
        dot.push_str("  node [shape=oval];\n");
        for (i, &pin) in self.output_pins.iter().enumerate() {
            let id = format!("o{}", i + 1);
            writeln!(dot, "  {} [label=\"out{}:pin{}\"];", id, i + 1, pin).unwrap();
            if let Some(id) = pin2id.insert(pin, id) {
                panic!("Pin {} is already connected to {}", pin, id)
            }
        }
        dot.push_str("  }\n");

        for (g, gate) in self.gates.iter().enumerate() {
            let id = format!("g{}", g + 1);
            let xs = (0..gate.num_inputs)
                .map(|i| format!("<x{}> x{0}:pin{}", i + 1, self.gate_input_pins[&g][i]))
                .join(" | ");
            let ys = (0..gate.num_outputs)
                .map(|i| format!("<y{}> y{0}:pin{}", i + 1, self.gate_output_pins[&g][i]))
                .join(" | ");
            writeln!(
                dot,
                "  {} [shape=Mrecord, label=\"{{{{ {} }} | {{ gate{} \\n {} }} | {{ {} }}}}\"];",
                id,
                xs,
                g + 1,
                gate.kind,
                ys
            )
            .unwrap();
            for (i, &pin) in self.gate_input_pins[&g].iter().enumerate() {
                let id = format!("g{}:x{}", g + 1, i + 1);
                if let Some(id) = pin2id.insert(pin, id) {
                    panic!("Pin {} is already connected to {}", pin, id)
                }
            }
            for (i, &pin) in self.gate_output_pins[&g].iter().enumerate() {
                let id = format!("g{}:y{}", g + 1, i + 1);
                if let Some(id) = pin2id.insert(pin, id) {
                    panic!("Pin {} is already connected to {}", pin, id)
                }
            }
        }

        for (&pin, &parent) in self.connections.iter() {
            writeln!(dot, "  {} -> {};", pin2id[&parent], pin2id[&pin]).unwrap();
        }

        dot.push_str("}");
        dot
    }
}
