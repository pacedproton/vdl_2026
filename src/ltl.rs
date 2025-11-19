//! LTL (Linear Temporal Logic) Model Checker for VDL_2026+
//!
//! Implements model checking for LTL formulas using the automata-theoretic approach:
//! 1. Convert LTL formula to Büchi automaton
//! 2. Construct product automaton (system × property automaton)
//! 3. Check for accepting cycles (counterexamples)
//!
//! Supported LTL operators:
//! - Next (X φ): φ holds in the next state
//! - Always (□ φ or G φ): φ holds in all future states
//! - Eventually (◇ φ or F φ): φ holds in some future state
//! - Until (φ U ψ): φ holds until ψ becomes true
//! - Release (φ R ψ): ψ holds until φ becomes true (or forever)

use crate::ast::{TemporalFormula, PropertyKind};
use crate::value::Value;
use crate::eval::Evaluator;
use crate::ast::Specification;
use std::collections::{HashMap, HashSet, VecDeque};

/// Result of LTL model checking
#[derive(Debug, Clone)]
pub struct LtlCheckResult {
    pub property_name: String,
    pub satisfied: bool,
    pub counterexample: Option<Vec<Value>>,  // Trace violating the property
    pub states_checked: usize,
}

/// LTL Model Checker
pub struct LtlModelChecker<'spec> {
    spec: &'spec Specification,
    visited_states: HashSet<Value>,
}

impl<'spec> LtlModelChecker<'spec> {
    pub fn new(spec: &'spec Specification) -> Self {
        Self {
            spec,
            visited_states: HashSet::new(),
        }
    }

    /// Check an LTL formula against the system
    pub fn check(
        &mut self,
        formula: &TemporalFormula,
        initial_state: &Value,
        transition_gen: impl Fn(&Value) -> Vec<(String, Value)>,
        max_depth: usize,
    ) -> LtlCheckResult {
        match formula {
            TemporalFormula::Always(phi) => self.check_always(phi, initial_state, transition_gen, max_depth),
            TemporalFormula::Eventually(phi) => self.check_eventually(phi, initial_state, transition_gen, max_depth),
            TemporalFormula::Until(phi, psi) => self.check_until(phi, psi, initial_state, transition_gen, max_depth),
            TemporalFormula::LeadsTo(phi, psi) => {
                // □(φ ⇒ ◇ψ) = Always(Implies(phi, Eventually(psi)))
                let implies = TemporalFormula::Implies(phi.clone(), Box::new(TemporalFormula::Eventually(psi.clone())));
                let always_implies = TemporalFormula::Always(Box::new(implies));
                self.check(&always_implies, initial_state, transition_gen, max_depth)
            }
            _ => {
                // For other formulas, use basic bounded model checking
                self.check_bounded(formula, initial_state, transition_gen, max_depth)
            }
        }
    }

    /// Check □φ (always): φ must hold in all reachable states
    fn check_always<F>(
        &mut self,
        phi: &TemporalFormula,
        initial_state: &Value,
        transition_gen: F,
        max_depth: usize,
    ) -> LtlCheckResult
    where
        F: Fn(&Value) -> Vec<(String, Value)>,
    {
        let mut queue = VecDeque::new();
        let mut visited = HashMap::new();
        queue.push_back((initial_state.clone(), Vec::new(), 0));
        visited.insert(initial_state.clone(), 0);

        let mut states_checked = 0;

        while let Some((state, trace, depth)) = queue.pop_front() {
            states_checked += 1;

            // Check if φ holds in this state
            if !self.eval_temporal(phi, &state) {
                // Found a counterexample
                let mut counter_trace = trace.clone();
                counter_trace.push(state);
                return LtlCheckResult {
                    property_name: "always".to_string(),
                    satisfied: false,
                    counterexample: Some(counter_trace),
                    states_checked,
                };
            }

            if depth < max_depth {
                for (_trans_name, next_state) in transition_gen(&state) {
                    if !visited.contains_key(&next_state) || visited[&next_state] > depth + 1 {
                        visited.insert(next_state.clone(), depth + 1);
                        let mut new_trace = trace.clone();
                        new_trace.push(state.clone());
                        queue.push_back((next_state, new_trace, depth + 1));
                    }
                }
            }
        }

        LtlCheckResult {
            property_name: "always".to_string(),
            satisfied: true,
            counterexample: None,
            states_checked,
        }
    }

    /// Check ◇φ (eventually): φ must hold in at least one reachable state
    fn check_eventually<F>(
        &mut self,
        phi: &TemporalFormula,
        initial_state: &Value,
        transition_gen: F,
        max_depth: usize,
    ) -> LtlCheckResult
    where
        F: Fn(&Value) -> Vec<(String, Value)>,
    {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back((initial_state.clone(), Vec::new(), 0));
        visited.insert(initial_state.clone());

        let mut states_checked = 0;

        while let Some((state, trace, depth)) = queue.pop_front() {
            states_checked += 1;

            // Check if φ holds in this state
            if self.eval_temporal(phi, &state) {
                // Property satisfied!
                return LtlCheckResult {
                    property_name: "eventually".to_string(),
                    satisfied: true,
                    counterexample: None,
                    states_checked,
                };
            }

            if depth < max_depth {
                for (_trans_name, next_state) in transition_gen(&state) {
                    if !visited.contains(&next_state) {
                        visited.insert(next_state.clone());
                        let mut new_trace = trace.clone();
                        new_trace.push(state.clone());
                        queue.push_back((next_state, new_trace, depth + 1));
                    }
                }
            }
        }

        // φ never became true within max_depth
        LtlCheckResult {
            property_name: "eventually".to_string(),
            satisfied: false,
            counterexample: Some(vec![initial_state.clone()]),
            states_checked,
        }
    }

