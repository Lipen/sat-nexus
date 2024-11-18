use cadical::statik::Cadical;

use crate::encoder::SatEncoder;
use crate::formula::BooleanFormula;
use crate::map::Map;
use crate::table::TruthTable;
use crate::utils::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NodeType {
    Terminal,
    And,
    Or,
    Not,
}

pub struct BooleanFormulaSynthesis {
    pub node_type: Map<usize, Map<NodeType, i32>>,
    pub index: Map<usize, Map<usize, i32>>,
    pub parent: Map<usize, Map<usize, i32>>,
    pub child: Map<usize, Map<usize, i32>>,
    pub value: Map<(usize, usize), i32>,
}

pub fn encode_boolean_synthesis(encoder: &mut SatEncoder, num_nodes: usize, truth_table: &TruthTable) -> BooleanFormulaSynthesis {
    // Note: most "domain" variables are 1-based, and some domains include 0 as sentinel value.
    // Note: 'row' is a 0-based index of a row in `truth_table.rows`.

    // Generate TYPE variables for each node and node type
    let mut node_type_vars = Map::default();
    for node in 1..=num_nodes {
        let possible_types = vec![NodeType::Terminal, NodeType::And, NodeType::Or, NodeType::Not];
        let var = encoder.new_direct(possible_types);

        // Each node must have exactly one type
        encoder.exactly_one(&var.values);

        node_type_vars.add(node, var);
    }

    // Last node must be a terminal
    {
        let t = node_type_vars[num_nodes][NodeType::Terminal];
        encoder.add_clause(vec![t]);
    }
    // The node before last cannot be a binary operation
    if num_nodes > 1 {
        let t = node_type_vars[num_nodes - 1][NodeType::And];
        encoder.add_clause(vec![-t]);
    }
    if num_nodes > 1 {
        let t = node_type_vars[num_nodes - 1][NodeType::Or];
        encoder.add_clause(vec![-t]);
    }

    // Generate INDEX variables for each node
    let mut index_vars = Map::default();
    for node in 1..=num_nodes {
        let possible_indices = (0..=truth_table.variables).collect();
        let index_var = encoder.new_direct(possible_indices);

        // Each node must have exactly one variable index associated
        encoder.exactly_one(&index_var.values);

        index_vars.add(node, index_var);
    }

    // Only terminal nodes can have variable index 0
    for node in 1..=num_nodes {
        let t = node_type_vars[node][NodeType::Terminal];
        let v = index_vars[node][0];
        // (node is terminal) <-> (variable != 0)
        encoder.add_clause(vec![-t, -v]);
        encoder.add_clause(vec![t, v]);
    }

    // Generate PARENT variables for each node
    let mut parent_vars = Map::default();
    for node in 2..=num_nodes {
        let possible_parents = (1..node).collect();
        let parent_var = encoder.new_direct(possible_parents);

        // Each node (except root) must have exactly one parent
        encoder.exactly_one(&parent_var.values);

        parent_vars.add(node, parent_var);
    }

    // BFS constraint
    // parent[j+1] >= parent[j]
    for node in 3..=(num_nodes - 1) {
        for parent1 in 2..node {
            for parent2 in 1..parent1 {
                let p1 = parent_vars[node][parent1];
                let p2 = parent_vars[node + 1][parent2];
                encoder.add_clause(vec![-p1, -p2])
            }
        }
    }

    // Generate CHILD variables for each node
    let mut child_vars = Map::default();
    for node in 1..=num_nodes {
        let mut possible_children = vec![0];
        possible_children.extend((node + 1)..=num_nodes);

        let child_var = encoder.new_direct(possible_children);

        // Each node must have exactly one child
        encoder.exactly_one(&child_var.values);

        child_vars.add(node, child_var);
    }

    // Encode parent-child relationships
    // child -> parent
    for node in 1..=num_nodes {
        for child in (node + 1)..=num_nodes {
            let c = child_vars[node][child];
            let p = parent_vars[child][node];
            // (child is parent's child) -> (parent is child's parent)
            encoder.add_clause(vec![-c, p]);
        }
    }
    // no child -> not a parent
    for node in 1..=num_nodes {
        let c = child_vars[node][0];
        for child in (node + 1)..=num_nodes {
            let p = parent_vars[child][node];
            // (child[node] = 0) -> forall c : (parent[c] != node)
            encoder.add_clause(vec![-c, -p]);
        }
    }

    // Only terminal nodes do not have children
    for node in 1..=num_nodes {
        let t = node_type_vars[node][NodeType::Terminal];
        let c = child_vars[node][0];
        // (node is terminal) <-> (child[node] == 0)
        encoder.add_clause(vec![-t, c]);
        encoder.add_clause(vec![t, -c]);
    }

    // For unary operations, child is parent's child
    for node in 1..=num_nodes {
        for node_type in [NodeType::Not] {
            let t = node_type_vars[node][node_type];
            for child in (node + 1)..=(num_nodes - 1) {
                let p = parent_vars[child][node];
                let c = child_vars[node][child];
                // (node is unary) /\ (child's parent is node) -> (node's child is child)
                encoder.add_clause(vec![-t, -p, c]);
            }
        }
    }

    // For binary operations, left child is parent's child
    for node in 1..=num_nodes {
        for node_type in [NodeType::And, NodeType::Or] {
            let t = node_type_vars[node][node_type];
            for child in (node + 1)..=(num_nodes - 1) {
                let p1 = parent_vars[child][node];
                let p2 = parent_vars[child + 1][node];
                let c = child_vars[node][child];
                // (node is binary) /\ (child's parent is node) -> (node's left child is child)
                encoder.add_clause(vec![-t, -p1, -p2, c]);
            }
        }
    }

    // For binary operations, right child is implicitly "left child + 1"
    for node in 1..=num_nodes {
        for node_type in [NodeType::And, NodeType::Or] {
            let t = node_type_vars[node][node_type];
            for child in (node + 1)..=(num_nodes - 1) {
                let c = child_vars[node][child];
                let p = parent_vars[child + 1][node];
                // (node is binary) /\ (node's child is i) -> (node is also i+1's parent)
                encoder.add_clause(vec![-t, -c, p]);
            }
        }
    }

    // For binary operations, left child cannot be the last node
    for node in 1..=(num_nodes - 1) {
        for node_type in [NodeType::And, NodeType::Or] {
            let t = node_type_vars[node][node_type];
            let c = child_vars[node][num_nodes];
            // (node is binary) -> (node's left child is not the last node)
            encoder.add_clause(vec![-t, -c]);
        }
    }

    // Generate VALUE variables for each node and input row
    let mut value_vars = Map::default();
    for node in 1..=num_nodes {
        for row in 0..truth_table.rows.len() {
            let v = encoder.new_var();
            value_vars.add((node, row), v);
        }
    }

    // Encode the semantics of the nodes

    // ROOT
    {
        let root = 1;
        for (row, &(_, output)) in truth_table.rows.iter().enumerate() {
            let v = value_vars[(root, row)];
            // (root's value is output)
            if output {
                encoder.add_clause(vec![v]);
            } else {
                encoder.add_clause(vec![-v]);
            }
        }
    }
    // TERMINAL
    for node in 1..=num_nodes {
        // let t = node_type_vars[node][NodeType::Terminal];
        for (row, (inputs, _)) in truth_table.rows.iter().enumerate() {
            assert_eq!(inputs.len(), truth_table.variables);
            let v = value_vars[(node, row)];
            for index in 1..=truth_table.variables {
                let i = &index_vars[node][index];
                // (node's variable is i) -> (node's value is input[i])
                if inputs[index - 1] {
                    encoder.add_clause(vec![-i, v]);
                } else {
                    encoder.add_clause(vec![-i, -v]);
                }
            }
        }
    }
    // AND
    for node in 1..num_nodes {
        let t = node_type_vars[node][NodeType::And];
        for child in (node + 1)..=(num_nodes - 1) {
            let c = child_vars[node][child];
            for row in 0..truth_table.rows.len() {
                let v = value_vars[(node, row)];
                let v1 = value_vars[(child, row)];
                let v2 = value_vars[(child + 1, row)];
                // (node is and) /\ (node's child is j) -> (node's value <-> child1.value /\ child2.value)
                encoder.add_clause(vec![-t, -c, v, -v1, -v2]);
                encoder.add_clause(vec![-t, -c, -v, v1]);
                encoder.add_clause(vec![-t, -c, -v, v2]);
            }
        }
    }
    // OR
    for node in 1..num_nodes {
        let t = node_type_vars[node][NodeType::Or];
        for child in (node + 1)..=(num_nodes - 1) {
            let c = child_vars[node][child];
            for row in 0..truth_table.rows.len() {
                let v = value_vars[(node, row)];
                let v1 = value_vars[(child, row)];
                let v2 = value_vars[(child + 1, row)];
                // (node is or) /\ (node's child is j) -> (node's value <-> child1.value \/ child2.value)
                encoder.add_clause(vec![-t, -c, -v, v1, v2]);
                encoder.add_clause(vec![-t, -c, v, -v1]);
                encoder.add_clause(vec![-t, -c, v, -v2]);
            }
        }
    }
    // NOT
    for node in 1..=num_nodes {
        let t = node_type_vars[node][NodeType::Not];
        for child in (node + 1)..=num_nodes {
            let c = child_vars[node][child];
            for row in 0..truth_table.rows.len() {
                let v = value_vars[(node, row)];
                let vc = value_vars[(child, row)];
                // (node is not) /\ (node's child is j) -> (node's value <-> !child.value)
                encoder.add_clause(vec![-t, -c, -v, -vc]);
                encoder.add_clause(vec![-t, -c, v, vc]);
            }
        }
    }

    BooleanFormulaSynthesis {
        node_type: node_type_vars,
        index: index_vars,
        parent: parent_vars,
        child: child_vars,
        value: value_vars,
    }
}

