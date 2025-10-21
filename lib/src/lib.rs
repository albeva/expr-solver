//! A mathematical expression evaluator library with bytecode compilation.
//!
//! This library provides a complete compiler pipeline for mathematical expressions,
//! from parsing to bytecode execution on a stack-based virtual machine.
//!
//! # Features
//!
//! - **Flexible numeric types** - Choose between f64 (default) or 128-bit Decimal precision
//! - **Rich error messages** - Parse errors with syntax highlighting
//! - **Bytecode compilation** - Compile once, execute many times
//! - **Custom symbols** - Add your own constants and functions
//! - **Local constants** - `let` ... `then` syntax for declaring scoped constants
//! - **Control flow** - Conditional expressions with `if(condition, then, else)`
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
//! use expr_solver::{eval, num};
//!
//! let result = eval("2 + 3 * 4").unwrap();
//! assert_eq!(result, num!(14));
//! ```
//!
//! ## Working with Numbers
//!
//! Use the [`num!`] macro to create numeric values that work with both f64 and Decimal backends:
//!
//! ```
//! use expr_solver::num;
//!
//! let x = num!(42);      // Works with any backend
//! let y = num!(3.14);    // Supports decimals
//! let z = num!(-10);     // And negative numbers
//! ```
//!
//! The macro ensures your code works regardless of which numeric backend is enabled.
//!
//! # Custom Symbols
//!
//! ```
//! use expr_solver::{num, eval_with_table, SymTable};
//!
//! let mut table = SymTable::stdlib();
//! table.add_const("x", num!(10), false).unwrap();
//!
//! let result = eval_with_table("x * 2", table).unwrap();
//! assert_eq!(result, num!(20));
//! ```
//!
//! # Advanced: Type-State Pattern
//!
//! The `Program` type uses the type-state pattern to enforce correct usage:
//!
//! ```
//! use expr_solver::{num, SymTable, Program};
//!
//! // Compile expression to bytecode
//! let program = Program::new_from_source("x + y").unwrap();
//!
//! // Link with symbol table (validated at link time)
//! let mut table = SymTable::new();
//! table.add_const("x", num!(10), false).unwrap();
//! table.add_const("y", num!(5), false).unwrap();
//!
//! let mut linked = program.link(table).unwrap();
//!
//! // Execute
//! let result = linked.execute().unwrap();
//! assert_eq!(result, num!(15));
//! ```
//!
//! # Local Constants with LET THEN
//!
//! Declare local constants using `let` ... `then` syntax:
//!
//! ```
//! use expr_solver::{eval, num};
//!
//! // Single declaration
//! let result = eval("let x = 10 then x * 2").unwrap();
//! assert_eq!(result, num!(20));
//!
//! // Multiple declarations
//! let result = eval("let x = 5, y = 3 then x + y").unwrap();
//! assert_eq!(result, num!(8));
//!
//! // Reference previous declarations
//! let result = eval("let x = 2, y = x * 3, z = y + 1 then z").unwrap();
//! assert_eq!(result, num!(7));
//!
//! // Use with globals and functions
//! let result = eval("let r = 5, area = pi * r ^ 2 then area").unwrap();
//! assert!((result - num!(78.53981633974483)).abs() < num!(0.0001));
//! ```
//!
//! **Notes:**
//! - Local constants are evaluated left-to-right
//! - Can reference previously declared locals and globals
//! - Cannot reference forward (e.g., `let x = y, y = 1 then x` is an error)
//! - Cannot shadow global constants (e.g., `let pi = 3 then pi` is an error)
//! - Duplicate names in the same `let` block are errors
//!
//! # Supported Operators
//!
//! - Arithmetic: `+`, `-`, `*`, `/`, `^` (power), `!` (factorial), unary `-`
//! - Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=` (return 1 or 0)
//! - Grouping: `(` `)`
//! - Control flow: `if(condition, then_value, else_value)`
//!
//! # Built-in Functions
//!
//! See [`SymTable::stdlib()`] for the complete list of built-in functions and constants.
//!
//! # Architecture
//!
//! The library uses a multi-stage compilation pipeline:
//!
//! 1. **Parsing** ([`Parser`]) - Source code → Abstract Syntax Tree (AST)
//! 2. **IR Generation** ([`IrBuilder`]) - AST → Bytecode + Symbol metadata
//! 3. **Linking** ([`Linker`]) - Bytecode + SymTable → Linked bytecode
//! 4. **Execution** ([`Vm`]) - Linked bytecode → Result
//!
//! Each stage has its own error type ([`ParseError`], [`IrError`], [`LinkerError`], [`VmError`])
//! which automatically converts to [`ProgramError`] for convenience.

// Core types (shared)
mod ir;
mod number;
mod span;
mod symbol;
mod symtable;
mod token;
mod vm;

// Expression solver implementation
mod ast;
mod error;
mod ir_builder;
mod lexer;
mod linker;
mod metadata;
mod parser;
mod print;
mod program;
mod style;

// Public API
pub use ast::{BinOp, Expr, ExprKind, UnOp};
pub use error::{IrError, LinkerError, ParseError, ProgramError};
pub use metadata::{SymbolKind, SymbolMetadata};
pub use number::{Number, ParseNumber};
pub use parser::Parser;
pub use print::Print;
pub use program::{Compiled, Linked, Program, ProgramOrigin};
pub use style::ExprStyle;
pub use symbol::{Symbol, SymbolError};
pub use symtable::SymTable;
pub use vm::{Vm, VmError};

// ============================================================================
// Helper functions for evaluating expressions
// ============================================================================

/// Evaluates an expression string with the standard library.
///
/// # Examples
///
/// ```
/// use expr_solver::{eval, num};
///
/// let result = eval("2 + 3 * 4").unwrap();
/// assert_eq!(result, num!(14));
/// ```
pub fn eval(expression: &str) -> Result<Number, String> {
    eval_with_table(expression, SymTable::stdlib())
}

/// Evaluates an expression string with a custom symbol table.
///
/// # Examples
///
/// ```
/// use expr_solver::{num, eval_with_table, SymTable};
///
/// let mut table = SymTable::stdlib();
/// table.add_const("x", num!(42), false).unwrap();
///
/// let result = eval_with_table("x * 2", table).unwrap();
/// assert_eq!(result, num!(84));
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
    let mut linked = program.link(table).map_err(|err| err.to_string())?;
    linked.execute().map_err(|err| err.to_string())
}
