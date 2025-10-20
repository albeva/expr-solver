//! Error types for parsing, linking, and program operations.

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
    #[error("{0}")]
    ParseError(String),

    #[error("Link error: {0}")]
    LinkError(#[from] LinkError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::error::EncodeError),

    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] bincode::error::DecodeError),

    #[error("Incompatible program version: expected {expected}, got {found}")]
    IncompatibleVersion { expected: String, found: String },

    #[error("Invalid symbol index: {0}")]
    InvalidSymbolIndex(usize),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
