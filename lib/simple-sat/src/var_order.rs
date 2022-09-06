use tracing::debug;

use crate::assignment::Assignment;
use crate::idx::{VarHeap, VarVec};
use crate::var::Var;

#[derive(Debug)]
pub struct VarOrder {
    activity: VarVec<f64>,
    order_heap: VarHeap,
    var_decay: f64,
    var_inc: f64,
}

impl VarOrder {
    pub fn new() -> Self {
        Self {
            activity: VarVec::new(),
            order_heap: VarHeap::new(),
            var_decay: 0.95,
            var_inc: 1.0,
        }
    }
}

impl Default for VarOrder {
    fn default() -> Self {
        Self::new()
    }
}

impl VarOrder {
    pub(crate) fn push_zero_activity(&mut self) {
        self.activity.push(0.0);
    }

    pub fn var_decay_activity(&mut self) {
        self.var_inc /= self.var_decay;
    }

    pub fn var_bump_activity(&mut self, var: Var) {
        let new = self.activity[var] + self.var_inc;
        self.activity[var] = new;

        // Rescale large activities, if necessary:
        if new > 1e100 {
            self.var_rescale_activity();
        }

        // Update `var` in heap:
        if self.order_heap.contains(&var) {
            self.update_var_order(var);
        }
    }

    pub fn var_rescale_activity(&mut self) {
        debug!("Rescaling activity");

        // Decrease the increment value:
        self.var_inc *= 1e-100;

        // Decrease all activities:
        for a in self.activity.iter_mut() {
            *a *= 1e-100;
        }
    }

    pub fn insert_var_order(&mut self, var: Var) {
        self.order_heap.insert_by(var, |&a, &b| self.activity[a] > self.activity[b]);
        // self.order_heap.insert_by(var, |&a, &b| match act[a].total_cmp(&act[b]) {
        //     Ordering::Less => false,
        //     Ordering::Equal => a.0 < b.0,
        //     Ordering::Greater => true,
        // });
    }

    pub fn update_var_order(&mut self, var: Var) {
        self.order_heap.update_by(var, |&a, &b| self.activity[a] > self.activity[b]);
        // self.order_heap.update_by(var, |&a, &b| match self.activity[a].total_cmp(&self.activity[b]) {
        //     Ordering::Less => false,
        //     Ordering::Equal => a.0 < b.0,
        //     Ordering::Greater => true,
        // });
    }

    pub fn pick_branching_variable(&mut self, assignment: &Assignment) -> Option<Var> {
        self.order_heap
            .sorted_iter_by(|&a, &b| self.activity[a] > self.activity[b])
            // .sorted_iter_by(|&a, &b| match self.activity[a].total_cmp(&self.activity[b]) {
            //     Ordering::Less => false,
            //     Ordering::Equal => a.0 < b.0,
            //     Ordering::Greater => true,
            // })
            .find(|&var| assignment.value_var(var).is_undef())
    }
}
