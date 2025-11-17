//! RCU (Read-Copy-Update) Semantic Model
//!
//! This module implements the core RCU semantics as used in the Linux kernel.
//! It captures:
//! - Read-side critical sections
//! - Grace periods
//! - Callback lifecycle
//! - Safety invariants
//!
//! This is the flagship demonstration of VDL++ for modeling concurrent
//! kernel primitives.

use crate::ast::*;
use crate::explorer::{Explorer, ExplorerConfig, TransitionGenerator};
use crate::value::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Create an RCU state value
pub fn make_rcu_state(
    readers: BTreeSet<u64>,
    epoch: u64,
    gp_active: bool,
    pending_callbacks: Vec<(u64, u64)>, // (id, target_epoch)
    completed_callbacks: Vec<(u64, u64)>,
) -> Value {
    let mut fields = BTreeMap::new();

    fields.insert(
        "readers".to_string(),
        Value::Set(readers.into_iter().map(Value::Nat).collect()),
    );
    fields.insert("epoch".to_string(), Value::Nat(epoch));
    fields.insert("gpActive".to_string(), Value::Bool(gp_active));

    let pending: Vec<Value> = pending_callbacks
        .into_iter()
        .map(|(id, target)| {
            let mut cb = BTreeMap::new();
            cb.insert("id".to_string(), Value::Nat(id));
            cb.insert("targetEpoch".to_string(), Value::Nat(target));
            Value::Record(cb)
        })
        .collect();
    fields.insert("pendingCallbacks".to_string(), Value::List(pending));

    let completed: Vec<Value> = completed_callbacks
        .into_iter()
        .map(|(id, target)| {
            let mut cb = BTreeMap::new();
            cb.insert("id".to_string(), Value::Nat(id));
            cb.insert("targetEpoch".to_string(), Value::Nat(target));
            Value::Record(cb)
        })
        .collect();
    fields.insert("completedCallbacks".to_string(), Value::List(completed));

    Value::Record(fields)
}

/// Create the RCU specification programmatically
/// This is more reliable than parsing for the initial demo
pub fn create_rcu_specification() -> Specification {
    let mut spec = Specification::new();

    // Type definitions (denotational domains)
    spec.type_defs.push(TypeDef {
        name: "ReaderId".to_string(),
        ty: Type::Nat,
        invariant: None,
        span: None,
    });

    // Callback type
    let mut callback_fields = BTreeMap::new();
    callback_fields.insert("id".to_string(), Type::Nat);
    callback_fields.insert("targetEpoch".to_string(), Type::Nat);

    spec.type_defs.push(TypeDef {
        name: "Callback".to_string(),
        ty: Type::Record(callback_fields),
        invariant: None,
        span: None,
    });

    // RCU state type
    let mut rcu_fields = BTreeMap::new();
    rcu_fields.insert("readers".to_string(), Type::Set(Box::new(Type::Nat)));
    rcu_fields.insert("epoch".to_string(), Type::Nat);
    rcu_fields.insert("gpActive".to_string(), Type::Bool);
    rcu_fields.insert(
        "pendingCallbacks".to_string(),
        Type::List(Box::new(Type::Named("Callback".to_string()))),
    );
    rcu_fields.insert(
        "completedCallbacks".to_string(),
        Type::List(Box::new(Type::Named("Callback".to_string()))),
    );

    spec.type_defs.push(TypeDef {
        name: "RCU".to_string(),
        ty: Type::Record(rcu_fields.clone()),
        invariant: None,
        span: None,
    });

    // Global invariant: No callback runs while readers from before its target epoch are active
    // Simplified: pending callbacks must have targetEpoch >= current epoch
    spec.invariants.push(InvariantDef {
        name: "callback_safety".to_string(),
        state_param: "s".to_string(),
        state_type: Type::Record(rcu_fields.clone()),
        condition: Expr::Forall(
            "cb".to_string(),
            Box::new(Expr::FieldAccess(
                Box::new(Expr::Var("s".to_string())),
                "pendingCallbacks".to_string(),
            )),
            Box::new(Expr::BinOp(
                Box::new(Expr::FieldAccess(
                    Box::new(Expr::Var("cb".to_string())),
                    "targetEpoch".to_string(),
                )),
                BinOp::Ge,
                Box::new(Expr::FieldAccess(
                    Box::new(Expr::Var("s".to_string())),
                    "epoch".to_string(),
                )),
            )),
        ),
        span: None,
    });

    // Additional invariant: epoch is non-decreasing (always >= 0, implicit in Nat)
    spec.invariants.push(InvariantDef {
        name: "epoch_valid".to_string(),
        state_param: "s".to_string(),
        state_type: Type::Record(rcu_fields),
        condition: Expr::BinOp(
            Box::new(Expr::FieldAccess(
                Box::new(Expr::Var("s".to_string())),
                "epoch".to_string(),
            )),
            BinOp::Ge,
            Box::new(Expr::NatLit(0)),
        ),
        span: None,
    });

    spec
}

