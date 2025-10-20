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
mod source;

use crate::span::SpanError;
use rust_decimal::Decimal;

// Public API
pub use ast::{BinOp, Expr, ExprKind, UnOp};
pub use error::{LinkError, ParseError, ProgramError};
pub use metadata::{SymbolKind, SymbolMetadata};
pub use parser::Parser;
pub use program::{Compiled, Linked, Program, ProgramOrigin};
pub use source::Source;
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
    let source = Source::new(expression);
    let program = load_source_with_table(&source, SymTable::stdlib())?;
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
    let source = Source::new(expression);
    let program = load_source_with_table(&source, table)?;
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

/// Loads source code and returns a compiled program.
///
/// # Examples
///
/// ```
/// use expr_solver::{load_source, Source, SymTable};
///
/// let source = Source::new("2 + 3 * 4");
/// let program = load_source(&source).unwrap();
/// let linked = program.link(SymTable::stdlib()).unwrap();
/// let result = linked.execute().unwrap();
/// assert_eq!(result.to_string(), "14");
/// ```
pub fn load_source(source: &Source) -> Result<Program<'_, Compiled>, String> {
    Program::new_from_source(source).map_err(|err| {
        // Extract ParseError from ProgramError for nice formatting
        match err {
            ProgramError::ParseError(parse_err) => {
                format!("{}\n{}", parse_err, source.highlight(&parse_err.span()))
            }
            other => other.to_string(),
        }
    })
}

/// Loads source code and returns a linked program ready to execute.
///
/// # Examples
///
/// ```
/// use expr_solver::{load_source_with_table, Source, SymTable};
///
/// let source = Source::new("sin(pi/2)");
/// let program = load_source_with_table(&source, SymTable::stdlib()).unwrap();
/// let result = program.execute().unwrap();
/// ```
pub fn load_source_with_table(
    source: &Source,
    table: SymTable,
) -> Result<Program<Linked>, String> {
    let program = load_source(source)?;
    program.link(table).map_err(|err| err.to_string())
}
