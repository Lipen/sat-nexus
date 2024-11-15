use std::fmt::Write as _;
use std::ops::Index;

use cadical::statik::Cadical;
use cadical::LitValue;

use crate::formula::BooleanFormula;
use crate::table::TruthTable;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NodeType {
    Terminal,
    And,
    Or,
    Not,
}

pub struct SatEncoder {
    pub num_vars: usize,
    pub clauses: Vec<Vec<i32>>,
}

impl SatEncoder {
    pub fn new(num_vars: usize) -> Self {
        Self {
            num_vars,
            clauses: Vec::new(),
        }
    }
}

impl Default for SatEncoder {
    fn default() -> Self {
        Self::new(0)
    }
}

impl SatEncoder {
    /// Generate a new unique variable ID for SAT encoding.
    fn new_var(&mut self) -> i32 {
        self.num_vars += 1;
        self.num_vars as i32
    }

    /// Add a clause to the SAT encoding in CNF format.
    fn add_clause(&mut self, clause: Vec<i32>) {
        self.clauses.push(clause);
    }

    /// Convert the SAT encoding to DIMACS format for the SAT solver.
    pub fn to_dimacs(&self) -> String {
        let mut output = String::new();
        writeln!(output, "p cnf {} {}", self.num_vars, self.clauses.len()).unwrap();
        for clause in &self.clauses {
            for lit in clause {
                write!(output, "{} ", lit).unwrap();
            }
            writeln!(output, "0").unwrap();
        }
        output
    }
}

impl SatEncoder {
    pub fn exactly_one(&mut self, vars: &[i32]) {
        self.at_least_one(vars);
        self.at_most_one(vars);
    }

    pub fn at_least_one(&mut self, vars: &[i32]) {
        assert!(!vars.is_empty(), "vars must not be empty");
        self.add_clause(vars.to_vec());
    }

    pub fn at_most_one(&mut self, vars: &[i32]) {
        for (i, &var1) in vars.iter().enumerate() {
            for &var2 in vars.iter().skip(i + 1) {
                self.add_clause(vec![-var1, -var2]);
            }
        }
    }
}

#[derive(Debug)]
pub struct DomainVar<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K, V> DomainVar<K, V> {
    pub fn new(keys: Vec<K>, values: Vec<V>) -> Self {
        assert_eq!(keys.len(), values.len());
        Self { keys, values }
    }
}

impl<K, V> Default for DomainVar<K, V> {
    fn default() -> Self {
        Self::new(Vec::new(), Vec::new())
    }
}

impl<K, V> DomainVar<K, V> {
    pub fn add(&mut self, key: K, value: V) {
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn get(&self, key: &K) -> &V
    where
        K: Eq,
        K: std::fmt::Debug,
    {
        let i = self
            .keys
            .iter()
            .position(|k| k == key)
            .unwrap_or_else(|| panic!("key '{:?}' not found in {:?}", key, self.keys));
        &self.values[i]
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.keys.iter().zip(self.values.iter())
    }
}

impl<K, V> Index<K> for DomainVar<K, V>
where
    K: Eq,
    K: std::fmt::Debug,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(&index)
    }
}

impl SatEncoder {
    pub fn new_direct<T>(&mut self, values: Vec<T>) -> DomainVar<T, i32> {
        let variables = values.iter().map(|_| self.new_var()).collect();
        DomainVar::new(values, variables)
    }
}

pub struct Vars {
    pub node_type: DomainVar<usize, DomainVar<NodeType, i32>>,
    pub index: DomainVar<usize, DomainVar<usize, i32>>,
    pub parent: DomainVar<usize, DomainVar<usize, i32>>,
    pub child: DomainVar<usize, DomainVar<usize, i32>>,
    pub value: DomainVar<(usize, usize), i32>,
}

