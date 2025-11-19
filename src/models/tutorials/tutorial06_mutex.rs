//! Tutorial 6: Mutual Exclusion Protocol
//!
//! Demonstrates:
//! - Multi-process synchronization
//! - Mutual exclusion invariants
//! - Process ownership and waiting queues

use crate::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Mutex state
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MutexState {
    pub locked: bool,
    pub holder: u64,  // 0 = none
    pub waiting: BTreeSet<u64>,
    pub in_critical: BTreeSet<u64>,
}

impl MutexState {
    pub fn new() -> Self {
        Self {
            locked: false,
            holder: 0,
            waiting: BTreeSet::new(),
            in_critical: BTreeSet::new(),
        }
    }

    /// SAFETY: At most one process in critical section
    pub fn check_mutual_exclusion(&self) -> bool {
        self.in_critical.len() <= 1
    }

    /// SAFETY: Lock consistency
    pub fn check_lock_consistency(&self) -> bool {
        self.locked == (self.holder != 0 && self.in_critical.len() == 1)
    }

    /// SAFETY: Holder is in critical section
    pub fn check_holder_in_critical(&self) -> bool {
        !self.locked || self.in_critical.contains(&self.holder)
    }

    /// SAFETY: Waiting and critical are disjoint
    pub fn check_waiting_disjoint_critical(&self) -> bool {
        self.waiting.intersection(&self.in_critical).next().is_none()
    }

    /// Check all invariants
    pub fn check_all_invariants(&self) -> bool {
        self.check_mutual_exclusion()
            && self.check_lock_consistency()
            && self.check_holder_in_critical()
            && self.check_waiting_disjoint_critical()
    }
}

/// Convert MutexState to Value
pub fn make_mutex_state(
    locked: bool,
    holder: u64,
    waiting: BTreeSet<u64>,
    in_critical: BTreeSet<u64>,
) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert("locked".to_string(), Value::Bool(locked));
    fields.insert("holder".to_string(), Value::Nat(holder));
    fields.insert("waiting".to_string(), Value::Set(waiting.into_iter().map(Value::Nat).collect()));
    fields.insert("in_critical".to_string(), Value::Set(in_critical.into_iter().map(Value::Nat).collect()));
    Value::Record(fields)
}

/// Extract set of Nat from Value
fn extract_nat_set(value: &Value) -> Option<BTreeSet<u64>> {
    if let Value::Set(set) = value {
        let mut result = BTreeSet::new();
        for v in set {
            result.insert(v.as_nat()?);
        }
        Some(result)
    } else {
        None
    }
}

/// Extract MutexState from Value
pub fn extract_mutex_state(value: &Value) -> Option<MutexState> {
    if let Value::Record(fields) = value {
        let locked = fields.get("locked")?.as_bool()?;
        let holder = fields.get("holder")?.as_nat()?;
        let waiting = extract_nat_set(fields.get("waiting")?)?;
        let in_critical = extract_nat_set(fields.get("in_critical")?)?;
        Some(MutexState {
            locked,
            holder,
            waiting,
            in_critical,
        })
    } else {
        None
    }
}

/// Apply mutex transition
pub fn apply_mutex_transition(state: &Value, transition: &str, pid: u64) -> Option<Value> {
    let s = extract_mutex_state(state)?;

    match transition {
        "lock_request" => {
            // pre: pid > 0 and pid < 100 and
            //      pid not_in waiting and pid not_in in_critical
            if pid == 0 || pid >= 100 || s.waiting.contains(&pid) || s.in_critical.contains(&pid) {
                return None;
            }

            if s.locked {
                // Lock held: join waiting queue
                let mut new_waiting = s.waiting.clone();
                new_waiting.insert(pid);
                Some(make_mutex_state(
                    s.locked,
                    s.holder,
                    new_waiting,
                    s.in_critical,
                ))
            } else {
                // Lock free: acquire immediately
                let mut new_in_critical = BTreeSet::new();
                new_in_critical.insert(pid);
                Some(make_mutex_state(true, pid, s.waiting, new_in_critical))
            }
        }

        "lock_acquire" => {
            // pre: not s.locked and pid in s.waiting
            if s.locked || !s.waiting.contains(&pid) {
                return None;
            }

            let mut new_waiting = s.waiting.clone();
            new_waiting.remove(&pid);
            let mut new_in_critical = BTreeSet::new();
            new_in_critical.insert(pid);
            Some(make_mutex_state(true, pid, new_waiting, new_in_critical))
        }

        "unlock" => {
            // pre: s.locked and s.holder == pid and pid in s.in_critical
            if !s.locked || s.holder != pid || !s.in_critical.contains(&pid) {
                return None;
            }

            Some(make_mutex_state(false, 0, s.waiting, BTreeSet::new()))
        }

        "cancel_wait" => {
            // pre: pid in s.waiting
            if !s.waiting.contains(&pid) {
                return None;
            }

            let mut new_waiting = s.waiting.clone();
            new_waiting.remove(&pid);
            Some(make_mutex_state(s.locked, s.holder, new_waiting, s.in_critical))
        }

        _ => None,
    }
}

