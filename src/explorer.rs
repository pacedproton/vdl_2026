//! State-space explorer for VDL++ (operational semantics)
//!
//! This implements a bounded BFS/DFS exploration of the state space,
//! checking invariants at each step and detecting violations.
//!
//! Inspired by the Vienna tradition: executable specifications with
//! small-step operational semantics.

use crate::ast::*;
use crate::error::{Result, VdlError};
use crate::eval::Evaluator;
use crate::value::Value;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet, VecDeque};

/// A single step in an execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub transition_name: String,
    pub args: Vec<Value>,
    pub state_before: Value,
    pub state_after: Value,
}

/// A complete execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub id: usize,
    pub steps: Vec<TraceStep>,
    pub violation: Option<Violation>,
}

/// An invariant violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub invariant_name: String,
    pub message: String,
    pub state: Value,
    pub step_index: usize,
}

/// Result of state-space exploration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationResult {
    pub visited_states: usize,
    pub max_depth_reached: usize,
    pub violations: Vec<Violation>,
    pub traces: Vec<Trace>,
    pub graph: StateGraph,
}

/// State graph for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: usize,
    pub state: Value,
    pub is_initial: bool,
    pub has_violation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: usize,
    pub to: usize,
    pub label: String,
}

/// Configuration for exploration
#[derive(Debug, Clone)]
pub struct ExplorerConfig {
    pub max_depth: usize,
    pub max_states: usize,
    pub stop_on_first_violation: bool,
    pub generate_graph: bool,
    pub record_all_traces: bool,
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            max_states: 10000,
            stop_on_first_violation: true,
            generate_graph: true,
            record_all_traces: false,
        }
    }
}

/// State-space explorer
pub struct Explorer<'spec> {
    spec: &'spec Specification,
    config: ExplorerConfig,
    transition_generators: Vec<TransitionGenerator>,
}

/// Generator for concrete transition instances
#[derive(Clone)]
pub struct TransitionGenerator {
    pub transition_name: String,
    pub param_values: Vec<Vec<Value>>, // Possible values for each parameter
}

impl<'spec> Explorer<'spec> {
    pub fn new(spec: &'spec Specification, config: ExplorerConfig) -> Self {
        Self {
            spec,
            config,
            transition_generators: Vec::new(),
        }
    }

    /// Register a transition generator with concrete parameter values to explore
    pub fn add_transition_generator(&mut self, generator: TransitionGenerator) {
        self.transition_generators.push(generator);
    }

