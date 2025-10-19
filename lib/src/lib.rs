//! A simple expression solver library
//!
//! Parses and evaluates mathematical expressions with built-in functions and constants.
//!
//! # Features
//!
//! - Mathematical operators: `+`, `-`, `*`, `/`, `^`, unary `-`, `!` (factorial)
//! - Comparison operators: `==`, `!=`, `<`, `<=`, `>`, `>=` (return 1.0 or 0.0)
//! - Built-in constants: `pi`, `e`, `tau`, `ln2`, `ln10`, `sqrt2`
//! - Basic math functions: `abs`, `floor`, `ceil`, `round`, `trunc`, `fract`
//! - Variadic functions: `min`, `max`, `sum`, `avg`
//! - 128-bit decimal arithmetic (no floating-point representation errors!)
//! - Error handling with source location information

// Core types (shared)
mod ir;
mod span;
mod symbol;
mod token;
mod vm;

// V2 implementation
pub mod v2;

use std::{fmt, path::PathBuf};

use crate::span::SpanError;
use rust_decimal::Decimal;

// Public API
pub use symbol::{SymTable, Symbol, SymbolError};
pub use v2::Source;
pub use vm::{Vm, VmError};

/// A wrapper that formats errors with source code highlighting
struct FormattedError {
    message: String,
}

impl fmt::Display for FormattedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<T: SpanError> From<(&T, &Source)> for FormattedError {
    fn from((error, source): (&T, &Source)) -> Self {
        Self {
            message: format!("{}\n{}", error, source.highlight(&error.span())),
        }
    }
}

/// Expression evaluator - simplified wrapper around Program.
///
/// `Eval` provides convenient methods for quickly creating compiled or linked programs.
///
/// # Examples
///
/// ```
/// use expr_solver::Eval;
///
/// // Quick evaluation
/// let result = Eval::evaluate("2 + 3 * 4").unwrap();
/// assert_eq!(result.to_string(), "14");
/// ```
pub struct Eval;

impl Eval {
    /// Quick evaluation of an expression with the standard library.
    ///
    /// This is a convenience method for one-off evaluations.
    ///
    /// # Examples
    ///
    /// ```
    /// use expr_solver::Eval;
    ///
    /// let result = Eval::evaluate("2^8").unwrap();
    /// assert_eq!(result.to_string(), "256");
    /// ```
    pub fn evaluate(expression: &str) -> Result<Decimal, String> {
        let program = Self::new(expression)?;
        program.execute().map_err(|err| err.to_string())
    }

    /// Quick evaluation of an expression with a custom symbol table.
    ///
    /// # Examples
    ///
    /// ```
    /// use expr_solver::{Eval, SymTable};
    /// use rust_decimal_macros::dec;
    ///
    /// let mut table = SymTable::stdlib();
    /// table.add_const("x", dec!(42)).unwrap();
    ///
    /// let result = Eval::evaluate_with_table("x * 2", table).unwrap();
    /// assert_eq!(result, dec!(84));
    /// ```
    pub fn evaluate_with_table(expression: &str, table: SymTable) -> Result<Decimal, String> {
        let program = Self::with_table(expression, table)?;
        program.execute().map_err(|err| err.to_string())
    }

    /// Creates a linked program with the standard library.
    ///
    /// Returns a `Program<Linked>` ready to execute.
    ///
    /// # Examples
    ///
    /// ```
    /// use expr_solver::Eval;
    ///
    /// let program = Eval::new("sin(pi/2)").unwrap();
    /// let result = program.execute().unwrap();
    /// ```
    pub fn new(expression: &str) -> Result<v2::Program<v2::Linked>, String> {
        Self::with_table(expression, SymTable::stdlib())
    }

    /// Creates a linked program with a custom symbol table.
    ///
    /// Returns a `Program<Linked>` ready to execute.
    ///
    /// # Examples
    ///
    /// ```
    /// use expr_solver::{Eval, SymTable};
    /// use rust_decimal_macros::dec;
    ///
    /// let mut table = SymTable::stdlib();
    /// table.add_const("x", dec!(42)).unwrap();
    ///
    /// let program = Eval::with_table("x * 2", table).unwrap();
    /// let result = program.execute().unwrap();
    /// assert_eq!(result, dec!(84));
    /// ```
    pub fn with_table(
        expression: &str,
        table: SymTable,
    ) -> Result<v2::Program<v2::Linked>, String> {
        let source = Source::new(expression);

        // Parse and compile
        let program = v2::Program::new_from_source(source.clone()).map_err(|err| {
            // Extract ParseError from ProgramError for nice formatting
            match err {
                v2::ProgramError::ParseError(parse_err) => {
                    FormattedError::from((&parse_err, &source)).to_string()
                }
                other => other.to_string(),
            }
        })?;

        // Link
        program.link(table).map_err(|err| err.to_string())
    }

    /// Creates a compiled program from a binary file.
    ///
    /// Returns a `Program<Compiled>` that can be linked with a symbol table.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use expr_solver::{Eval, SymTable};
    /// use std::path::PathBuf;
    ///
    /// let program = Eval::new_from_file(PathBuf::from("expr.bin")).unwrap();
    /// let linked = program.link(SymTable::stdlib()).unwrap();
    /// let result = linked.execute().unwrap();
    /// ```
    pub fn new_from_file(path: PathBuf) -> Result<v2::Program<v2::Compiled>, String> {
        v2::Program::new_from_file(path.to_string_lossy().to_string())
            .map_err(|err| err.to_string())
    }

    /// Creates a linked program from a binary file with a custom symbol table.
    ///
    /// Returns a `Program<Linked>` ready to execute.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use expr_solver::{Eval, SymTable};
    /// use std::path::PathBuf;
    ///
    /// let program = Eval::from_file_with_table(
    ///     PathBuf::from("expr.bin"),
    ///     SymTable::stdlib()
    /// ).unwrap();
    /// let result = program.execute().unwrap();
    /// ```
    pub fn from_file_with_table(
        path: PathBuf,
        table: SymTable,
    ) -> Result<v2::Program<v2::Linked>, String> {
        let program = Self::new_from_file(path)?;
        program.link(table).map_err(|err| err.to_string())
    }
}
