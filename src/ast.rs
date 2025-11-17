//! Abstract Syntax Tree for VDL++
//!
//! VDL++ uses a hybrid semantics approach:
//! - Denotational semantics for types, expressions, and invariants
//! - Operational semantics for state transitions (small-step)
//!
//! This design honors the meta-IV tradition of the Vienna Development Method.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Source location for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Identifier (variable, type, or function name)
pub type Ident = String;

/// A complete VDL++ specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specification {
    pub type_defs: Vec<TypeDef>,
    pub functions: Vec<FunctionDef>,
    pub transitions: Vec<TransitionDef>,
    pub invariants: Vec<InvariantDef>,
}

impl Specification {
    pub fn new() -> Self {
        Self {
            type_defs: Vec::new(),
            functions: Vec::new(),
            transitions: Vec::new(),
            invariants: Vec::new(),
        }
    }
}

/// Type definition (denotational domain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: Ident,
    pub ty: Type,
    pub invariant: Option<Expr>,
    pub span: Option<Span>,
}

/// Types in VDL++ (denotational semantics - domains)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Natural numbers (non-negative integers)
    Nat,
    /// Integers
    Int,
    /// Booleans
    Bool,
    /// Set type: Set<T>
    Set(Box<Type>),
    /// List/Sequence type: List<T>
    List(Box<Type>),
    /// Map type: Map<K, V>
    Map(Box<Type>, Box<Type>),
    /// Record type: { field1: T1, field2: T2, ... }
    Record(BTreeMap<Ident, Type>),
    /// Named type reference
    Named(Ident),
    /// Function type: T1 -> T2
    Function(Box<Type>, Box<Type>),
    /// Unit type (empty product)
    Unit,
}

/// Function definition (denotational - mathematical function)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: Ident,
    pub params: Vec<(Ident, Type)>,
    pub return_type: Type,
    pub body: Expr,
    pub span: Option<Span>,
}

/// Transition definition (operational semantics - state transformer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDef {
    pub name: Ident,
    pub params: Vec<(Ident, Type)>,
    pub state_param: Ident,
    pub state_type: Type,
    pub precondition: Option<Expr>,
    pub postcondition: Expr,
    pub span: Option<Span>,
}

/// Global invariant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantDef {
    pub name: Ident,
    pub state_param: Ident,
    pub state_type: Type,
    pub condition: Expr,
    pub span: Option<Span>,
}

/// Expressions in VDL++ (denotational semantics)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    // Literals
    NatLit(u64),
    IntLit(i64),
    BoolLit(bool),

    // Collections
    SetLit(Vec<Expr>),
    ListLit(Vec<Expr>),
    MapLit(Vec<(Expr, Expr)>),
    RecordLit(BTreeMap<Ident, Expr>),

    // Variable reference
    Var(Ident),

    // Record field access
    FieldAccess(Box<Expr>, Ident),

    // Record update: s with [ field = value ]
    RecordUpdate(Box<Expr>, Vec<(Ident, Expr)>),

    // Binary operations
    BinOp(Box<Expr>, BinOp, Box<Expr>),

    // Unary operations
    UnaryOp(UnaryOp, Box<Expr>),

    // Function application
    App(Box<Expr>, Vec<Expr>),

    // Let binding: let x = e1 in e2
    Let(Ident, Box<Expr>, Box<Expr>),

    // Conditional: if e1 then e2 else e3
    If(Box<Expr>, Box<Expr>, Box<Expr>),

    // Set operations
    SetMember(Box<Expr>, Box<Expr>),        // x in s
    SetInsert(Box<Expr>, Box<Expr>),        // s union {x}
    SetRemove(Box<Expr>, Box<Expr>),        // s \ {x}
    SetUnion(Box<Expr>, Box<Expr>),         // s1 union s2
    SetIntersect(Box<Expr>, Box<Expr>),     // s1 intersect s2
    SetDiff(Box<Expr>, Box<Expr>),          // s1 \ s2
    SetEmpty(Box<Expr>),                    // isEmpty(s)
    SetSize(Box<Expr>),                     // |s|

    // List operations
    ListAppend(Box<Expr>, Box<Expr>),       // xs ++ ys
    ListHead(Box<Expr>),                    // head(xs)
    ListTail(Box<Expr>),                    // tail(xs)
    ListLength(Box<Expr>),                  // length(xs)
    ListEmpty(Box<Expr>),                   // isEmpty(xs)
    ListCons(Box<Expr>, Box<Expr>),         // x :: xs
    ListFilter(Box<Expr>, Ident, Box<Expr>), // filter(xs, x => pred)

    // Quantifiers (for invariants and pre/post conditions)
    Forall(Ident, Box<Expr>, Box<Expr>),    // forall x in s: pred
    Exists(Ident, Box<Expr>, Box<Expr>),    // exists x in s: pred

    // Implication (common in formal specs)
    Implies(Box<Expr>, Box<Expr>),          // p -> q

    // Old state reference (for postconditions)
    Old(Box<Expr>),

    // New/Post state variable
    PostState(Ident),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Neg,
}

impl Default for Specification {
    fn default() -> Self {
        Self::new()
    }
}