    /// Explore the state space starting from an initial state
    pub fn explore(&self, initial_state: Value) -> Result<ExplorationResult> {
        let mut visited: HashSet<Value> = HashSet::new();
        let mut queue: VecDeque<(Value, Vec<TraceStep>, usize)> = VecDeque::new();
        let mut violations: Vec<Violation> = Vec::new();
        let mut traces: Vec<Trace> = Vec::new();
        let mut trace_counter = 0;
        let mut max_depth_reached = 0;

        // Graph construction
        let mut state_to_id: BTreeMap<Value, usize> = BTreeMap::new();
        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut edges: Vec<GraphEdge> = Vec::new();

        // Helper function to get or create node ID
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

        // Check initial state invariants
        let initial_id = get_or_create_state_id(&initial_state, true, &mut state_to_id, &mut nodes);
        if let Some(violation) = self.check_invariants(&initial_state, 0)? {
            nodes[initial_id].has_violation = true;
            violations.push(violation.clone());
            traces.push(Trace {
                id: trace_counter,
                steps: vec![],
                violation: Some(violation),
            });
            if self.config.stop_on_first_violation {
                return Ok(ExplorationResult {
                    visited_states: 1,
                    max_depth_reached: 0,
                    violations,
                    traces,
                    graph: StateGraph { nodes, edges },
                });
            }
        }

        visited.insert(initial_state.clone());
        queue.push_back((initial_state, vec![], 0));

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
            for tgen in &self.transition_generators {
                let combinations = self.generate_param_combinations(&tgen.param_values);

                for args in combinations {
                    match self.apply_transition(&tgen.transition_name, &args, &current_state) {
                        Ok(Some(next_state)) => {
                            let next_id = get_or_create_state_id(&next_state, false, &mut state_to_id, &mut nodes);

                            // Record edge
                            if self.config.generate_graph {
                                let label = format!(
                                    "{}({})",
                                    tgen.transition_name,
                                    args.iter()
                                        .map(|v| v.pretty())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                );
                                edges.push(GraphEdge {
                                    from: current_id,
                                    to: next_id,
                                    label,
                                });
                            }

                            // Create trace step
                            let step = TraceStep {
                                transition_name: tgen.transition_name.clone(),
                                args: args.clone(),
                                state_before: current_state.clone(),
                                state_after: next_state.clone(),
                            };

                            let mut new_trace = trace.clone();
                            new_trace.push(step);

                            // Check invariants on new state
                            if let Some(violation) =
                                self.check_invariants(&next_state, new_trace.len())?
                            {
                                nodes[next_id].has_violation = true;
                                violations.push(violation.clone());
                                traces.push(Trace {
                                    id: trace_counter,
                                    steps: new_trace.clone(),
                                    violation: Some(violation),
                                });
                                trace_counter += 1;

                                if self.config.stop_on_first_violation {
                                    return Ok(ExplorationResult {
                                        visited_states: visited.len(),
                                        max_depth_reached,
                                        violations,
                                        traces,
                                        graph: StateGraph { nodes, edges },
                                    });
                                }
                            }

                            // Enqueue if not visited
                            if !visited.contains(&next_state) {
                                visited.insert(next_state.clone());
                                queue.push_back((next_state, new_trace, depth + 1));
                            }
                        }
                        Ok(None) => {
                            // Precondition not satisfied, skip
                        }
                        Err(e) => {
                            // Transition caused runtime error
                            eprintln!(
                                "Warning: Transition {} failed: {}",
                                tgen.transition_name, e
                            );
                        }
                    }
                }
            }
        }

        // Record successful trace if requested
        if self.config.record_all_traces && violations.is_empty() {
            traces.push(Trace {
                id: trace_counter,
                steps: vec![],
                violation: None,
            });
        }

