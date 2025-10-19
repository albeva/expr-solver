//! Error types for v2 implementation.

use crate::span::Span;
use crate::span::SpanError;
use thiserror::Error;

/// Errors that can occur during parsing.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: {message}")]
    UnexpectedToken { message: String, span: Span },
    #[error("Unexpected end of input")]
    UnexpectedEof { span: Span },
    #[error("Invalid number literal: {message}")]
    InvalidNumber { message: String, span: Span },
}

impl SpanError for ParseError {
    fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => *span,
            ParseError::UnexpectedEof { span } => *span,
            ParseError::InvalidNumber { span, .. } => *span,
        }
    }
}

/// Errors that can occur during compilation.
#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Semantic error: {0}")]
    SemanticError(#[from] SemanticError),
    #[error("Code generation error: {0}")]
    CodeGenError(String),
}

/// Errors that can occur during semantic analysis.
#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("Undefined symbol: '{name}'")]
    UndefinedSymbol { name: String, span: Span },
    #[error("Symbol '{name}' is not a constant")]
    SymbolIsNotAConstant { name: String, span: Span },
    #[error("Symbol '{name}' is not a function")]
    SymbolIsNotAFunction { name: String, span: Span },
    #[error("Function '{name}' expects {expected} arguments, got {actual}")]
    ArgumentCountMismatch {
        name: String,
        expected: usize,
        actual: usize,
        span: Span,
    },
    #[error("Function '{name}' expects at least {expected} arguments, got {actual}")]
    InsufficientArguments {
        name: String,
        expected: usize,
        actual: usize,
        span: Span,
    },
}

impl SpanError for SemanticError {
    fn span(&self) -> Span {
        match self {
            SemanticError::UndefinedSymbol { span, .. } => *span,
            SemanticError::SymbolIsNotAConstant { span, .. } => *span,
            SemanticError::SymbolIsNotAFunction { span, .. } => *span,
            SemanticError::ArgumentCountMismatch { span, .. } => *span,
            SemanticError::InsufficientArguments { span, .. } => *span,
        }
    }
}

/// Errors that can occur during linking.
#[derive(Error, Debug)]
pub enum LinkError {
    #[error("Missing symbol: '{name}' is required by bytecode but not in symbol table")]
    MissingSymbol { name: String },

    #[error("Type mismatch for symbol '{name}': expected {expected}, found {found}")]
    TypeMismatch {
        name: String,
        expected: String,
        found: String,
    },

    #[error("Symbol table error: {0}")]
    SymbolTableError(#[from] crate::symbol::SymbolError),
}

/// Errors that can occur during program operations.
#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Compile error: {0}")]
    CompileError(#[from] CompileError),

    #[error("Link error: {0}")]
    LinkError(#[from] LinkError),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Incompatible program version: expected {expected}, got {found}")]
    IncompatibleVersion { expected: String, found: String },

    #[error("Invalid symbol index: {0}")]
    InvalidSymbolIndex(usize),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
