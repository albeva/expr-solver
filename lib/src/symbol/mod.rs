//! Symbol table for constants and functions.
//!
//! This module provides symbol table functionality with type-specific implementations
//! for different numeric backends:
//!
//! - `f64.rs` - Standard f64 floating-point with relaxed error handling
//! - `decimal.rs` - High-precision 128-bit Decimal with strict validation
//!
//! The appropriate implementation is selected at compile-time via feature flags.

use crate::number::Number;
use std::borrow::Cow;
use thiserror::Error;

// Type-specific implementations
#[cfg(feature = "decimal")]
mod decimal;
#[cfg(not(feature = "decimal"))]
mod f64;

/// Errors that can occur during function evaluation.
#[derive(Error, Debug, Clone)]
pub enum FuncError {
    #[error("Conversion error: failed to convert number to f64")]
    ToF64Conversion,
    #[error("Conversion error: failed to convert f64 result back to number")]
    FromF64Conversion,
    #[error("Square root of negative number: {value}")]
    NegativeSqrt { value: Number },
    #[error("Domain error in function '{function}': invalid input {input}")]
    DomainError { function: String, input: Number },
    #[error("Math error: {message}")]
    MathError { message: String },
}

/// Errors that can occur during symbol table operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SymbolError {
    /// A symbol with this name already exists in the table.
    #[error("Duplicate symbol definition: '{0}'")]
    DuplicateSymbol(String),

    /// Fired when no symbol with given name exists
    #[error("Symbol '{0}' not found")]
    SymbolNotFound(String),
}

/// A symbol representing either a constant or function.
///
/// Symbols are stored in a [`SymTable`] and referenced during evaluation.
#[derive(Debug, Clone)]
pub enum Symbol {
    /// Named constant (e.g., `pi`).
    Const {
        name: Cow<'static, str>,
        value: Number,
        description: Option<Cow<'static, str>>,
        /// Whether this is a local constant (from let) or global (from stdlib)
        local: bool,
    },
    /// Function with specified arity and callback.
    Func {
        name: Cow<'static, str>,
        /// Minimum number of arguments
        args: usize,
        /// Whether the function accepts additional arguments
        variadic: bool,
        callback: fn(&[Number]) -> Result<Number, FuncError>,
        description: Option<Cow<'static, str>>,
        /// Whether this is a local function (reserved for future use) or global (from stdlib)
        local: bool,
        /// Func offset (local only)
        offset: Option<usize>,
    },
}

impl Symbol {
    /// Returns the name of the symbol.
    pub fn name(&self) -> &str {
        match self {
            Symbol::Const { name, .. } => name,
            Symbol::Func { name, .. } => name,
        }
    }

    /// Returns the description of the symbol, if available.
    pub fn description(&self) -> Option<&str> {
        match self {
            Symbol::Const { description, .. } => description.as_deref(),
            Symbol::Func { description, .. } => description.as_deref(),
        }
    }

    /// Returns whether this symbol is local (from let) or global (from stdlib).
    pub fn is_local(&self) -> bool {
        match self {
            Symbol::Const { local, .. } => *local,
            Symbol::Func { local, .. } => *local,
        }
    }
}