        Ok(ExplorationResult {
            visited_states: visited.len(),
            max_depth_reached,
            violations,
            traces,
            graph: StateGraph { nodes, edges },
        })
    }

    /// Apply a transition to a state, returning the new state if precondition holds
    fn apply_transition(
        &self,
        transition_name: &str,
        args: &[Value],
        state: &Value,
    ) -> Result<Option<Value>> {
        let transition = self
            .spec
            .transitions
            .iter()
            .find(|t| t.name == transition_name)
            .ok_or_else(|| VdlError::UndefinedTransition {
                name: transition_name.to_string(),
            })?;

        let mut eval = Evaluator::new(self.spec);

        // Bind parameters
        eval.env.push_scope();
        for ((param_name, _), arg_val) in transition.params.iter().zip(args) {
            eval.env.bind(param_name.clone(), arg_val.clone());
        }

        // Bind state parameter
        eval.env
            .bind(transition.state_param.clone(), state.clone());

        // Check precondition
        if let Some(pre) = &transition.precondition {
            let pre_result = eval.eval(pre)?;
            if !pre_result.is_truthy() {
                return Ok(None); // Precondition not satisfied
            }
        }

        // For postcondition evaluation, we need to construct the new state
        // The postcondition defines what the new state looks like
        // We'll interpret the postcondition as an expression that produces the new state

        // Store old state for old() references
        eval.old_state = Some(state.clone());

        // Evaluate postcondition to get new state
        // The postcondition should be structured as an expression that evaluates to the new state
        let new_state = self.compute_new_state(&transition.postcondition, &mut eval)?;

        eval.env.pop_scope();

        Ok(Some(new_state))
    }

    /// Compute new state from postcondition
    /// This interprets the postcondition as a state expression
    fn compute_new_state(&self, post: &Expr, eval: &mut Evaluator) -> Result<Value> {
        // The postcondition is expected to be a let-expression that binds the post-state
        // and then an expression that constrains it
        // For now, we'll interpret the postcondition directly as the new state expression

        match post {
            Expr::Let(_var, _val_expr, body) => {
                // The body should contain the actual state expression
                // We expect patterns like: s'.field = expr && ...
                // For simplicity, we'll compute incrementally

                // Actually, let's interpret the postcondition differently:
                // We expect the body to be a conjunction of equalities
                self.extract_state_from_postcondition(body, eval)
            }
            _ => eval.eval(post),
        }
    }

    /// Extract the new state from a postcondition expression
    fn extract_state_from_postcondition(
        &self,
        post: &Expr,
        eval: &mut Evaluator,
    ) -> Result<Value> {
        // Get current state as base
        let state_param = eval
            .env
            .lookup(&eval.spec.transitions[0].state_param)
            .cloned()
            .unwrap_or(Value::Unit);

        match post {
            // Handle s' = expr pattern
            Expr::BinOp(left, BinOp::Eq, right) => {
                // Check if left is a post-state reference
                match left.as_ref() {
                    Expr::PostState(_) | Expr::Var(_) => eval.eval(right),
                    _ => {
                        // Maybe it's a record update pattern
                        self.build_state_from_constraints(post, &state_param, eval)
                    }
                }
            }
            // Handle conjunction of constraints
            Expr::BinOp(left, BinOp::And, _right) => {
                // Start with the left constraint
                self.extract_state_from_postcondition(left, eval)
            }
            // Direct record update
            Expr::RecordUpdate(_, _) => eval.eval(post),
            // Let binding
            Expr::Let(var, val_expr, body) => {
                let val = eval.eval(val_expr)?;
                eval.env.push_scope();
                eval.env.bind(var.clone(), val);
                let result = self.extract_state_from_postcondition(body, eval);
                eval.env.pop_scope();
                result
            }
            _ => eval.eval(post),
        }
    }

    /// Build state from a conjunction of field constraints
    fn build_state_from_constraints(
        &self,
        _post: &Expr,
        base_state: &Value,
        _eval: &mut Evaluator,
    ) -> Result<Value> {
        // For now, return base state
        // This would need more sophisticated pattern matching
        Ok(base_state.clone())
    }

    /// Check all invariants against a state
    fn check_invariants(&self, state: &Value, step_index: usize) -> Result<Option<Violation>> {
        for inv in &self.spec.invariants {
            let mut eval = Evaluator::new(self.spec);
            eval.env.push_scope();
            eval.env.bind(inv.state_param.clone(), state.clone());

            let result = eval.eval(&inv.condition)?;
            eval.env.pop_scope();

            if !result.is_truthy() {
                return Ok(Some(Violation {
                    invariant_name: inv.name.clone(),
                    message: format!("Invariant '{}' violated", inv.name),
                    state: state.clone(),
                    step_index,
                }));
            }
        }
        Ok(None)
    }

    /// Generate all combinations of parameter values
    fn generate_param_combinations(&self, param_values: &[Vec<Value>]) -> Vec<Vec<Value>> {
        if param_values.is_empty() {
            return vec![vec![]];
        }

        let mut combinations = vec![vec![]];
        for values in param_values {
            let mut new_combinations = Vec::new();
            for combo in &combinations {
                for value in values {
                    let mut new_combo = combo.clone();
                    new_combo.push(value.clone());
                    new_combinations.push(new_combo);
                }
            }
            combinations = new_combinations;
        }
        combinations
    }
}

/// Generate DOT format for GraphViz visualization
pub fn generate_dot(graph: &StateGraph) -> String {
    let mut dot = String::new();
    dot.push_str("digraph StateSpace {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box];\n\n");

    // Nodes
    for node in &graph.nodes {
        let style = if node.has_violation {
            "style=filled,fillcolor=red"
        } else if node.is_initial {
            "style=filled,fillcolor=green"
        } else {
            "style=solid"
        };

        let label = format!("State {}", node.id);
        dot.push_str(&format!(
            "  {} [label=\"{}\",{}];\n",
            node.id, label, style
        ));
    }

    dot.push('\n');

    // Edges
    for edge in &graph.edges {
        dot.push_str(&format!(
            "  {} -> {} [label=\"{}\"];\n",
            edge.from, edge.to, edge.label
        ));
    }

    dot.push_str("}\n");
    dot
}
