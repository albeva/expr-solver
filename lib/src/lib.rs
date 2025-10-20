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

// Expression solver implementation
mod ast;
mod error;
mod lexer;
mod metadata;
mod parser;
mod program;

use rust_decimal::Decimal;

// Public API
pub use ast::{BinOp, Expr, ExprKind, UnOp};
pub use error::{LinkError, ParseError, ProgramError};
pub use metadata::{SymbolKind, SymbolMetadata};
pub use parser::Parser;
pub use program::{Compiled, Linked, Program, ProgramOrigin};
pub use symbol::{SymTable, Symbol, SymbolError};
pub use vm::{Vm, VmError};

// ============================================================================
// Helper functions for evaluating expressions
// ============================================================================

/// Evaluates an expression string with the standard library.
///
/// # Examples
///
/// ```
/// use expr_solver::eval;
///
/// let result = eval("2 + 3 * 4").unwrap();
/// assert_eq!(result.to_string(), "14");
/// ```
pub fn eval(expression: &str) -> Result<Decimal, String> {
    let program = load_with_table(expression, SymTable::stdlib())?;
    program.execute().map_err(|err| err.to_string())
}

/// Evaluates an expression string with a custom symbol table.
///
/// # Examples
///
/// ```
/// use expr_solver::{eval_with_table, SymTable};
/// use rust_decimal_macros::dec;
///
/// let mut table = SymTable::stdlib();
/// table.add_const("x", dec!(42)).unwrap();
///
/// let result = eval_with_table("x * 2", table).unwrap();
/// assert_eq!(result, dec!(84));
/// ```
pub fn eval_with_table(expression: &str, table: SymTable) -> Result<Decimal, String> {
    let program = load_with_table(expression, table)?;
    program.execute().map_err(|err| err.to_string())
}

/// Evaluates an expression from a binary file with the standard library.
///
/// # Examples
///
/// ```no_run
/// use expr_solver::eval_file;
///
/// let result = eval_file("expr.bin").unwrap();
/// ```
pub fn eval_file(path: impl AsRef<str>) -> Result<Decimal, String> {
    eval_file_with_table(path, SymTable::stdlib())
}

/// Evaluates an expression from a binary file with a custom symbol table.
///
/// # Examples
///
/// ```no_run
/// use expr_solver::{eval_file_with_table, SymTable};
///
/// let result = eval_file_with_table("expr.bin", SymTable::stdlib()).unwrap();
/// ```
pub fn eval_file_with_table(path: impl AsRef<str>, table: SymTable) -> Result<Decimal, String> {
    let program = Program::new_from_file(path.as_ref()).map_err(|err| err.to_string())?;
    let linked = program.link(table).map_err(|err| err.to_string())?;
    linked.execute().map_err(|err| err.to_string())
}

/// Loads and compiles an expression, returning a compiled program.
///
/// # Examples
///
/// ```
/// use expr_solver::{load, SymTable};
///
/// let program = load("2 + 3 * 4").unwrap();
/// let linked = program.link(SymTable::stdlib()).unwrap();
/// let result = linked.execute().unwrap();
/// assert_eq!(result.to_string(), "14");
/// ```
pub fn load(expression: &str) -> Result<Program<'_, Compiled>, String> {
    Program::new_from_source(expression).map_err(|err| err.to_string())
}

/// Loads, compiles, and links an expression, returning a ready-to-execute program.
///
/// # Examples
///
/// ```
/// use expr_solver::{load_with_table, SymTable};
///
/// let program = load_with_table("sin(pi/2)", SymTable::stdlib()).unwrap();
/// let result = program.execute().unwrap();
/// ```
pub fn load_with_table(expression: &str, table: SymTable) -> Result<Program<'_, Linked>, String> {
    let program = load(expression)?;
    program.link(table).map_err(|err| err.to_string())
}
