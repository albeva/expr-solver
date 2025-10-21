//! Error types for parsing, linking, and program operations.

use crate::SymbolError;
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

/// Errors that can occur during IR (bytecode) generation.
#[derive(Error, Debug)]
pub enum IrError {
    #[error("{0}")]
    SymbolError(#[from] SymbolError),
}

/// Errors that can occur during linking.
#[derive(Error, Debug)]
pub enum LinkError {
    #[error("Type mismatch for symbol '{name}': expected {expected}, found {found}")]
    TypeMismatch {
        name: String,
        expected: String,
        found: String,
    },
}

/// Errors that can occur during program operations.
#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("{0}")]
    ParseError(String),

    #[error("{0}")]
    IrError(#[from] IrError),

    #[error("Link error: {0}")]
    LinkError(#[from] LinkError),

    #[error("{0}")]
    VmError(#[from] crate::vm::VmError),

    #[error("{0}")]
    SymError(#[from] SymbolError),

    #[cfg(feature = "serialization")]
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::error::EncodeError),

    #[cfg(feature = "serialization")]
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] bincode::error::DecodeError),

    #[cfg(feature = "serialization")]
    #[error("Incompatible program version: expected {expected}, got {found}")]
    IncompatibleVersion { expected: String, found: String },

    #[cfg(feature = "serialization")]
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
