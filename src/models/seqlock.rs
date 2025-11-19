//! Seqlock model: Writer-optimized reader-writer synchronization
//!
//! Sequence lock protocol from Linux kernel.
//! Writers increment sequence number on entry/exit.
//! Readers check sequence for consistency.

use crate::{ExplorerConfig, Value};
use std::collections::BTreeMap;

/// Seqlock state representation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SeqlockState {
    pub seq: u64,      // Sequence number
    pub data: i64,     // Protected data
    pub writing: bool, // Writer active flag
}

impl SeqlockState {
    pub fn new() -> Self {
        Self {
            seq: 0,
            data: 0,
            writing: false,
        }
    }

    /// Check invariant: seq parity matches writing state
    pub fn check_parity(&self) -> bool {
        self.writing == (self.seq % 2 == 1)
    }
}

/// Create Value from SeqlockState
pub fn make_seqlock_state(seq: u64, data: i64, writing: bool) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert("seq".to_string(), Value::Nat(seq));
    fields.insert("data".to_string(), Value::Int(data));
    fields.insert("writing".to_string(), Value::Bool(writing));
    Value::Record(fields)
}

/// Apply seqlock transition
pub fn apply_seqlock_transition(
    state: &Value,
    transition: &str,
    val: Option<i64>,
) -> Option<Value> {
    if let Value::Record(fields) = state {
        let seq = match fields.get("seq")? {
            Value::Nat(n) => *n,
            _ => return None,
        };
        let data = match fields.get("data")? {
            Value::Int(d) => *d,
            _ => return None,
        };
        let writing = match fields.get("writing")? {
            Value::Bool(w) => *w,
            _ => return None,
        };

        match transition {
            "write_begin" => {
                if !writing {
                    Some(make_seqlock_state(seq + 1, data, true))
                } else {
                    None
                }
            }
            "write_end" => {
                if writing {
                    let new_data = val.unwrap_or(data + 1);
                    Some(make_seqlock_state(seq + 1, new_data, false))
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Check seq_parity invariant on a seqlock state value
pub fn check_seq_parity(state: &Value) -> bool {
    if let Value::Record(fields) = state {
        if let (Some(Value::Nat(seq)), Some(Value::Bool(writing))) =
            (fields.get("seq"), fields.get("writing"))
        {
            return (*writing) == (*seq % 2 == 1);
        }
    }
    false
}

/// Seqlock VDL source (for reference)
pub const SEQLOCK_VDL_SOURCE: &str = include_str!("../../examples/seqlock/basic_seqlock.vdl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seqlock_initial_state() {
        let state = SeqlockState::new();
        assert_eq!(state.seq, 0);
        assert_eq!(state.data, 0);
        assert!(!state.writing);
        assert!(state.check_parity());
    }

    #[test]
    fn test_seqlock_write_begin() {
        let initial = make_seqlock_state(0, 42, false);
        let next = apply_seqlock_transition(&initial, "write_begin", None).unwrap();

        if let Value::Record(fields) = next {
            assert_eq!(fields.get("seq"), Some(&Value::Nat(1)));
            assert_eq!(fields.get("data"), Some(&Value::Int(42)));
            assert_eq!(fields.get("writing"), Some(&Value::Bool(true)));
        } else {
            panic!("Expected Record");
        }
    }

    #[test]
    fn test_seqlock_write_end() {
        let initial = make_seqlock_state(1, 42, true);
        let next = apply_seqlock_transition(&initial, "write_end", Some(100)).unwrap();

        if let Value::Record(fields) = next {
            assert_eq!(fields.get("seq"), Some(&Value::Nat(2)));
            assert_eq!(fields.get("data"), Some(&Value::Int(100)));
            assert_eq!(fields.get("writing"), Some(&Value::Bool(false)));
        } else {
            panic!("Expected Record");
        }
    }

    #[test]
    fn test_seqlock_write_begin_blocked_when_writing() {
        let initial = make_seqlock_state(1, 42, true);
        let next = apply_seqlock_transition(&initial, "write_begin", None);
        assert!(next.is_none(), "write_begin should be blocked when already writing");
    }

    #[test]
    fn test_seqlock_write_end_blocked_when_not_writing() {
        let initial = make_seqlock_state(0, 42, false);
        let next = apply_seqlock_transition(&initial, "write_end", Some(100));
        assert!(next.is_none(), "write_end should be blocked when not writing");
    }

    #[test]
    fn test_seqlock_parity_invariant() {
        // Even seq, not writing -> valid
        let state1 = SeqlockState { seq: 0, data: 0, writing: false };
        assert!(state1.check_parity());

        // Odd seq, writing -> valid
        let state2 = SeqlockState { seq: 1, data: 0, writing: true };
        assert!(state2.check_parity());

        // Even seq, writing -> invalid
        let state3 = SeqlockState { seq: 0, data: 0, writing: true };
        assert!(!state3.check_parity());

        // Odd seq, not writing -> invalid
        let state4 = SeqlockState { seq: 1, data: 0, writing: false };
        assert!(!state4.check_parity());
    }

    #[test]
    fn test_seqlock_invariant_checking() {
        // Test invariant on various states
        let s0 = make_seqlock_state(0, 10, false);
        assert!(check_seq_parity(&s0), "Even seq, not writing should satisfy invariant");

        let s1 = make_seqlock_state(1, 10, true);
        assert!(check_seq_parity(&s1), "Odd seq, writing should satisfy invariant");

        let s2 = make_seqlock_state(2, 20, false);
        assert!(check_seq_parity(&s2), "Even seq after write should satisfy invariant");

        let s3 = make_seqlock_state(0, 10, true);
        assert!(!check_seq_parity(&s3), "Even seq with writing should violate invariant");

        let s4 = make_seqlock_state(1, 10, false);
        assert!(!check_seq_parity(&s4), "Odd seq without writing should violate invariant");
    }

    #[test]
    fn test_seqlock_protocol_sequence() {
        // Test a complete write sequence: idle -> begin -> end -> idle
        let s0 = make_seqlock_state(0, 10, false);

        let s1 = apply_seqlock_transition(&s0, "write_begin", None).unwrap();
        // After write_begin: seq=1 (odd), writing=true
        if let Value::Record(ref fields) = s1 {
            assert_eq!(fields.get("seq"), Some(&Value::Nat(1)));
            assert_eq!(fields.get("writing"), Some(&Value::Bool(true)));
        }

        let s2 = apply_seqlock_transition(&s1, "write_end", Some(20)).unwrap();
        // After write_end: seq=2 (even), writing=false, data=20
        if let Value::Record(ref fields) = s2 {
            assert_eq!(fields.get("seq"), Some(&Value::Nat(2)));
            assert_eq!(fields.get("writing"), Some(&Value::Bool(false)));
            assert_eq!(fields.get("data"), Some(&Value::Int(20)));
        }

        // Can begin again
        let s3 = apply_seqlock_transition(&s2, "write_begin", None).unwrap();
        if let Value::Record(ref fields) = s3 {
            assert_eq!(fields.get("seq"), Some(&Value::Nat(3)));
            assert_eq!(fields.get("writing"), Some(&Value::Bool(true)));
        }
    }
}
