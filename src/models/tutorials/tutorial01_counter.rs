//! Tutorial 1: Basic Counter
//!
//! Demonstrates basic VDL_2026 syntax:
//! - Simple state with Nat and Bool types
//! - Transitions with pre/post conditions
//! - Basic arithmetic operations

use crate::Value;
use std::collections::BTreeMap;

/// Counter state representation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CounterState {
    pub count: u64,
    pub running: bool,
}

impl CounterState {
    pub fn new() -> Self {
        Self {
            count: 0,
            running: false,
        }
    }

    pub fn with_values(count: u64, running: bool) -> Self {
        Self { count, running }
    }
}

/// Convert CounterState to Value
pub fn make_counter_state(count: u64, running: bool) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert("count".to_string(), Value::Nat(count));
    fields.insert("running".to_string(), Value::Bool(running));
    Value::Record(fields)
}

/// Extract CounterState from Value
pub fn extract_counter_state(value: &Value) -> Option<CounterState> {
    if let Value::Record(fields) = value {
        let count = fields.get("count")?.as_nat()?;
        let running = fields.get("running")?.as_bool()?;
        Some(CounterState { count, running })
    } else {
        None
    }
}

/// Apply counter transition
pub fn apply_counter_transition(state: &Value, transition: &str) -> Option<Value> {
    let s = extract_counter_state(state)?;

    match transition {
        "increment" => {
            // pre: s.running == true
            if !s.running {
                return None;
            }
            // post: s'.count == s.count + 1 and s'.running == s.running
            Some(make_counter_state(s.count + 1, s.running))
        }

        "decrement" => {
            // pre: s.running == true and s.count > 0
            if !s.running || s.count == 0 {
                return None;
            }
            // post: s'.count == s.count - 1 and s'.running == s.running
            Some(make_counter_state(s.count - 1, s.running))
        }

        "reset" => {
            // pre: true
            // post: s'.count == 0 and s'.running == s.running
            Some(make_counter_state(0, s.running))
        }

        "start" => {
            // pre: s.running == false
            if s.running {
                return None;
            }
            // post: s'.count == s.count and s'.running == true
            Some(make_counter_state(s.count, true))
        }

        "stop" => {
            // pre: s.running == true
            if !s.running {
                return None;
            }
            // post: s'.count == s.count and s'.running == false
            Some(make_counter_state(s.count, false))
        }

        _ => None,
    }
}

/// VDL source for reference
pub const TUTORIAL_01_VDL: &str = include_str!("../../../examples/tutorials/01_basic_counter.vdl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let s = CounterState::new();
        assert_eq!(s.count, 0);
        assert_eq!(s.running, false);
    }

    #[test]
    fn test_start_transition() {
        let s0 = make_counter_state(0, false);
        let s1 = apply_counter_transition(&s0, "start").unwrap();

        let result = extract_counter_state(&s1).unwrap();
        assert_eq!(result.count, 0);
        assert_eq!(result.running, true);
    }

    #[test]
    fn test_start_blocked_when_running() {
        let s0 = make_counter_state(5, true);
        let result = apply_counter_transition(&s0, "start");
        assert!(result.is_none(), "start should be blocked when already running");
    }

    #[test]
    fn test_increment_transition() {
        let s0 = make_counter_state(0, true);
        let s1 = apply_counter_transition(&s0, "increment").unwrap();

        let result = extract_counter_state(&s1).unwrap();
        assert_eq!(result.count, 1);
        assert_eq!(result.running, true);
    }

    #[test]
    fn test_increment_blocked_when_stopped() {
        let s0 = make_counter_state(5, false);
        let result = apply_counter_transition(&s0, "increment");
        assert!(result.is_none(), "increment should be blocked when not running");
    }

    #[test]
    fn test_decrement_transition() {
        let s0 = make_counter_state(5, true);
        let s1 = apply_counter_transition(&s0, "decrement").unwrap();

        let result = extract_counter_state(&s1).unwrap();
        assert_eq!(result.count, 4);
        assert_eq!(result.running, true);
    }

    #[test]
    fn test_decrement_blocked_at_zero() {
        let s0 = make_counter_state(0, true);
        let result = apply_counter_transition(&s0, "decrement");
        assert!(result.is_none(), "decrement should be blocked at count=0");
    }

    #[test]
    fn test_decrement_blocked_when_stopped() {
        let s0 = make_counter_state(5, false);
        let result = apply_counter_transition(&s0, "decrement");
        assert!(result.is_none(), "decrement should be blocked when not running");
    }

    #[test]
    fn test_reset_transition() {
        let s0 = make_counter_state(42, true);
        let s1 = apply_counter_transition(&s0, "reset").unwrap();

        let result = extract_counter_state(&s1).unwrap();
        assert_eq!(result.count, 0);
        assert_eq!(result.running, true);
    }

    #[test]
    fn test_stop_transition() {
        let s0 = make_counter_state(10, true);
        let s1 = apply_counter_transition(&s0, "stop").unwrap();

        let result = extract_counter_state(&s1).unwrap();
        assert_eq!(result.count, 10);
        assert_eq!(result.running, false);
    }

    #[test]
    fn test_stop_blocked_when_not_running() {
        let s0 = make_counter_state(5, false);
        let result = apply_counter_transition(&s0, "stop");
        assert!(result.is_none(), "stop should be blocked when not running");
    }

    #[test]
    fn test_complete_sequence() {
        // Scenario: start -> increment x3 -> decrement -> stop -> reset
        let s0 = make_counter_state(0, false);

        // start
        let s1 = apply_counter_transition(&s0, "start").unwrap();
        assert_eq!(extract_counter_state(&s1).unwrap(), CounterState { count: 0, running: true });

        // increment
        let s2 = apply_counter_transition(&s1, "increment").unwrap();
        assert_eq!(extract_counter_state(&s2).unwrap(), CounterState { count: 1, running: true });

        // increment
        let s3 = apply_counter_transition(&s2, "increment").unwrap();
        assert_eq!(extract_counter_state(&s3).unwrap(), CounterState { count: 2, running: true });

        // increment
        let s4 = apply_counter_transition(&s3, "increment").unwrap();
        assert_eq!(extract_counter_state(&s4).unwrap(), CounterState { count: 3, running: true });

        // decrement
        let s5 = apply_counter_transition(&s4, "decrement").unwrap();
        assert_eq!(extract_counter_state(&s5).unwrap(), CounterState { count: 2, running: true });

        // stop
        let s6 = apply_counter_transition(&s5, "stop").unwrap();
        assert_eq!(extract_counter_state(&s6).unwrap(), CounterState { count: 2, running: false });

        // reset (works even when stopped)
        let s7 = apply_counter_transition(&s6, "reset").unwrap();
        assert_eq!(extract_counter_state(&s7).unwrap(), CounterState { count: 0, running: false });
    }

    #[test]
    fn test_invalid_operations_when_stopped() {
        let s = make_counter_state(5, false);

        assert!(apply_counter_transition(&s, "increment").is_none());
        assert!(apply_counter_transition(&s, "decrement").is_none());
        assert!(apply_counter_transition(&s, "start").is_some());
        assert!(apply_counter_transition(&s, "reset").is_some());
    }
}
