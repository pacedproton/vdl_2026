//! Expression evaluator for VDL++ (denotational + operational semantics)
//!
//! This evaluator interprets VDL++ expressions to produce runtime values.
//! For transitions, it implements small-step operational semantics.

use crate::ast::*;
use crate::error::{Result, VdlError};
use crate::value::{Env, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Evaluator context containing the specification and current environment
pub struct Evaluator<'spec> {
    pub spec: &'spec Specification,
    pub env: Env,
    pub old_state: Option<Value>,
}

impl<'spec> Evaluator<'spec> {
    pub fn new(spec: &'spec Specification) -> Self {
        Self {
            spec,
            env: Env::new(),
            old_state: None,
        }
    }

    /// Evaluate an expression to a value
    pub fn eval(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::NatLit(n) => Ok(Value::Nat(*n)),
            Expr::IntLit(n) => Ok(Value::Int(*n)),
            Expr::BoolLit(b) => Ok(Value::Bool(*b)),

            Expr::SetLit(elements) => {
                let mut set = BTreeSet::new();
                for e in elements {
                    set.insert(self.eval(e)?);
                }
                Ok(Value::Set(set))
            }

            Expr::ListLit(elements) => {
                let mut list = Vec::new();
                for e in elements {
                    list.push(self.eval(e)?);
                }
                Ok(Value::List(list))
            }

            Expr::MapLit(pairs) => {
                let mut map = BTreeMap::new();
                for (k, v) in pairs {
                    map.insert(self.eval(k)?, self.eval(v)?);
                }
                Ok(Value::Map(map))
            }

            Expr::RecordLit(fields) => {
                let mut record = BTreeMap::new();
                for (name, expr) in fields {
                    record.insert(name.clone(), self.eval(expr)?);
                }
                Ok(Value::Record(record))
            }

            Expr::Var(name) => self
                .env
                .lookup(name)
                .cloned()
                .ok_or_else(|| VdlError::UndefinedVariable {
                    name: name.clone(),
                    span: None,
                }),

            Expr::FieldAccess(record_expr, field) => {
                let record = self.eval(record_expr)?;
                match record {
                    Value::Record(fields) => fields
                        .get(field)
                        .cloned()
                        .ok_or_else(|| VdlError::RuntimeError {
                            message: format!("Field '{}' not found in record", field),
                        }),
                    _ => Err(VdlError::RuntimeError {
                        message: "Expected record".to_string(),
                    }),
                }
            }

            Expr::RecordUpdate(record_expr, updates) => {
                let mut record = self.eval(record_expr)?;
                match &mut record {
                    Value::Record(fields) => {
                        for (field_name, value_expr) in updates {
                            let value = self.eval(value_expr)?;
                            fields.insert(field_name.clone(), value);
                        }
                        Ok(record)
                    }
                    _ => Err(VdlError::RuntimeError {
                        message: "Expected record for update".to_string(),
                    }),
                }
            }

            Expr::BinOp(left, op, right) => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                self.eval_binop(&l, *op, &r)
            }

            Expr::UnaryOp(op, expr) => {
                let v = self.eval(expr)?;
                self.eval_unaryop(*op, &v)
            }

            Expr::App(func_expr, args) => {
                let func_val = self.eval(func_expr)?;
                match func_val {
                    Value::Function(name) => {
                        let func_def = self
                            .spec
                            .functions
                            .iter()
                            .find(|f| f.name == name)
                            .ok_or_else(|| VdlError::UndefinedFunction {
                                name: name.clone(),
                                span: None,
                            })?
                            .clone();

                        if args.len() != func_def.params.len() {
                            return Err(VdlError::RuntimeError {
                                message: format!(
                                    "Function {} expects {} arguments, got {}",
                                    name,
                                    func_def.params.len(),
                                    args.len()
                                ),
                            });
                        }

                        self.env.push_scope();
                        for ((param_name, _), arg_expr) in func_def.params.iter().zip(args) {
                            let arg_val = self.eval(arg_expr)?;
                            self.env.bind(param_name.clone(), arg_val);
                        }

                        let result = self.eval(&func_def.body);
                        self.env.pop_scope();
                        result
                    }
                    _ => Err(VdlError::RuntimeError {
                        message: "Expected function".to_string(),
                    }),
                }
            }