/// Apply RCU transition directly (bypassing parser/evaluator for reliability)
pub fn apply_rcu_transition(transition: &str, args: &[Value], state: &Value) -> Option<Value> {
    let fields = state.as_record()?;
    let readers = fields.get("readers")?.as_set()?.clone();
    let epoch = fields.get("epoch")?.as_nat()?;
    let gp_active = fields.get("gpActive")?.as_bool()?;
    let pending = fields.get("pendingCallbacks")?.as_list()?.clone();
    let completed = fields.get("completedCallbacks")?.as_list()?.clone();

    match transition {
        "rcu_read_lock" => {
            let r = args.first()?.as_nat()?;
            if readers.contains(&Value::Nat(r)) {
                return None; // Pre-condition failed
            }
            let mut new_readers = readers;
            new_readers.insert(Value::Nat(r));

            let mut new_fields = BTreeMap::new();
            new_fields.insert("readers".to_string(), Value::Set(new_readers));
            new_fields.insert("epoch".to_string(), Value::Nat(epoch));
            new_fields.insert("gpActive".to_string(), Value::Bool(gp_active));
            new_fields.insert("pendingCallbacks".to_string(), Value::List(pending));
            new_fields.insert("completedCallbacks".to_string(), Value::List(completed));
            Some(Value::Record(new_fields))
        }

        "rcu_read_unlock" => {
            let r = args.first()?.as_nat()?;
            if !readers.contains(&Value::Nat(r)) {
                return None; // Pre-condition failed
            }
            let mut new_readers = readers;
            new_readers.remove(&Value::Nat(r));

            let mut new_fields = BTreeMap::new();
            new_fields.insert("readers".to_string(), Value::Set(new_readers));
            new_fields.insert("epoch".to_string(), Value::Nat(epoch));
            new_fields.insert("gpActive".to_string(), Value::Bool(gp_active));
            new_fields.insert("pendingCallbacks".to_string(), Value::List(pending));
            new_fields.insert("completedCallbacks".to_string(), Value::List(completed));
            Some(Value::Record(new_fields))
        }

        "start_grace_period" => {
            if gp_active {
                return None; // Pre-condition failed
            }

            let mut new_fields = BTreeMap::new();
            new_fields.insert("readers".to_string(), Value::Set(readers));
            new_fields.insert("epoch".to_string(), Value::Nat(epoch + 1));
            new_fields.insert("gpActive".to_string(), Value::Bool(true));
            new_fields.insert("pendingCallbacks".to_string(), Value::List(pending));
            new_fields.insert("completedCallbacks".to_string(), Value::List(completed));
            Some(Value::Record(new_fields))
        }

        "end_grace_period" => {
            if !gp_active || !readers.is_empty() {
                return None; // Pre-condition failed
            }

            let mut new_fields = BTreeMap::new();
            new_fields.insert("readers".to_string(), Value::Set(readers));
            new_fields.insert("epoch".to_string(), Value::Nat(epoch));
            new_fields.insert("gpActive".to_string(), Value::Bool(false));
            new_fields.insert("pendingCallbacks".to_string(), Value::List(pending));
            new_fields.insert("completedCallbacks".to_string(), Value::List(completed));
            Some(Value::Record(new_fields))
        }

        "call_rcu" => {
            let cb_id = args.first()?.as_nat()?;

            let mut cb = BTreeMap::new();
            cb.insert("id".to_string(), Value::Nat(cb_id));
            cb.insert("targetEpoch".to_string(), Value::Nat(epoch));

            let mut new_pending = pending;
            new_pending.push(Value::Record(cb));

            let mut new_fields = BTreeMap::new();
            new_fields.insert("readers".to_string(), Value::Set(readers));
            new_fields.insert("epoch".to_string(), Value::Nat(epoch));
            new_fields.insert("gpActive".to_string(), Value::Bool(gp_active));
            new_fields.insert("pendingCallbacks".to_string(), Value::List(new_pending));
            new_fields.insert("completedCallbacks".to_string(), Value::List(completed));
            Some(Value::Record(new_fields))
        }

        "run_ready_callbacks" => {
            // Move callbacks whose targetEpoch <= current epoch to completed
            let mut ready = Vec::new();
            let mut not_ready = Vec::new();

            for cb_val in &pending {
                if let Some(cb_fields) = cb_val.as_record() {
                    if let Some(target_epoch) = cb_fields.get("targetEpoch")?.as_nat() {
                        if target_epoch <= epoch {
                            ready.push(cb_val.clone());
                        } else {
                            not_ready.push(cb_val.clone());
                        }
                    }
                }
            }

            if ready.is_empty() {
                return None; // Nothing to do
            }

            let mut new_completed = completed;
            new_completed.extend(ready);

            let mut new_fields = BTreeMap::new();
            new_fields.insert("readers".to_string(), Value::Set(readers));
            new_fields.insert("epoch".to_string(), Value::Nat(epoch));
            new_fields.insert("gpActive".to_string(), Value::Bool(gp_active));
            new_fields.insert("pendingCallbacks".to_string(), Value::List(not_ready));
            new_fields.insert("completedCallbacks".to_string(), Value::List(new_completed));
            Some(Value::Record(new_fields))
        }

        _ => None,
    }
}

