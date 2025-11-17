//! Error handling for VDL++

use thiserror::Error;
use crate::ast::Span;

/// VDL++ Error type
#[derive(Debug, Error)]
pub enum VdlError {
    #[error("Lexer error at position {position}: {message}")]
    LexerError { position: usize, message: String },

    #[error("Parse error at position {position}: {message}")]
    ParseError { position: usize, message: String },

    #[error("Type error: {message}")]
    TypeError { message: String, span: Option<Span> },

    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String, span: Option<Span> },

    #[error("Undefined type: {name}")]
    UndefinedType { name: String, span: Option<Span> },

    #[error("Undefined function: {name}")]
    UndefinedFunction { name: String, span: Option<Span> },

    #[error("Undefined transition: {name}")]
    UndefinedTransition { name: String },

    #[error("Invariant violation: {message}")]
    InvariantViolation { message: String },

    #[error("Precondition failed for transition {transition}: {message}")]
    PreconditionFailed { transition: String, message: String },

    #[error("Runtime error: {message}")]
    RuntimeError { message: String },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Set operation error: {message}")]
    SetError { message: String },

    #[error("List operation error: {message}")]
    ListError { message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for VDL++ operations
pub type Result<T> = std::result::Result<T, VdlError>;
