//! Runtime values for VDL++ (operational semantics)
//!
//! Values are the concrete instantiation of denotational domains at runtime.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};

/// Runtime value in VDL++
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Value {
    Nat(u64),
    Int(i64),
    Bool(bool),
    Set(BTreeSet<Value>),
    List(Vec<Value>),
    Map(BTreeMap<Value, Value>),
    Record(BTreeMap<String, Value>),
    Unit,
    Function(String), // Reference to named function
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Nat(n) => n.hash(state),
            Value::Int(n) => n.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Set(s) => {
                for v in s {
                    v.hash(state);
                }
            }
            Value::List(l) => {
                for v in l {
                    v.hash(state);
                }
            }
            Value::Map(m) => {
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::Record(r) => {
                for (k, v) in r {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::Unit => {}
            Value::Function(name) => name.hash(state),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => false,
        }
    }

    pub fn as_nat(&self) -> Option<u64> {
        match self {
            Value::Nat(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_set(&self) -> Option<&BTreeSet<Value>> {
        match self {
            Value::Set(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_set_mut(&mut self) -> Option<&mut BTreeSet<Value>> {
        match self {
            Value::Set(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_record(&self) -> Option<&BTreeMap<String, Value>> {
        match self {
            Value::Record(r) => Some(r),
            _ => None,
        }
    }

    pub fn as_record_mut(&mut self) -> Option<&mut BTreeMap<String, Value>> {
        match self {
            Value::Record(r) => Some(r),
            _ => None,
        }
    }

    /// Pretty print the value
    pub fn pretty(&self) -> String {
        match self {
            Value::Nat(n) => n.to_string(),
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Set(s) => {
                let elements: Vec<String> = s.iter().map(|v| v.pretty()).collect();
                format!("{{{}}}", elements.join(", "))
            }
            Value::List(l) => {
                let elements: Vec<String> = l.iter().map(|v| v.pretty()).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Map(m) => {
                let pairs: Vec<String> = m
                    .iter()
                    .map(|(k, v)| format!("{} -> {}", k.pretty(), v.pretty()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Value::Record(r) => {
                let fields: Vec<String> = r
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, v.pretty()))
                    .collect();
                format!("{{ {} }}", fields.join(", "))
            }
            Value::Unit => "()".to_string(),
            Value::Function(name) => format!("<function {}>", name),
        }
    }
}

/// Environment for variable bindings during evaluation
#[derive(Debug, Clone)]
pub struct Env {
    bindings: Vec<BTreeMap<String, Value>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            bindings: vec![BTreeMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.bindings.push(BTreeMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.bindings.len() > 1 {
            self.bindings.pop();
        }
    }

    pub fn bind(&mut self, name: String, value: Value) {
        if let Some(scope) = self.bindings.last_mut() {
            scope.insert(name, value);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Value> {
        for scope in self.bindings.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        None
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
