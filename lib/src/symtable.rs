//! Symbol table implementation.

use crate::number::Number;
use crate::symbol::{FuncError, Symbol, SymbolError};
use std::borrow::Cow;

/// Symbol table containing constants and functions.
///
/// The table stores mathematical constants like `pi` and functions like `sin`.
/// Symbol lookups are case-insensitive.
///
/// # Examples
///
/// ```
/// use expr_solver::{num, SymTable};
///
/// let mut table = SymTable::stdlib();
/// table.add_const("x", num!(42)).unwrap();
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

    /// Creates a symbol table from a vector of symbols.
    /// This is used internally by the type-specific stdlib implementations.
    pub(crate) fn from_symbols(symbols: Vec<Symbol>) -> Self {
        Self { symbols }
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