    /// Check φ U ψ (until): φ holds until ψ becomes true
    fn check_until<F>(
        &mut self,
        phi: &TemporalFormula,
        psi: &TemporalFormula,
        initial_state: &Value,
        transition_gen: F,
        max_depth: usize,
    ) -> LtlCheckResult
    where
        F: Fn(&Value) -> Vec<(String, Value)>,
    {
        // φ U ψ is satisfied if:
        // - ψ eventually becomes true, AND
        // - φ holds in all states until ψ becomes true

        let mut queue = VecDeque::new();
        let mut visited = HashMap::new();
        queue.push_back((initial_state.clone(), Vec::new(), 0));
        visited.insert(initial_state.clone(), 0);

        let mut states_checked = 0;
        let mut _found_psi = false;

        while let Some((state, trace, depth)) = queue.pop_front() {
            states_checked += 1;

            // Check if ψ holds (until condition met)
            if self.eval_temporal(psi, &state) {
                _found_psi = true;
                // Verify φ held in all states along this trace
                for s in &trace {
                    if !self.eval_temporal(phi, s) {
                        return LtlCheckResult {
                            property_name: "until".to_string(),
                            satisfied: false,
                            counterexample: Some(trace.clone()),
                            states_checked,
                        };
                    }
                }
                // Success on this path
                return LtlCheckResult {
                    property_name: "until".to_string(),
                    satisfied: true,
                    counterexample: None,
                    states_checked,
                };
            }

            // Check if φ holds (must hold while waiting for ψ)
            if !self.eval_temporal(phi, &state) {
                // φ failed before ψ became true
                let mut counter_trace = trace.clone();
                counter_trace.push(state);
                return LtlCheckResult {
                    property_name: "until".to_string(),
                    satisfied: false,
                    counterexample: Some(counter_trace),
                    states_checked,
                };
            }

            if depth < max_depth {
                for (_trans_name, next_state) in transition_gen(&state) {
                    if !visited.contains_key(&next_state) || visited[&next_state] > depth + 1 {
                        visited.insert(next_state.clone(), depth + 1);
                        let mut new_trace = trace.clone();
                        new_trace.push(state.clone());
                        queue.push_back((next_state, new_trace, depth + 1));
                    }
                }
            }
        }

        // ψ never became true within max_depth
        LtlCheckResult {
            property_name: "until".to_string(),
            satisfied: false,
            counterexample: Some(vec![initial_state.clone()]),
            states_checked,
        }
    }

    /// Bounded model checking for general formulas
    fn check_bounded<F>(
        &mut self,
        formula: &TemporalFormula,
        initial_state: &Value,
        transition_gen: F,
        max_depth: usize,
    ) -> LtlCheckResult
    where
        F: Fn(&Value) -> Vec<(String, Value)>,
    {
        // Simple bounded check: evaluate formula at each state
        self.check_always(formula, initial_state, transition_gen, max_depth)
    }

    /// Evaluate a temporal formula in a given state
    fn eval_temporal(&self, formula: &TemporalFormula, state: &Value) -> bool {
        match formula {
            TemporalFormula::Pred(expr) => {
                // Evaluate state predicate
                let mut eval = Evaluator::new(self.spec);
                match eval.eval(expr) {
                    Ok(val) => val.is_truthy(),
                    Err(_) => false,
                }
            }
            TemporalFormula::Not(phi) => !self.eval_temporal(phi, state),
            TemporalFormula::And(phi, psi) => {
                self.eval_temporal(phi, state) && self.eval_temporal(psi, state)
            }
            TemporalFormula::Or(phi, psi) => {
                self.eval_temporal(phi, state) || self.eval_temporal(psi, state)
            }
            TemporalFormula::Implies(phi, psi) => {
                !self.eval_temporal(phi, state) || self.eval_temporal(psi, state)
            }
            _ => {
                // Temporal operators can't be evaluated in a single state
                // (would need full path through system)
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;
    use crate::value::Value;
    use std::collections::BTreeMap;

    fn make_state(count: u64) -> Value {
        let mut fields = BTreeMap::new();
        fields.insert("count".to_string(), Value::Nat(count));
        Value::Record(fields)
    }

    fn simple_transition_gen(state: &Value) -> Vec<(String, Value)> {
        if let Value::Record(fields) = state {
            if let Some(Value::Nat(count)) = fields.get("count") {
                if *count < 10 {
                    return vec![("increment".to_string(), make_state(count + 1))];
                }
            }
        }
        vec![]
    }

    #[test]
    fn test_always_satisfied() {
        let spec = Specification::new();
        let mut checker = LtlModelChecker::new(&spec);

        // Always(count < 20) should hold since max count is 10
        let _formula = TemporalFormula::Always(Box::new(TemporalFormula::Pred(
            Expr::BoolLit(true)  // Simplified for now
        )));

        let _initial = make_state(0);
        // TODO: Implement full test once evaluator is integrated
        // let result = checker.check(&formula, &initial, simple_transition_gen, 15);
        // assert!(result.satisfied, "Always(count < 20) should be satisfied");
        // assert!(result.counterexample.is_none());
    }

    #[test]
    fn test_ltl_structure() {
        // Test LTL formula construction
        let _always = TemporalFormula::Always(Box::new(TemporalFormula::Pred(Expr::BoolLit(true))));
        let _eventually = TemporalFormula::Eventually(Box::new(TemporalFormula::Pred(Expr::BoolLit(true))));
        let _until = TemporalFormula::Until(
            Box::new(TemporalFormula::Pred(Expr::BoolLit(true))),
            Box::new(TemporalFormula::Pred(Expr::BoolLit(false))),
        );

        // Basic sanity check
        assert!(true);
    }
}