/// Check invariants on mutex state
pub fn check_invariants(value: &Value) -> Vec<String> {
    let mut violations = Vec::new();

    if let Some(mutex) = extract_mutex_state(value) {
        if !mutex.check_mutual_exclusion() {
            violations.push(format!(
                "mutual_exclusion violated: {} processes in critical section",
                mutex.in_critical.len()
            ));
        }
        if !mutex.check_lock_consistency() {
            violations.push(format!(
                "lock_consistency violated: locked={}, holder={}, in_critical={}",
                mutex.locked,
                mutex.holder,
                mutex.in_critical.len()
            ));
        }
        if !mutex.check_holder_in_critical() {
            violations.push(format!(
                "holder_in_critical violated: holder {} not in critical section",
                mutex.holder
            ));
        }
        if !mutex.check_waiting_disjoint_critical() {
            violations.push("waiting_disjoint_critical violated: overlap between waiting and critical".to_string());
        }
    }

    violations
}

pub const TUTORIAL_06_VDL: &str = include_str!("../../../examples/tutorials/06_mutex.vdl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let mutex = MutexState::new();
        assert!(!mutex.locked);
        assert_eq!(mutex.holder, 0);
        assert!(mutex.waiting.is_empty());
        assert!(mutex.in_critical.is_empty());
        assert!(mutex.check_all_invariants());
    }

    #[test]
    fn test_lock_request_when_free() {
        let s0 = make_mutex_state(false, 0, BTreeSet::new(), BTreeSet::new());
        let s1 = apply_mutex_transition(&s0, "lock_request", 1).unwrap();

        let mutex = extract_mutex_state(&s1).unwrap();
        assert!(mutex.locked);
        assert_eq!(mutex.holder, 1);
        assert!(mutex.waiting.is_empty());
        assert_eq!(mutex.in_critical, vec![1].into_iter().collect());
        assert!(mutex.check_all_invariants());
    }

    #[test]
    fn test_lock_request_when_held() {
        let mut in_crit = BTreeSet::new();
        in_crit.insert(1);
        let s0 = make_mutex_state(true, 1, BTreeSet::new(), in_crit);
        let s1 = apply_mutex_transition(&s0, "lock_request", 2).unwrap();

        let mutex = extract_mutex_state(&s1).unwrap();
        assert!(mutex.locked);
        assert_eq!(mutex.holder, 1);
        assert_eq!(mutex.waiting, vec![2].into_iter().collect());
        assert_eq!(mutex.in_critical, vec![1].into_iter().collect());
        assert!(mutex.check_all_invariants());
    }

    #[test]
    fn test_unlock() {
        let mut in_crit = BTreeSet::new();
        in_crit.insert(1);
        let s0 = make_mutex_state(true, 1, BTreeSet::new(), in_crit);
        let s1 = apply_mutex_transition(&s0, "unlock", 1).unwrap();

        let mutex = extract_mutex_state(&s1).unwrap();
        assert!(!mutex.locked);
        assert_eq!(mutex.holder, 0);
        assert!(mutex.waiting.is_empty());
        assert!(mutex.in_critical.is_empty());
        assert!(mutex.check_all_invariants());
    }

    #[test]
    fn test_unlock_wrong_process_blocked() {
        let mut in_crit = BTreeSet::new();
        in_crit.insert(1);
        let s0 = make_mutex_state(true, 1, BTreeSet::new(), in_crit);
        let result = apply_mutex_transition(&s0, "unlock", 2);
        assert!(result.is_none(), "unlock by non-holder should be blocked");
    }

    #[test]
    fn test_lock_acquire_from_waiting() {
        let mut waiting = BTreeSet::new();
        waiting.insert(2);
        let s0 = make_mutex_state(false, 0, waiting, BTreeSet::new());
        let s1 = apply_mutex_transition(&s0, "lock_acquire", 2).unwrap();

        let mutex = extract_mutex_state(&s1).unwrap();
        assert!(mutex.locked);
        assert_eq!(mutex.holder, 2);
        assert!(mutex.waiting.is_empty());
        assert_eq!(mutex.in_critical, vec![2].into_iter().collect());
        assert!(mutex.check_all_invariants());
    }

    #[test]
    fn test_lock_acquire_when_locked_blocked() {
        let mut waiting = BTreeSet::new();
        waiting.insert(2);
        let mut in_crit = BTreeSet::new();
        in_crit.insert(1);
        let s0 = make_mutex_state(true, 1, waiting, in_crit);
        let result = apply_mutex_transition(&s0, "lock_acquire", 2);
        assert!(result.is_none(), "lock_acquire should be blocked when mutex is locked");
    }

    #[test]
    fn test_cancel_wait() {
        let mut waiting = BTreeSet::new();
        waiting.insert(2);
        waiting.insert(3);
        let mut in_crit = BTreeSet::new();
        in_crit.insert(1);
        let s0 = make_mutex_state(true, 1, waiting, in_crit.clone());
        let s1 = apply_mutex_transition(&s0, "cancel_wait", 2).unwrap();

        let mutex = extract_mutex_state(&s1).unwrap();
        assert!(mutex.locked);
        assert_eq!(mutex.holder, 1);
        assert_eq!(mutex.waiting, vec![3].into_iter().collect());
        assert_eq!(mutex.in_critical, in_crit);
        assert!(mutex.check_all_invariants());
    }

    #[test]
    fn test_three_process_scenario() {
        // Process 1 acquires lock
        let s0 = make_mutex_state(false, 0, BTreeSet::new(), BTreeSet::new());
        let s1 = apply_mutex_transition(&s0, "lock_request", 1).unwrap();
        assert!(extract_mutex_state(&s1).unwrap().check_all_invariants());

        // Process 2 requests (goes to waiting)
        let s2 = apply_mutex_transition(&s1, "lock_request", 2).unwrap();
        assert!(extract_mutex_state(&s2).unwrap().check_all_invariants());
        assert_eq!(extract_mutex_state(&s2).unwrap().waiting.len(), 1);

        // Process 3 requests (also goes to waiting)
        let s3 = apply_mutex_transition(&s2, "lock_request", 3).unwrap();
        assert!(extract_mutex_state(&s3).unwrap().check_all_invariants());
        assert_eq!(extract_mutex_state(&s3).unwrap().waiting.len(), 2);

        // Process 1 unlocks
        let s4 = apply_mutex_transition(&s3, "unlock", 1).unwrap();
        assert!(extract_mutex_state(&s4).unwrap().check_all_invariants());
        assert!(!extract_mutex_state(&s4).unwrap().locked);

        // Process 2 acquires
        let s5 = apply_mutex_transition(&s4, "lock_acquire", 2).unwrap();
        assert!(extract_mutex_state(&s5).unwrap().check_all_invariants());
        assert_eq!(extract_mutex_state(&s5).unwrap().holder, 2);
        assert_eq!(extract_mutex_state(&s5).unwrap().waiting.len(), 1);

        // Process 2 unlocks
        let s6 = apply_mutex_transition(&s5, "unlock", 2).unwrap();
        assert!(extract_mutex_state(&s6).unwrap().check_all_invariants());

        // Process 3 acquires
        let s7 = apply_mutex_transition(&s6, "lock_acquire", 3).unwrap();
        assert!(extract_mutex_state(&s7).unwrap().check_all_invariants());
        assert_eq!(extract_mutex_state(&s7).unwrap().holder, 3);
        assert!(extract_mutex_state(&s7).unwrap().waiting.is_empty());
    }

    #[test]
    fn test_mutual_exclusion_never_violated() {
        // Try to violate mutual exclusion (should be impossible)
        let s0 = make_mutex_state(false, 0, BTreeSet::new(), BTreeSet::new());

        // Process 1 acquires
        let s1 = apply_mutex_transition(&s0, "lock_request", 1).unwrap();
        assert_eq!(extract_mutex_state(&s1).unwrap().in_critical.len(), 1);

        // Process 2 tries to request - should go to waiting, not critical
        let s2 = apply_mutex_transition(&s1, "lock_request", 2).unwrap();
        assert_eq!(extract_mutex_state(&s2).unwrap().in_critical.len(), 1);
        assert!(extract_mutex_state(&s2).unwrap().in_critical.contains(&1));
        assert!(!extract_mutex_state(&s2).unwrap().in_critical.contains(&2));

        // Can't unlock as process 2
        assert!(apply_mutex_transition(&s2, "unlock", 2).is_none());

        // Can't acquire while locked
        assert!(apply_mutex_transition(&s2, "lock_acquire", 2).is_none());
    }

    #[test]
    fn test_invariant_checking() {
        // Valid state
        let s0 = make_mutex_state(false, 0, BTreeSet::new(), BTreeSet::new());
        assert!(check_invariants(&s0).is_empty());

        // Valid state with lock held
        let mut in_crit = BTreeSet::new();
        in_crit.insert(1);
        let s1 = make_mutex_state(true, 1, BTreeSet::new(), in_crit);
        assert!(check_invariants(&s1).is_empty());

        // Invalid: two processes in critical section
        let mut bad_in_crit = BTreeSet::new();
        bad_in_crit.insert(1);
        bad_in_crit.insert(2);
        let s2 = make_mutex_state(true, 1, BTreeSet::new(), bad_in_crit);
        let violations = check_invariants(&s2);
        assert!(!violations.is_empty());
        assert!(violations[0].contains("mutual_exclusion"));
    }
}
