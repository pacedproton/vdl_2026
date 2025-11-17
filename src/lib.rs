//! VDL_2026 - Vienna Definition Language 2026
//!
//! IMPLICIT NONE for kernel concurrency.
//!
//! A modern successor to the Vienna Definition Language for formal
//! specification and state-space exploration.
//!
//! VDL_2026 combines:
//! - Denotational semantics for types, expressions, and invariants (meta-IV heritage)
//! - Operational semantics for state transitions (small-step semantics)
//! - IMPLICIT NONE philosophy: no hidden state, no implicit assumptions
//!
//! This implementation honors the Vienna Lab tradition while bringing
//! executable formal specifications to modern systems, particularly
//! for modeling concurrent protocols like Linux kernel's RCU.
//!
//! Like Fortran's IMPLICIT NONE, every transition is explicit,
//! every invariant is visible, every assumption is checkable.

pub mod ast;
pub mod error;
pub mod eval;
pub mod explorer;
pub mod lexer;
pub mod models;
pub mod parser;
pub mod value;

// Re-exports for convenience
pub use ast::{Specification, TransitionDef, Type, TypeDef};
pub use error::{Result, VdlError};
pub use eval::Evaluator;
pub use explorer::{
    generate_dot, ExplorationResult, Explorer, ExplorerConfig, StateGraph, Trace,
    TransitionGenerator, Violation,
};
pub use parser::parse;
pub use value::{Env, Value};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse and explore a VDL++ specification
pub fn parse_and_explore(
    source: &str,
    initial_state: Value,
    generators: Vec<TransitionGenerator>,
    config: ExplorerConfig,
) -> Result<ExplorationResult> {
    let spec = parse(source)?;
    let mut explorer = Explorer::new(&spec, config);

    for generator in generators {
        explorer.add_transition_generator(generator);
    }

    explorer.explore(initial_state)
}