impl SatEncoder {
    pub fn encode(&mut self, num_nodes: usize, truth_table: &TruthTable) -> Vars {
        // Note: most variables are 1-based.
        // Note: 'row' is 0-based index of a row in `truth_table.rows`.

        // Generate TYPE variables for each node and node type
        let mut node_type_vars = DomainVar::default();
        for node in 1..=num_nodes {
            let var = self.new_direct(vec![NodeType::Terminal, NodeType::And, NodeType::Or, NodeType::Not]);

            // Each node must have exactly one type
            self.exactly_one(&var.values);

            node_type_vars.add(node, var);
        }

        // Last node must be a terminal
        {
            let t = node_type_vars[num_nodes][NodeType::Terminal];
            self.add_clause(vec![t]);
        }
        // The node before last cannot be a binary operation
        if num_nodes > 1 {
            let t = node_type_vars[num_nodes - 1][NodeType::And];
            self.add_clause(vec![-t]);
        }
        if num_nodes > 1 {
            let t = node_type_vars[num_nodes - 1][NodeType::Or];
            self.add_clause(vec![-t]);
        }

        // Generate INDEX variables for each node
        let mut index_vars = DomainVar::default();
        for node in 1..=num_nodes {
            let possible_indices = (0..=truth_table.variables).collect();
            let index_var = self.new_direct(possible_indices);

            // Each node must have exactly one variable index associated
            self.exactly_one(&index_var.values);

            index_vars.add(node, index_var);
        }

        // Only terminal nodes can have variable index 0
        for node in 1..=num_nodes {
            let t = node_type_vars[node][NodeType::Terminal];
            let v = index_vars[node][0];
            // (node is terminal) <-> (variable != 0)
            self.add_clause(vec![-t, -v]);
            self.add_clause(vec![t, v]);
        }

        // Generate PARENT variables for each node
        let mut parent_vars = DomainVar::default();
        for node in 2..=num_nodes {
            let possible_parents = (1..node).collect();
            let parent_var = self.new_direct(possible_parents);

            // Each node (except root) must have exactly one parent
            self.exactly_one(&parent_var.values);

            parent_vars.add(node, parent_var);
        }

        // BFS constraint
        // parent[j+1] >= parent[j]
        for node in 3..=(num_nodes - 1) {
            for parent1 in 2..node {
                for parent2 in 1..parent1 {
                    let p1 = parent_vars[node][parent1];
                    let p2 = parent_vars[node + 1][parent2];
                    self.add_clause(vec![-p1, -p2])
                }
            }
        }

        // Generate CHILD variables for each node
        let mut child_vars = DomainVar::default();
        for node in 1..=num_nodes {
            let mut possible_children = vec![0];
            possible_children.extend((node + 1)..=num_nodes);

            let child_var = self.new_direct(possible_children);

            // Each node must have exactly one child
            self.exactly_one(&child_var.values);

            child_vars.add(node, child_var);
        }

        // Encode parent-child relationships
        // child -> parent
        for node in 1..=num_nodes {
            for child in (node + 1)..=num_nodes {
                let c = child_vars[node][child];
                let p = parent_vars[child][node];
                // (child is parent's child) -> (parent is child's parent)
                self.add_clause(vec![-c, p]);
            }
        }
        // no child -> not a parent
        for node in 1..=num_nodes {
            let c = child_vars[node][0];
            for child in (node + 1)..=num_nodes {
                let p = parent_vars[child][node];
                // (child[node] = 0) -> forall c : (parent[c] != node)
                self.add_clause(vec![-c, -p]);
            }
        }

        // Only terminal nodes do not have children
        for node in 1..=num_nodes {
            let t = node_type_vars[node][NodeType::Terminal];
            let c = child_vars[node][0];
            // (node is terminal) <-> (child[node] == 0)
            self.add_clause(vec![-t, c]);
            self.add_clause(vec![t, -c]);
        }

        // For unary operations, child is parent's child
        for node in 1..=num_nodes {
            for node_type in [NodeType::Not] {
                let t = node_type_vars[node][node_type];
                for child in (node + 1)..=(num_nodes - 1) {
                    let p = parent_vars[child][node];
                    let c = child_vars[node][child];
                    // (node is unary) /\ (child's parent is node) -> (node's child is child)
                    self.add_clause(vec![-t, -p, c]);
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
                    self.add_clause(vec![-t, -p1, -p2, c]);
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
                    self.add_clause(vec![-t, -c, p]);
                }
            }
        }

        // For binary operations, left child cannot be the last node
        for node in 1..=(num_nodes - 1) {
            for node_type in [NodeType::And, NodeType::Or] {
                let t = node_type_vars[node][node_type];
                let c = child_vars[node][num_nodes];
                // (node is binary) -> (node's left child is not the last node)
                self.add_clause(vec![-t, -c]);
            }
        }

        // Generate VALUE variables for each node and input row
        let mut value_vars = DomainVar::default();
        for node in 1..=num_nodes {
            for row in 0..truth_table.rows.len() {
                let v = self.new_var();
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
                    self.add_clause(vec![v]);
                } else {
                    self.add_clause(vec![-v]);
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
                        self.add_clause(vec![-i, v]);
                    } else {
                        self.add_clause(vec![-i, -v]);
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
                    self.add_clause(vec![-t, -c, v, -v1, -v2]);
                    self.add_clause(vec![-t, -c, -v, v1]);
                    self.add_clause(vec![-t, -c, -v, v2]);
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
                    self.add_clause(vec![-t, -c, -v, v1, v2]);
                    self.add_clause(vec![-t, -c, v, -v1]);
                    self.add_clause(vec![-t, -c, v, -v2]);
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
                    self.add_clause(vec![-t, -c, -v, -vc]);
                    self.add_clause(vec![-t, -c, v, vc]);
                }
            }
        }

        Vars {
            node_type: node_type_vars,
            index: index_vars,
            parent: parent_vars,
            child: child_vars,
            value: value_vars,
        }
    }
}

pub fn decode_onehot<'a, T>(var: &'a DomainVar<T, i32>, solver: &Cadical) -> Option<&'a T> {
    var.iter().find_map(|(key, &t)| {
        if solver.val(t).unwrap() == LitValue::True {
            Some(key)
        } else {
            None
        }
    })
}

impl Vars {
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
