use std::collections::HashMap;
use std::iter::zip;

use itertools::Itertools;

use cadical::statik::Cadical;
use sat_nexus_core::map::Map;

use crate::circuit::{BooleanCircuit, LogicGate};
use crate::encoder::CnfEncoder;
use crate::table::TruthTable;
use crate::utils::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GateType {
    And2,
    Or2,
    Not,
    // Add more gates as needed (e.g., And3, Or3, Xor2)
}

pub struct CircuitSynthesis {
    pub num_gates: usize,
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub external_input_pins: Vec<Pin>,
    pub external_output_pins: Vec<Pin>,
    pub gate_input_pins: HashMap<Gate, Vec<Pin>>,
    pub gate_output_pins: HashMap<Gate, Vec<Pin>>,
    pub unique_cubes: Vec<Vec<bool>>,
    pub gate_type: Map<Gate, Map<GateType, i32>>,
    pub pin_parent: Map<Pin, Map<Pin, i32>>,
    pub pin_value: Map<Pin, Map<usize, i32>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Gate(pub usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Pin(pub usize);

impl Pin {
    pub const DISCONNECTED: Pin = Pin(0);
}

pub fn encode_circuit_synthesis(encoder: &mut CnfEncoder, num_gates: usize, truth_tables: &[TruthTable]) -> CircuitSynthesis {
    assert!(truth_tables.iter().all(|tt| tt.variables == truth_tables[0].variables));

    let num_inputs = truth_tables[0].variables;
    let num_outputs = truth_tables.len();

    let max_gate_inputs: usize = 2; // Maximum number of inputs for a gate
    let max_gate_outputs: usize = 1; // Maximum number of outputs for a gate

    let mut outputs_for_cube: HashMap<Vec<bool>, Vec<Option<bool>>> = HashMap::new();
    for (i, tt) in truth_tables.iter().enumerate() {
        for &(ref inputs, output) in tt.rows.iter() {
            outputs_for_cube.entry(inputs.clone()).or_insert_with(|| vec![None; num_outputs])[i] = Some(output);
        }
    }
    let unique_cubes: Vec<Vec<bool>> = outputs_for_cube.keys().cloned().sorted().collect();
    let num_cubes = unique_cubes.len();

    // Generate GATE_TYPE variables for each gate
    let mut gate_type_vars = Map::default();
    for gate in (1..=num_gates).map(Gate) {
        let possible_gate_types = vec![GateType::And2, GateType::Or2, GateType::Not];
        let var = encoder.new_direct(possible_gate_types);

        // Each gate must have exactly one type
        encoder.exactly_one(var.values());

        gate_type_vars.add(gate, var);
    }

    let mut num_pins = 0;

    let mut external_input_pins: Vec<Pin> = Vec::new(); // [outgoing_pin]
    for _ in 0..num_inputs {
        num_pins += 1;
        external_input_pins.push(Pin(num_pins));
    }
    println!("external_input_pins = {:?}", external_input_pins);

    let mut external_output_pins: Vec<Pin> = Vec::new(); // [incoming_pin]
    for _ in 0..num_outputs {
        num_pins += 1;
        external_output_pins.push(Pin(num_pins));
    }
    println!("external_output_pins = {:?}", external_output_pins);

    let mut gate_input_pins: HashMap<Gate, Vec<Pin>> = HashMap::new(); // gate: [incoming_pin]
    let mut gate_output_pins: HashMap<Gate, Vec<Pin>> = HashMap::new(); // gate: [outgoing_pin]
    for gate in (1..=num_gates).map(Gate) {
        let pins = (1..=max_gate_inputs)
            .map(|_| {
                num_pins += 1;
                Pin(num_pins)
            })
            .collect();
        println!("gate = {:?}, input_pins = {:?}", gate, pins);
        gate_input_pins.insert(gate, pins);

        let pins = (1..=max_gate_outputs)
            .map(|_| {
                num_pins += 1;
                Pin(num_pins)
            })
            .collect();
        println!("gate = {:?}, output_pins = {:?}", gate, pins);
        gate_output_pins.insert(gate, pins);
    }

    let num_pins = num_pins; // make immutable

    // Generate PIN_PARENT variables for each gate and input pin
    let mut pin_parent_vars = Map::default();
    for gate in (1..=num_gates).map(Gate) {
        for &pin in gate_input_pins[&gate].iter() {
            let mut possible_parent_pins = vec![Pin::DISCONNECTED]; // [outgoing_pin]
            possible_parent_pins.extend_from_slice(&external_input_pins);
            for other_gate in (1..gate.0).map(Gate) {
                // all gates with lower number
                possible_parent_pins.extend_from_slice(&gate_output_pins[&other_gate]);
            }
            println!(
                "gate = {:?}, pin = {:?}, possible_parent_pins = {:?}",
                gate, pin, possible_parent_pins
            );

            let var = encoder.new_direct(possible_parent_pins);

            // Each gate input pin must have exactly one parent
            encoder.exactly_one(var.values());

            pin_parent_vars.add(pin, var);
        }
    }
    for &pin in external_output_pins.iter() {
        let mut possible_parent_pins = vec![Pin::DISCONNECTED]; // [incoming_pin]
        possible_parent_pins.extend_from_slice(&external_input_pins);
        for gate in (1..=num_gates).map(Gate) {
            possible_parent_pins.extend_from_slice(&gate_output_pins[&gate]);
        }
        println!("pin = {:?}, possible_parent_pins = {:?}", pin, possible_parent_pins);

        let var = encoder.new_direct(possible_parent_pins);

        // Each gate input pin must have exactly one parent
        encoder.exactly_one(var.values());

        pin_parent_vars.add(pin, var);
    }

    // Pin parent absence propagation
    for gate in (1..=num_gates).map(Gate) {
        for (&pin1, &pin2) in gate_input_pins[&gate].iter().tuple_windows() {
            let p1 = pin_parent_vars[pin1][Pin::DISCONNECTED];
            let p2 = pin_parent_vars[pin2][Pin::DISCONNECTED];
            encoder.add_clause(vec![-p1, p2]);
        }
    }

    // Generate PIN_VALUE variables for each pin and input cube
    let mut pin_value_vars = Map::default();
    for pin in (1..=num_pins).map(Pin) {
        let var = encoder.new_direct((0..num_cubes).collect());
        pin_value_vars.add(pin, var);
    }

    // Encode input values
    for (i, cube) in unique_cubes.iter().enumerate() {
        for (j, &input) in cube.iter().enumerate() {
            let v = pin_value_vars[external_input_pins[j]][i];
            if input {
                encoder.add_clause(vec![v]);
            } else {
                encoder.add_clause(vec![-v]);
            }
        }
    }

    // Encode output values
    for (i, cube) in unique_cubes.iter().enumerate() {
        let outputs = &outputs_for_cube[cube];
        for (j, &output) in outputs.iter().enumerate() {
            if let Some(output) = output {
                let v = pin_value_vars[external_output_pins[j]][i];
                if output {
                    encoder.add_clause(vec![v]);
                } else {
                    encoder.add_clause(vec![-v]);
                }
            }
        }
    }

    // Encode pin connections
    for (&pin, pin_parent) in pin_parent_vars.iter() {
        for (&parent, &p) in pin_parent.iter() {
            for i in 0..num_cubes {
                let v = pin_value_vars[pin][i];
                if parent == Pin::DISCONNECTED {
                    // (parent[p] = 0) => (value[p] = False)
                    encoder.add_clause(vec![-p, -v]);
                } else {
                    let vp = pin_value_vars[parent][i];
                    // (parent[p1] = p2) => (value[p1] <=> value[p2])
                    encoder.add_clause(vec![-p, -v, vp]);
                    encoder.add_clause(vec![-p, v, -vp]);
                }
            }
        }
    }

    // Encode gate semantics
    for gate in (1..=num_gates).map(Gate) {
        for i in 0..num_cubes {
            // AND2 Gate
            {
                let t = gate_type_vars[gate][GateType::And2];
                let v_out = pin_value_vars[gate_output_pins[&gate][0]][i];
                let v_in1 = pin_value_vars[gate_input_pins[&gate][0]][i];
                let v_in2 = pin_value_vars[gate_input_pins[&gate][1]][i];
                // (type is AND) => (value_out <=> (value_in1 and value_in2))
                encoder.add_clause(vec![-t, v_out, -v_in1, -v_in2]);
                encoder.add_clause(vec![-t, -v_out, v_in1]);
                encoder.add_clause(vec![-t, -v_out, v_in2]);
            }

            // OR2 Gate
            {
                let t = gate_type_vars[gate][GateType::Or2];
                let v_out = pin_value_vars[gate_output_pins[&gate][0]][i];
                let v_in1 = pin_value_vars[gate_input_pins[&gate][0]][i];
                let v_in2 = pin_value_vars[gate_input_pins[&gate][1]][i];
                // (type is OR) => (value_out <=> (value_in1 or value_in2))
                encoder.add_clause(vec![-t, -v_out, v_in1, v_in2]);
                encoder.add_clause(vec![-t, v_out, -v_in1]);
                encoder.add_clause(vec![-t, v_out, -v_in2]);
            }

            // NOT Gate
            {
                let t = gate_type_vars[gate][GateType::Not];
                let v_out = pin_value_vars[gate_output_pins[&gate][0]][i];
                let v_in1 = pin_value_vars[gate_input_pins[&gate][0]][i];
                // (type is NOT) => (value_out <=> not value_in1)
                encoder.add_clause(vec![-t, -v_out, -v_in1]);
                encoder.add_clause(vec![-t, v_out, v_in1]);
            }
        }
    }

    // NOT gates have only one input
    if max_gate_inputs > 1 {
        for gate_type in [GateType::Not] {
            for gate in (1..=num_gates).map(Gate) {
                let t = gate_type_vars[gate][gate_type];
                let p = pin_parent_vars[gate_input_pins[&gate][1]][Pin::DISCONNECTED];
                // (type is NOT) => (parent[2] = 0)
                encoder.add_clause(vec![-t, p]);
            }
        }
    }

    // Binary gates have only two inputs
    if max_gate_inputs > 2 {
        for gate_type in [GateType::And2, GateType::Or2] {
            for gate in (1..=num_gates).map(Gate) {
                let t = gate_type_vars[gate][gate_type];
                let p = pin_parent_vars[gate_input_pins[&gate][2]][Pin::DISCONNECTED];
                // (type is binary) => (parent[3] = 0)
                encoder.add_clause(vec![-t, p]);
            }
        }
    }

    // hack: unary gate inputs are connected
    for gate in (1..=num_gates).map(Gate) {
        for gate_type in [GateType::Not] {
            let t = gate_type_vars[gate][gate_type];
            // (type is unary) => (parent[1] != 0)
            let p = pin_parent_vars[gate_input_pins[&gate][0]][Pin::DISCONNECTED];
            encoder.add_clause(vec![-t, -p]);
        }
    }

    // hack: binary gate inputs are connected
    for gate in (1..=num_gates).map(Gate) {
        for gate_type in [GateType::And2, GateType::Or2] {
            let t = gate_type_vars[gate][gate_type];
            // (type is binary) => (parent[2] != 0)
            let p = pin_parent_vars[gate_input_pins[&gate][1]][Pin::DISCONNECTED];
            encoder.add_clause(vec![-t, -p]);
        }
    }

    CircuitSynthesis {
        num_gates,
        num_inputs,
        num_outputs,
        external_input_pins,
        external_output_pins,
        gate_input_pins,
        gate_output_pins,
        unique_cubes,
        gate_type: gate_type_vars,
        pin_parent: pin_parent_vars,
        pin_value: pin_value_vars,
    }
}

impl CircuitSynthesis {
    pub fn build_circuit(&self, solver: &Cadical) -> BooleanCircuit {
        let mut circuit = BooleanCircuit::new(self.num_inputs, self.num_outputs);

        let mut pin_mapping = HashMap::new();

        for (&pin, &i) in zip(&self.external_input_pins, &circuit.input_pins) {
            if let Some(j) = pin_mapping.insert(pin, i) {
                panic!("pin {:?} is already mapped to {}", pin, j);
            }
        }
        for (&pin, &i) in zip(&self.external_output_pins, &circuit.output_pins) {
            if let Some(j) = pin_mapping.insert(pin, i) {
                panic!("pin {:?} is already mapped to {}", pin, j);
            }
        }

        let mut gate_mapping = HashMap::new();

        for (&gate, gate_type_var) in self.gate_type.iter() {
            let gate_type = decode_onehot(gate_type_var, solver).unwrap();
            let g = match gate_type {
                GateType::And2 => LogicGate {
                    kind: "AND".to_string(),
                    num_inputs: 2,
                    num_outputs: 1,
                },
                GateType::Or2 => LogicGate {
                    kind: "OR".to_string(),
                    num_inputs: 2,
                    num_outputs: 1,
                },
                GateType::Not => LogicGate {
                    kind: "NOT".to_string(),
                    num_inputs: 1,
                    num_outputs: 1,
                },
            };
            let index = circuit.add_gate(g);
            for (&pin, &i) in zip(&self.gate_input_pins[&gate], &circuit.gate_input_pins[&index]) {
                if let Some(j) = pin_mapping.insert(pin, i) {
                    panic!("pin {:?} is already mapped to {}", pin, j);
                }
            }
            for (&pin, &i) in zip(&self.gate_output_pins[&gate], &circuit.gate_output_pins[&index]) {
                if let Some(j) = pin_mapping.insert(pin, i) {
                    panic!("pin {:?} is already mapped to {}", pin, j);
                }
            }
            gate_mapping.insert(gate, index);
        }

        for (&pin, pin_parent_var) in self.pin_parent.iter() {
            let pin_parent = *decode_onehot(pin_parent_var, solver).unwrap();
            if pin_parent == Pin::DISCONNECTED {
                continue;
            }
            let i = pin_mapping[&pin];
            let j = pin_mapping[&pin_parent];
            circuit.connect(i, j);
        }

        circuit
    }
}
