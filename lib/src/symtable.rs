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
/// table.add_const("x", num!(42), false).unwrap();
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
    ///
    /// # Parameters
    /// - `name`: Constant name
    /// - `value`: Constant value
    /// - `local`: Whether this is a local constant (from let) or global (from stdlib)
    pub fn add_const<S: Into<Cow<'static, str>>>(
        &mut self,
        name: S,
        value: Number,
        local: bool,
    ) -> Result<&mut Self, SymbolError> {
        let name = name.into();
        if self.get_by_name(&name).is_ok() {
            return Err(SymbolError::DuplicateSymbol(name.to_string()));
        }
        self.symbols.push(Symbol::Const {
            name,
            value,
            description: None,
            local,
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
    /// - `local`: Whether this is a local function (reserved for future) or global (from stdlib)
    ///
    /// Returns an error if a symbol with the same name already exists.
    pub fn add_func<S: Into<Cow<'static, str>>>(
        &mut self,
        name: S,
        args: usize,
        variadic: bool,
        callback: fn(&[Number]) -> Result<Number, FuncError>,
        local: bool,
    ) -> Result<&mut Self, SymbolError> {
        let name = name.into();
        if self.get_by_name(&name).is_ok() {
            return Err(SymbolError::DuplicateSymbol(name.to_string()));
        }
        self.symbols.push(Symbol::Func {
            name,
            args,
            variadic,
            callback,
            description: None,
            local,
        });
        Ok(self)
    }

    /// Looks up a symbol by name (case-insensitive).
    pub fn get_by_name(&self, name: &str) -> Result<&Symbol, SymbolError> {
        self.symbols
            .iter()
            .find(|sym| sym.name().eq_ignore_ascii_case(name))
            .ok_or_else(|| SymbolError::SymbolNotFound(name.to_string()))
    }

    /// Looks up a symbol by name and returns its index and reference (case-insensitive).
    pub fn get_with_index(&self, name: &str) -> Result<(usize, &Symbol), SymbolError> {
        self.symbols
            .iter()
            .enumerate()
            .find(|(_, sym)| sym.name().eq_ignore_ascii_case(name))
            .ok_or_else(|| SymbolError::SymbolNotFound(name.to_string()))
    }

    /// Returns a symbol by index.
    pub fn get_by_index(&self, index: usize) -> Result<&Symbol, SymbolError> {
        self.symbols.get(index).ok_or_else(|| SymbolError::SymbolNotFound(index.to_string()))
    }

    /// Returns a symbol by index.
    pub fn get_mut_by_index(&mut self, index: usize) -> Result<&mut Symbol, SymbolError> {
        self.symbols.get_mut(index).ok_or_else(|| SymbolError::SymbolNotFound(index.to_string()))
    }

    /// Returns an iterator over all symbols in the table.
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }
}