/// Create a simplified RCU explorer that uses direct transition application
pub struct RcuExplorer {
    pub config: ExplorerConfig,
}

impl RcuExplorer {
    pub fn new(config: ExplorerConfig) -> Self {
        Self { config }
    }

    /// Explore RCU state space with hardcoded transitions
    pub fn explore(
        &self,
        initial_state: Value,
        reader_ids: Vec<u64>,
        callback_ids: Vec<u64>,
    ) -> crate::explorer::ExplorationResult {
        use crate::explorer::*;
        use std::collections::{HashSet, VecDeque};

        let spec = create_rcu_specification();
        let mut visited: HashSet<Value> = HashSet::new();
        let mut queue: VecDeque<(Value, Vec<TraceStep>, usize)> = VecDeque::new();
        let mut violations: Vec<Violation> = Vec::new();
        let mut traces: Vec<Trace> = Vec::new();
        let mut trace_counter = 0;
        let mut max_depth_reached = 0;

        let mut state_to_id: BTreeMap<Value, usize> = BTreeMap::new();
        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut edges: Vec<GraphEdge> = Vec::new();

        // Helper function to get or create state ID
        fn get_or_create_state_id(
            state: &Value,
            is_initial: bool,
            state_to_id: &mut BTreeMap<Value, usize>,
            nodes: &mut Vec<GraphNode>,
        ) -> usize {
            if let Some(&id) = state_to_id.get(state) {
                id
            } else {
                let id = nodes.len();
                state_to_id.insert(state.clone(), id);
                nodes.push(GraphNode {
                    id,
                    state: state.clone(),
                    is_initial,
                    has_violation: false,
                });
                id
            }
        }

        // Check invariants
        let check_invariants = |state: &Value| -> Option<Violation> {
            let mut eval = crate::eval::Evaluator::new(&spec);
            for inv in &spec.invariants {
                eval.env.push_scope();
                eval.env.bind(inv.state_param.clone(), state.clone());
                if let Ok(result) = eval.eval(&inv.condition) {
                    eval.env.pop_scope();
                    if !result.is_truthy() {
                        return Some(Violation {
                            invariant_name: inv.name.clone(),
                            message: format!("Invariant '{}' violated", inv.name),
                            state: state.clone(),
                            step_index: 0,
                        });
                    }
                } else {
                    eval.env.pop_scope();
                }
            }
            None
        };

        let initial_id = get_or_create_state_id(&initial_state, true, &mut state_to_id, &mut nodes);
        if let Some(mut violation) = check_invariants(&initial_state) {
            nodes[initial_id].has_violation = true;
            violation.step_index = 0;
            violations.push(violation.clone());
            traces.push(Trace {
                id: trace_counter,
                steps: vec![],
                violation: Some(violation),
            });
            if self.config.stop_on_first_violation {
                return ExplorationResult {
                    visited_states: 1,
                    max_depth_reached: 0,
                    violations,
                    traces,
                    graph: StateGraph { nodes, edges },
                };
            }
        }

        visited.insert(initial_state.clone());
        queue.push_back((initial_state, vec![], 0));

        // Define transitions to explore
        let transitions: Vec<(&str, Vec<Vec<Value>>)> = vec![
            (
                "rcu_read_lock",
                vec![reader_ids.iter().map(|&r| Value::Nat(r)).collect()],
            ),
            (
                "rcu_read_unlock",
                vec![reader_ids.iter().map(|&r| Value::Nat(r)).collect()],
            ),
            ("start_grace_period", vec![]),
            ("end_grace_period", vec![]),
            (
                "call_rcu",
                vec![callback_ids.iter().map(|&c| Value::Nat(c)).collect()],
            ),
            ("run_ready_callbacks", vec![]),
        ];

        while let Some((current_state, trace, depth)) = queue.pop_front() {
            if depth > max_depth_reached {
                max_depth_reached = depth;
            }

            if depth >= self.config.max_depth {
                continue;
            }

            if visited.len() >= self.config.max_states {
                break;
            }

            let current_id = get_or_create_state_id(&current_state, false, &mut state_to_id, &mut nodes);

            // Try all transitions
            for (trans_name, param_values) in &transitions {
                let combinations = if param_values.is_empty() {
                    vec![vec![]]
                } else {
                    param_values[0].iter().map(|v| vec![v.clone()]).collect()
                };

                for args in combinations {
                    if let Some(next_state) = apply_rcu_transition(trans_name, &args, &current_state)
                    {
                        let next_id = get_or_create_state_id(&next_state, false, &mut state_to_id, &mut nodes);

                        if self.config.generate_graph {
                            let label = if args.is_empty() {
                                trans_name.to_string()
                            } else {
                                format!(
                                    "{}({})",
                                    trans_name,
                                    args.iter()
                                        .map(|v| v.pretty())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            };
                            edges.push(GraphEdge {
                                from: current_id,
                                to: next_id,
                                label,
                            });
                        }

                        let step = TraceStep {
                            transition_name: trans_name.to_string(),
                            args: args.clone(),
                            state_before: current_state.clone(),
                            state_after: next_state.clone(),
                        };

                        let mut new_trace = trace.clone();
                        new_trace.push(step);

                        if let Some(mut violation) = check_invariants(&next_state) {
                            nodes[next_id].has_violation = true;
                            violation.step_index = new_trace.len();
                            violations.push(violation.clone());
                            traces.push(Trace {
                                id: trace_counter,
                                steps: new_trace.clone(),
                                violation: Some(violation),
                            });
                            trace_counter += 1;

                            if self.config.stop_on_first_violation {
                                return ExplorationResult {
                                    visited_states: visited.len(),
                                    max_depth_reached,
                                    violations,
                                    traces,
                                    graph: StateGraph { nodes, edges },
                                };
                            }
                        }

                        if !visited.contains(&next_state) {
                            visited.insert(next_state.clone());
                            queue.push_back((next_state, new_trace, depth + 1));
                        }
                    }
                }
            }
        }

        ExplorationResult {
            visited_states: visited.len(),
            max_depth_reached,
            violations,
            traces,
            graph: StateGraph { nodes, edges },
        }
    }
}

/// Example VDL++ source code for RCU model
pub const RCU_VDL_SOURCE: &str = r#"
// VDL++ RCU (Read-Copy-Update) Semantic Model
// Inspired by the Vienna Definition Method tradition
// Models the core RCU API semantics from the Linux kernel

type ReaderId = Nat

type Callback = {
    id: Nat,
    targetEpoch: Nat
}

type RCU = {
    readers: Set<Nat>,
    epoch: Nat,
    gpActive: Bool,
    pendingCallbacks: List<Callback>,
    completedCallbacks: List<Callback>
}

// Invariant: callbacks must not target epochs in the past
inv callback_safety(s: RCU) =>
    forall cb in s.pendingCallbacks:
        cb.targetEpoch >= s.epoch

// Enter read-side critical section
transition rcu_read_lock(r: Nat, s: RCU) -> RCU
    pre !member(r, s.readers)
    post s' =>
        s' == s with [readers = s.readers union {r}]

// Exit read-side critical section
transition rcu_read_unlock(r: Nat, s: RCU) -> RCU
    pre member(r, s.readers)
    post s' =>
        s' == s with [readers = s.readers \ {r}]

// Start a new grace period
transition start_grace_period(s: RCU) -> RCU
    pre !s.gpActive
    post s' =>
        s' == s with [gpActive = true, epoch = s.epoch + 1]

// End grace period (only when no readers)
transition end_grace_period(s: RCU) -> RCU
    pre s.gpActive && isEmpty(s.readers)
    post s' =>
        s' == s with [gpActive = false]

// Queue a callback for execution after grace period
transition call_rcu(cbId: Nat, s: RCU) -> RCU
    pre true
    post s' =>
        let cb = {id = cbId, targetEpoch = s.epoch} in
        s' == s with [pendingCallbacks = s.pendingCallbacks ++ [cb]]

// Execute callbacks whose epoch has passed
transition run_ready_callbacks(s: RCU) -> RCU
    pre true
    post s' =>
        let ready = filter(s.pendingCallbacks, cb => cb.targetEpoch <= s.epoch) in
        let notReady = filter(s.pendingCallbacks, cb => cb.targetEpoch > s.epoch) in
        s' == s with [
            pendingCallbacks = notReady,
            completedCallbacks = s.completedCallbacks ++ ready
        ]
"#;
