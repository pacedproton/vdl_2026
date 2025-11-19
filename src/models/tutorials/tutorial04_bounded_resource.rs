//! Tutorial 4: Bounded Resource with Invariants
//!
//! Demonstrates invariant checking:
//! - Type invariants
//! - State invariants
//! - Invariant violation detection
//! - Resource bounds

use crate::Value;
use std::collections::BTreeMap;

/// Resource pool state
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourcePool {
    pub allocated: u64,
    pub available: u64,
    pub total: u64,
    pub requests: u64,
}

impl ResourcePool {
    pub fn new(total: u64) -> Self {
        Self {
            allocated: 0,
            available: total,
            total,
            requests: 0,
        }
    }

    /// Check invariant: total == allocated + available
    pub fn check_total_correct(&self) -> bool {
        self.total == self.allocated + self.available
    }

    /// Check invariant: allocated <= total
    pub fn check_no_overallocation(&self) -> bool {
        self.allocated <= self.total
    }

    /// Check invariant: available <= total
    pub fn check_available_valid(&self) -> bool {
        self.available <= self.total
    }

    /// Check invariant: total <= 100
    pub fn check_capacity_bounded(&self) -> bool {
        self.total <= 100
    }

    /// Check all invariants
    pub fn check_all_invariants(&self) -> bool {
        self.check_total_correct()
            && self.check_no_overallocation()
            && self.check_available_valid()
            && self.check_capacity_bounded()
    }
}

/// Convert ResourcePool to Value
pub fn make_resource_pool(allocated: u64, available: u64, total: u64, requests: u64) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert("allocated".to_string(), Value::Nat(allocated));
    fields.insert("available".to_string(), Value::Nat(available));
    fields.insert("total".to_string(), Value::Nat(total));
    fields.insert("requests".to_string(), Value::Nat(requests));
    Value::Record(fields)
}

/// Extract ResourcePool from Value
pub fn extract_resource_pool(value: &Value) -> Option<ResourcePool> {
    if let Value::Record(fields) = value {
        let allocated = fields.get("allocated")?.as_nat()?;
        let available = fields.get("available")?.as_nat()?;
        let total = fields.get("total")?.as_nat()?;
        let requests = fields.get("requests")?.as_nat()?;
        Some(ResourcePool {
            allocated,
            available,
            total,
            requests,
        })
    } else {
        None
    }
}

/// Apply resource pool transition
pub fn apply_resource_transition(state: &Value, transition: &str, amount: u64) -> Option<Value> {
    let s = extract_resource_pool(state)?;

    match transition {
        "allocate" => {
            // pre: amount > 0 and amount <= s.available
            if amount == 0 || amount > s.available {
                return None;
            }
            // post: s'.allocated == s.allocated + amount and
            //       s'.available == s.available - amount
            Some(make_resource_pool(
                s.allocated + amount,
                s.available - amount,
                s.total,
                s.requests + 1,
            ))
        }

        "release" => {
            // pre: amount > 0 and amount <= s.allocated
            if amount == 0 || amount > s.allocated {
                return None;
            }
            // post: s'.allocated == s.allocated - amount and
            //       s'.available == s.available + amount
            Some(make_resource_pool(
                s.allocated - amount,
                s.available + amount,
                s.total,
                s.requests,
            ))
        }

        "expand" => {
            // pre: amount > 0 and s.total + amount <= 100
            if amount == 0 || s.total + amount > 100 {
                return None;
            }
            // post: s'.total == s.total + amount and
            //       s'.available == s.available + amount
            Some(make_resource_pool(
                s.allocated,
                s.available + amount,
                s.total + amount,
                s.requests,
            ))
        }

        "shrink" => {
            // pre: amount > 0 and amount <= s.available
            if amount == 0 || amount > s.available {
                return None;
            }
            // post: s'.total == s.total - amount and
            //       s'.available == s.available - amount
            Some(make_resource_pool(
                s.allocated,
                s.available - amount,
                s.total - amount,
                s.requests,
            ))
        }

        _ => None,
    }
}

/// Check invariants on a resource pool value
pub fn check_invariants(value: &Value) -> Vec<String> {
    let mut violations = Vec::new();

    if let Some(pool) = extract_resource_pool(value) {
        if !pool.check_total_correct() {
            violations.push(format!(
                "total_correct violated: {} != {} + {}",
                pool.total, pool.allocated, pool.available
            ));
        }
        if !pool.check_no_overallocation() {
            violations.push(format!(
                "no_overallocation violated: {} > {}",
                pool.allocated, pool.total
            ));
        }
        if !pool.check_available_valid() {
            violations.push(format!(
                "available_valid violated: {} > {}",
                pool.available, pool.total
            ));
        }
        if !pool.check_capacity_bounded() {
            violations.push(format!(
                "capacity_bounded violated: {} > 100",
                pool.total
            ));
        }
    }

    violations
}