            Expr::Let(name, value_expr, body_expr) => {
                let value = self.eval(value_expr)?;
                self.env.push_scope();
                self.env.bind(name.clone(), value);
                let result = self.eval(body_expr);
                self.env.pop_scope();
                result
            }

            Expr::If(cond, then_branch, else_branch) => {
                let cond_val = self.eval(cond)?;
                if cond_val.is_truthy() {
                    self.eval(then_branch)
                } else {
                    self.eval(else_branch)
                }
            }

            Expr::SetMember(elem, set) => {
                let elem_val = self.eval(elem)?;
                let set_val = self.eval(set)?;
                match set_val {
                    Value::Set(s) => Ok(Value::Bool(s.contains(&elem_val))),
                    _ => Err(VdlError::SetError {
                        message: "Expected set".to_string(),
                    }),
                }
            }

            Expr::SetInsert(set_expr, elem_expr) => {
                let set_val = self.eval(set_expr)?;
                let elem_val = self.eval(elem_expr)?;
                match set_val {
                    Value::Set(mut s) => {
                        s.insert(elem_val);
                        Ok(Value::Set(s))
                    }
                    _ => Err(VdlError::SetError {
                        message: "Expected set".to_string(),
                    }),
                }
            }

            Expr::SetRemove(set_expr, elem_expr) => {
                let set_val = self.eval(set_expr)?;
                let elem_val = self.eval(elem_expr)?;
                match set_val {
                    Value::Set(mut s) => {
                        s.remove(&elem_val);
                        Ok(Value::Set(s))
                    }
                    _ => Err(VdlError::SetError {
                        message: "Expected set".to_string(),
                    }),
                }
            }

