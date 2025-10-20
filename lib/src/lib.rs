//! A mathematical expression evaluator library with bytecode compilation.
//!
//! This library provides a complete compiler pipeline for mathematical expressions,
//! from parsing to bytecode execution on a stack-based virtual machine.
//!
//! # Features
//!
//! - **Type-safe compilation** - Uses Rust's type system to enforce correct pipeline order
//! - **Flexible numeric types** - Choose between f64 (default) or 128-bit Decimal precision
//! - **Rich error messages** - Parse errors with syntax highlighting
//! - **Bytecode compilation** - Compile once, execute many times
//! - **Custom symbols** - Add your own constants and functions
//! - **Serialization** - Save/load compiled programs (requires `serialization` feature)
//!
//! ## Numeric Type Selection
//!
//! The library supports two numeric backends via feature flags:
//!
//! - **`f64-floats`** (default) - Standard f64 floating-point arithmetic. Faster and simpler,
//!   suitable for most use cases. Allows Inf and NaN results.
//! - **`decimal-precision`** - 128-bit Decimal arithmetic for high precision. No floating-point
//!   errors, checked arithmetic with overflow detection. Use for financial calculations or when
//!   exact decimal representation is required.
//!
//! **Note**: Only one numeric backend can be enabled at a time.
//!
//! # Quick Start
//!
//! ```
//! use expr_solver::eval;
//!
//! // Simple evaluation
//! let result = eval("2 + 3 * 4").unwrap();
//! assert_eq!(result.to_string(), "14");
//! ```
//!
//! # Custom Symbols
//!
//! ```
//! use expr_solver::{eval_with_table, SymTable, Number, ParseNumber};
//!
//! let mut table = SymTable::stdlib();
//! table.add_const("x", Number::parse_number("10").unwrap()).unwrap();
//!
//! let result = eval_with_table("x * 2", table).unwrap();
//! assert_eq!(result.to_string(), "20");
//! ```
//!
//! # Advanced: Type-State Pattern
//!
//! The `Program` type uses the type-state pattern to enforce correct usage:
//!
//! ```
//! use expr_solver::{SymTable, Program, Number, ParseNumber};
//!
//! // Compile expression to bytecode
//! let program = Program::new_from_source("x + y").unwrap();
//!
//! // Link with symbol table (validated at link time)
//! let mut table = SymTable::new();
//! table.add_const("x", Number::parse_number("10").unwrap()).unwrap();
//! table.add_const("y", Number::parse_number("5").unwrap()).unwrap();
//!
//! let linked = program.link(table).unwrap();
//!
//! // Execute
//! let result = linked.execute().unwrap();
//! assert_eq!(result.to_string(), "15");
//! ```
//!
//! # Supported Operators
//!
//! - Arithmetic: `+`, `-`, `*`, `/`, `^` (power), `!` (factorial), unary `-`
//! - Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=` (return 1 or 0)
//! - Grouping: `(` `)`
//!
//! # Built-in Functions
//!
//! See [`SymTable::stdlib()`] for the complete list of built-in functions and constants.

// Core types (shared)
mod ir;
pub mod number;
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

// Public API
pub use ast::{BinOp, Expr, ExprKind, UnOp};
pub use error::{LinkError, ParseError, ProgramError};
pub use metadata::{SymbolKind, SymbolMetadata};
pub use number::{Number, ParseNumber};
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
pub fn eval(expression: &str) -> Result<Number, String> {
    eval_with_table(expression, SymTable::stdlib())
}

/// Evaluates an expression string with a custom symbol table.
///
/// # Examples
///
/// ```
/// use expr_solver::{eval_with_table, SymTable, Number, ParseNumber};
///
/// let mut table = SymTable::stdlib();
/// table.add_const("x", Number::parse_number("42").unwrap()).unwrap();
///
/// let result = eval_with_table("x * 2", table).unwrap();
/// assert_eq!(result.to_string(), "84");
/// ```
pub fn eval_with_table(expression: &str, table: SymTable) -> Result<Number, String> {
    Program::new_from_source(expression)
        .map_err(|err| err.to_string())?
        .link(table)
        .map_err(|err| err.to_string())?
        .execute()
        .map_err(|err| err.to_string())
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
#[cfg(feature = "serialization")]
pub fn eval_file(path: impl AsRef<str>) -> Result<Number, String> {
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
#[cfg(feature = "serialization")]
pub fn eval_file_with_table(path: impl AsRef<str>, table: SymTable) -> Result<Number, String> {
    let program = Program::new_from_file(path.as_ref()).map_err(|err| err.to_string())?;
    let linked = program.link(table).map_err(|err| err.to_string())?;
    linked.execute().map_err(|err| err.to_string())
}