impl BooleanFormulaSynthesis {
    pub fn build_formula(&self, solver: &Cadical) -> BooleanFormula {
        let num_nodes = self.node_type.keys.len();

        let mut formula: Vec<Option<BooleanFormula>> = vec![None; num_nodes];

        for node in (1..=num_nodes).rev() {
            let node_type = *decode_onehot(&self.node_type[node], solver).unwrap();
            match node_type {
                NodeType::Terminal => {
                    let index = *decode_onehot(&self.index[node], solver).unwrap();
                    formula[node - 1] = Some(BooleanFormula::var(index));
                }
                NodeType::And => {
                    let child = *decode_onehot(&self.child[node], solver).unwrap();
                    let left = formula[child - 1].take().unwrap();
                    let right = formula[child].take().unwrap();
                    formula[node - 1] = Some(BooleanFormula::and(left, right));
                }
                NodeType::Or => {
                    let child = *decode_onehot(&self.child[node], solver).unwrap();
                    let left = formula[child - 1].take().unwrap();
                    let right = formula[child].take().unwrap();
                    formula[node - 1] = Some(BooleanFormula::or(left, right));
                }
                NodeType::Not => {
                    let child = *decode_onehot(&self.child[node], solver).unwrap();
                    let child_formula = formula[child - 1].take().unwrap();
                    formula[node - 1] = Some(BooleanFormula::not(child_formula));
                }
            }
        }

        formula[0].take().unwrap()
    }
}
