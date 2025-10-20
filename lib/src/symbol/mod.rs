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
#[cfg(feature = "decimal-precision")]
mod decimal;
#[cfg(feature = "f64-floats")]
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
}

/// Symbol table containing constants and functions.
///
/// The table stores mathematical constants like `pi` and functions like `sin`.
/// Symbol lookups are case-insensitive.
///
/// # Examples
///
/// ```
/// use expr_solver::{SymTable, Number, ParseNumber};
///
/// let mut table = SymTable::stdlib();
/// table.add_const("x", Number::parse_number("42").unwrap()).unwrap();
/// ```
#[derive(Debug, Default, Clone)]
pub struct SymTable {
    symbols: Vec<Symbol>,
}

impl SymTable {
    /// Creates an empty symbol table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a constant to the table.
    ///
    /// Returns an error if a symbol with the same name already exists.
    pub fn add_const<S: Into<Cow<'static, str>>>(
        &mut self,
        name: S,
        value: Number,
    ) -> Result<&mut Self, SymbolError> {
        let name = name.into();
        if self.get(&name).is_some() {
            return Err(SymbolError::DuplicateSymbol(name.to_string()));
        }
        self.symbols.push(Symbol::Const {
            name,
            value,
            description: None,
        });
        Ok(self)
    }

    /// Adds a function to the table.
    ///
    /// # Parameters
    /// - `name`: Function name
    /// - `args`: Minimum number of arguments
    /// - `variadic`: Whether the function accepts additional arguments
    /// - `callback`: Function implementation
    ///
    /// Returns an error if a symbol with the same name already exists.
    pub fn add_func<S: Into<Cow<'static, str>>>(
        &mut self,
        name: S,
        args: usize,
        variadic: bool,
        callback: fn(&[Number]) -> Result<Number, FuncError>,
    ) -> Result<&mut Self, SymbolError> {
        let name = name.into();
        if self.get(&name).is_some() {
            return Err(SymbolError::DuplicateSymbol(name.to_string()));
        }
        self.symbols.push(Symbol::Func {
            name,
            args,
            variadic,
            callback,
            description: None,
        });
        Ok(self)
    }

    /// Looks up a symbol by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols
            .iter()
            .find(|sym| sym.name().eq_ignore_ascii_case(name))
    }

    /// Looks up a symbol by name and returns its index and reference (case-insensitive).
    pub fn get_with_index(&self, name: &str) -> Option<(usize, &Symbol)> {
        self.symbols
            .iter()
            .enumerate()
            .find(|(_, sym)| sym.name().eq_ignore_ascii_case(name))
    }

    /// Returns a symbol by index.
    pub fn get_by_index(&self, index: usize) -> Option<&Symbol> {
        self.symbols.get(index)
    }

    /// Returns an iterator over all symbols in the table.
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }
}