pub const TUTORIAL_04_VDL: &str =
    include_str!("../../../examples/tutorials/04_bounded_resource.vdl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let pool = ResourcePool::new(50);
        assert_eq!(pool.allocated, 0);
        assert_eq!(pool.available, 50);
        assert_eq!(pool.total, 50);
        assert_eq!(pool.requests, 0);
        assert!(pool.check_all_invariants());
    }

    #[test]
    fn test_allocate() {
        let s0 = make_resource_pool(0, 50, 50, 0);
        let s1 = apply_resource_transition(&s0, "allocate", 10).unwrap();

        let pool = extract_resource_pool(&s1).unwrap();
        assert_eq!(pool.allocated, 10);
        assert_eq!(pool.available, 40);
        assert_eq!(pool.total, 50);
        assert_eq!(pool.requests, 1);
        assert!(pool.check_all_invariants());
    }

    #[test]
    fn test_allocate_blocked_insufficient() {
        let s0 = make_resource_pool(0, 50, 50, 0);
        let result = apply_resource_transition(&s0, "allocate", 60);
        assert!(result.is_none(), "allocate should be blocked when amount > available");
    }

    #[test]
    fn test_allocate_zero_blocked() {
        let s0 = make_resource_pool(0, 50, 50, 0);
        let result = apply_resource_transition(&s0, "allocate", 0);
        assert!(result.is_none(), "allocate(0) should be blocked");
    }

    #[test]
    fn test_release() {
        let s0 = make_resource_pool(30, 20, 50, 5);
        let s1 = apply_resource_transition(&s0, "release", 10).unwrap();

        let pool = extract_resource_pool(&s1).unwrap();
        assert_eq!(pool.allocated, 20);
        assert_eq!(pool.available, 30);
        assert_eq!(pool.total, 50);
        assert!(pool.check_all_invariants());
    }

    #[test]
    fn test_release_blocked_excessive() {
        let s0 = make_resource_pool(20, 30, 50, 0);
        let result = apply_resource_transition(&s0, "release", 30);
        assert!(result.is_none(), "release should be blocked when amount > allocated");
    }

    #[test]
    fn test_expand() {
        let s0 = make_resource_pool(30, 20, 50, 0);
        let s1 = apply_resource_transition(&s0, "expand", 20).unwrap();

        let pool = extract_resource_pool(&s1).unwrap();
        assert_eq!(pool.allocated, 30);
        assert_eq!(pool.available, 40);
        assert_eq!(pool.total, 70);
        assert!(pool.check_all_invariants());
    }

    #[test]
    fn test_expand_blocked_over_limit() {
        let s0 = make_resource_pool(0, 50, 50, 0);
        let result = apply_resource_transition(&s0, "expand", 60);
        assert!(result.is_none(), "expand should be blocked when total would exceed 100");
    }

    #[test]
    fn test_shrink() {
        let s0 = make_resource_pool(30, 40, 70, 0);
        let s1 = apply_resource_transition(&s0, "shrink", 20).unwrap();

        let pool = extract_resource_pool(&s1).unwrap();
        assert_eq!(pool.allocated, 30);
        assert_eq!(pool.available, 20);
        assert_eq!(pool.total, 50);
        assert!(pool.check_all_invariants());
    }

    #[test]
    fn test_shrink_blocked_over_available() {
        let s0 = make_resource_pool(40, 10, 50, 0);
        let result = apply_resource_transition(&s0, "shrink", 20);
        assert!(result.is_none(), "shrink should be blocked when amount > available");
    }

    #[test]
    fn test_complete_scenario() {
        // Start with 50 resources
        let s0 = make_resource_pool(0, 50, 50, 0);
        assert!(extract_resource_pool(&s0).unwrap().check_all_invariants());

        // Allocate 30
        let s1 = apply_resource_transition(&s0, "allocate", 30).unwrap();
        assert!(extract_resource_pool(&s1).unwrap().check_all_invariants());

        // Try allocate 30 more (should fail - only 20 available)
        assert!(apply_resource_transition(&s1, "allocate", 30).is_none());

        // Allocate remaining 20
        let s2 = apply_resource_transition(&s1, "allocate", 20).unwrap();
        assert!(extract_resource_pool(&s2).unwrap().check_all_invariants());
        assert_eq!(extract_resource_pool(&s2).unwrap().available, 0);

        // Try allocate when full (should fail)
        assert!(apply_resource_transition(&s2, "allocate", 1).is_none());

        // Release 10
        let s3 = apply_resource_transition(&s2, "release", 10).unwrap();
        assert!(extract_resource_pool(&s3).unwrap().check_all_invariants());

        // Now can allocate 5
        let s4 = apply_resource_transition(&s3, "allocate", 5).unwrap();
        assert!(extract_resource_pool(&s4).unwrap().check_all_invariants());
    }

    #[test]
    fn test_invariant_checking() {
        // Valid state
        let s0 = make_resource_pool(30, 20, 50, 0);
        let violations = check_invariants(&s0);
        assert!(violations.is_empty(), "No violations expected for valid state");

        // Invalid state: total != allocated + available
        let s1 = make_resource_pool(30, 20, 60, 0);
        let violations = check_invariants(&s1);
        assert!(!violations.is_empty(), "Should detect total_correct violation");
        assert!(violations[0].contains("total_correct"));

        // Invalid state: total > 100
        let s2 = make_resource_pool(0, 150, 150, 0);
        let violations = check_invariants(&s2);
        assert!(!violations.is_empty(), "Should detect capacity_bounded violation");
        assert!(violations.iter().any(|v| v.contains("capacity_bounded")));
    }

    #[test]
    fn test_all_transitions_preserve_invariants() {
        let initial_states = vec![
            make_resource_pool(0, 50, 50, 0),
            make_resource_pool(25, 25, 50, 5),
            make_resource_pool(40, 10, 50, 10),
        ];

        for s0 in initial_states {
            // Try all valid transitions
            if let Some(s1) = apply_resource_transition(&s0, "allocate", 5) {
                assert!(check_invariants(&s1).is_empty(), "allocate should preserve invariants");
            }
            if let Some(s1) = apply_resource_transition(&s0, "release", 5) {
                assert!(check_invariants(&s1).is_empty(), "release should preserve invariants");
            }
            if let Some(s1) = apply_resource_transition(&s0, "expand", 10) {
                assert!(check_invariants(&s1).is_empty(), "expand should preserve invariants");
            }
            if let Some(s1) = apply_resource_transition(&s0, "shrink", 5) {
                assert!(check_invariants(&s1).is_empty(), "shrink should preserve invariants");
            }
        }
    }
}