            Expr::SetUnion(left, right) => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                match (l, r) {
                    (Value::Set(mut s1), Value::Set(s2)) => {
                        s1.extend(s2);
                        Ok(Value::Set(s1))
                    }
                    _ => Err(VdlError::SetError {
                        message: "Expected sets".to_string(),
                    }),
                }
            }

            Expr::SetIntersect(left, right) => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                match (l, r) {
                    (Value::Set(s1), Value::Set(s2)) => {
                        let intersection: BTreeSet<Value> =
                            s1.intersection(&s2).cloned().collect();
                        Ok(Value::Set(intersection))
                    }
                    _ => Err(VdlError::SetError {
                        message: "Expected sets".to_string(),
                    }),
                }
            }

            Expr::SetDiff(left, right) => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                match (l, r) {
                    (Value::Set(s1), Value::Set(s2)) => {
                        let diff: BTreeSet<Value> = s1.difference(&s2).cloned().collect();
                        Ok(Value::Set(diff))
                    }
                    _ => Err(VdlError::SetError {
                        message: "Expected sets".to_string(),
                    }),
                }
            }

            Expr::SetEmpty(set_expr) => {
                let set_val = self.eval(set_expr)?;
                match set_val {
                    Value::Set(s) => Ok(Value::Bool(s.is_empty())),
                    Value::List(l) => Ok(Value::Bool(l.is_empty())),
                    _ => Err(VdlError::RuntimeError {
                        message: "Expected set or list".to_string(),
                    }),
                }
            }

            Expr::SetSize(set_expr) => {
                let set_val = self.eval(set_expr)?;
                match set_val {
                    Value::Set(s) => Ok(Value::Nat(s.len() as u64)),
                    _ => Err(VdlError::SetError {
                        message: "Expected set".to_string(),
                    }),
                }
            }

            Expr::ListAppend(left, right) => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                match (l, r) {
                    (Value::List(mut l1), Value::List(l2)) => {
                        l1.extend(l2);
                        Ok(Value::List(l1))
                    }
                    _ => Err(VdlError::ListError {
                        message: "Expected lists".to_string(),
                    }),
                }
            }

            Expr::ListHead(list_expr) => {
                let list_val = self.eval(list_expr)?;
                match list_val {
                    Value::List(l) => l.first().cloned().ok_or_else(|| VdlError::ListError {
                        message: "Empty list".to_string(),
                    }),
                    _ => Err(VdlError::ListError {
                        message: "Expected list".to_string(),
                    }),
                }
            }

            Expr::ListTail(list_expr) => {
                let list_val = self.eval(list_expr)?;
                match list_val {
                    Value::List(l) => {
                        if l.is_empty() {
                            Err(VdlError::ListError {
                                message: "Empty list".to_string(),
                            })
                        } else {
                            Ok(Value::List(l[1..].to_vec()))
                        }
                    }
                    _ => Err(VdlError::ListError {
                        message: "Expected list".to_string(),
                    }),
                }
            }

            Expr::ListLength(list_expr) => {
                let list_val = self.eval(list_expr)?;
                match list_val {
                    Value::List(l) => Ok(Value::Nat(l.len() as u64)),
                    _ => Err(VdlError::ListError {
                        message: "Expected list".to_string(),
                    }),
                }
            }

            Expr::ListEmpty(list_expr) => {
                let list_val = self.eval(list_expr)?;
                match list_val {
                    Value::List(l) => Ok(Value::Bool(l.is_empty())),
                    _ => Err(VdlError::ListError {
                        message: "Expected list".to_string(),
                    }),
                }
            }

            Expr::ListCons(elem_expr, list_expr) => {
                let elem_val = self.eval(elem_expr)?;
                let list_val = self.eval(list_expr)?;
                match list_val {
                    Value::List(mut l) => {
                        l.insert(0, elem_val);
                        Ok(Value::List(l))
                    }
                    _ => Err(VdlError::ListError {
                        message: "Expected list".to_string(),
                    }),
                }
            }

            Expr::ListFilter(list_expr, var, pred) => {
                let list_val = self.eval(list_expr)?;
                match list_val {
                    Value::List(l) => {
                        let mut result = Vec::new();
                        for elem in l {
                            self.env.push_scope();
                            self.env.bind(var.clone(), elem.clone());
                            let keep = self.eval(pred)?;
                            self.env.pop_scope();
                            if keep.is_truthy() {
                                result.push(elem);
                            }
                        }
                        Ok(Value::List(result))
                    }
                    _ => Err(VdlError::ListError {
                        message: "Expected list".to_string(),
                    }),
                }
            }

            Expr::Forall(var, domain_expr, pred) => {
                let domain = self.eval(domain_expr)?;
                let elements: Vec<Value> = match domain {
                    Value::Set(s) => s.into_iter().collect(),
                    Value::List(l) => l,
                    _ => {
                        return Err(VdlError::RuntimeError {
                            message: "Expected set or list for quantifier".to_string(),
                        })
                    }
                };

                for elem in elements {
                    self.env.push_scope();
                    self.env.bind(var.clone(), elem);
                    let result = self.eval(pred)?;
                    self.env.pop_scope();
                    if !result.is_truthy() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }

            Expr::Exists(var, domain_expr, pred) => {
                let domain = self.eval(domain_expr)?;
                let elements: Vec<Value> = match domain {
                    Value::Set(s) => s.into_iter().collect(),
                    Value::List(l) => l,
                    _ => {
                        return Err(VdlError::RuntimeError {
                            message: "Expected set or list for quantifier".to_string(),
                        })
                    }
                };

                for elem in elements {
                    self.env.push_scope();
                    self.env.bind(var.clone(), elem);
                    let result = self.eval(pred)?;
                    self.env.pop_scope();
                    if result.is_truthy() {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }

            Expr::Implies(left, right) => {
                let l = self.eval(left)?;
                if !l.is_truthy() {
                    // False implies anything
                    Ok(Value::Bool(true))
                } else {
                    self.eval(right)
                }
            }

            Expr::Old(expr) => {
                if let Some(old) = &self.old_state {
                    let saved_env = self.env.clone();
                    self.env.push_scope();
                    self.env.bind("__old_state__".to_string(), old.clone());
                    // Replace state variable with old state
                    let result = self.eval(expr);
                    self.env = saved_env;
                    result
                } else {
                    Err(VdlError::RuntimeError {
                        message: "old() used outside of postcondition".to_string(),
                    })
                }
            }

            Expr::PostState(_name) => {
                // This should be bound in the evaluation context
                self.env
                    .lookup("__new_state__")
                    .cloned()
                    .ok_or_else(|| VdlError::RuntimeError {
                        message: "Post-state not available".to_string(),
                    })
            }
        }
    }

    fn eval_binop(&self, left: &Value, op: BinOp, right: &Value) -> Result<Value> {
        match op {
            BinOp::Add => match (left, right) {
                (Value::Nat(a), Value::Nat(b)) => Ok(Value::Nat(a + b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for addition".to_string(),
                    span: None,
                }),
            },
            BinOp::Sub => match (left, right) {
                (Value::Nat(a), Value::Nat(b)) => {
                    if a >= b {
                        Ok(Value::Nat(a - b))
                    } else {
                        Ok(Value::Int(*a as i64 - *b as i64))
                    }
                }
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for subtraction".to_string(),
                    span: None,
                }),
            },
            BinOp::Mul => match (left, right) {
                (Value::Nat(a), Value::Nat(b)) => Ok(Value::Nat(a * b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for multiplication".to_string(),
                    span: None,
                }),
            },
            BinOp::Div => match (left, right) {
                (Value::Nat(a), Value::Nat(b)) => {
                    if *b == 0 {
                        Err(VdlError::DivisionByZero)
                    } else {
                        Ok(Value::Nat(a / b))
                    }
                }
                (Value::Int(a), Value::Int(b)) => {
                    if *b == 0 {
                        Err(VdlError::DivisionByZero)
                    } else {
                        Ok(Value::Int(a / b))
                    }
                }
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for division".to_string(),
                    span: None,
                }),
            },
            BinOp::Mod => match (left, right) {
                (Value::Nat(a), Value::Nat(b)) => {
                    if *b == 0 {
                        Err(VdlError::DivisionByZero)
                    } else {
                        Ok(Value::Nat(a % b))
                    }
                }
                (Value::Int(a), Value::Int(b)) => {
                    if *b == 0 {
                        Err(VdlError::DivisionByZero)
                    } else {
                        Ok(Value::Int(a % b))
                    }
                }
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for modulo".to_string(),
                    span: None,
                }),
            },
            BinOp::Eq => Ok(Value::Bool(left == right)),
            BinOp::Ne => Ok(Value::Bool(left != right)),
            BinOp::Lt => match (left, right) {
                (Value::Nat(a), Value::Nat(b)) => Ok(Value::Bool(a < b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for comparison".to_string(),
                    span: None,
                }),
            },
            BinOp::Le => match (left, right) {
                (Value::Nat(a), Value::Nat(b)) => Ok(Value::Bool(a <= b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for comparison".to_string(),
                    span: None,
                }),
            },
            BinOp::Gt => match (left, right) {
                (Value::Nat(a), Value::Nat(b)) => Ok(Value::Bool(a > b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for comparison".to_string(),
                    span: None,
                }),
            },
            BinOp::Ge => match (left, right) {
                (Value::Nat(a), Value::Nat(b)) => Ok(Value::Bool(a >= b)),
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for comparison".to_string(),
                    span: None,
                }),
            },
            BinOp::And => match (left, right) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for logical and".to_string(),
                    span: None,
                }),
            },
            BinOp::Or => match (left, right) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
                _ => Err(VdlError::TypeError {
                    message: "Type mismatch for logical or".to_string(),
                    span: None,
                }),
            },
        }
    }

    fn eval_unaryop(&self, op: UnaryOp, val: &Value) -> Result<Value> {
        match op {
            UnaryOp::Not => match val {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(VdlError::TypeError {
                    message: "Expected boolean for negation".to_string(),
                    span: None,
                }),
            },
            UnaryOp::Neg => match val {
                Value::Nat(n) => Ok(Value::Int(-(*n as i64))),
                Value::Int(n) => Ok(Value::Int(-n)),
                _ => Err(VdlError::TypeError {
                    message: "Expected number for negation".to_string(),
                    span: None,
                }),
            },
        }
    }
}
