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

mod ast;
mod ir;
mod lexer;
mod parser;
mod program;

mod sema;
mod source;
mod span;
mod symbol;
mod token;
mod vm;

// V2 implementation with improved architecture
pub mod v2;

use std::{borrow::Cow, fmt, fs, path::PathBuf};

// Public API
pub use ir::IrBuilder;
pub use parser::Parser;
pub use program::Program;

use crate::ast::Expr;
use crate::span::SpanError;
use rust_decimal::Decimal;
pub use sema::Sema;
pub use source::Source;
pub use symbol::{SymTable, Symbol, SymbolError};
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

impl<T: SpanError> From<(&T, &Source<'_>)> for FormattedError {
    fn from((error, source): (&T, &Source<'_>)) -> Self {
        Self {
            message: format!("{}\n{}", error, source.highlight(&error.span())),
        }
    }
}

#[derive(Debug)]
enum EvalSource<'str> {
    Source(Cow<'str, Source<'str>>),
    File(PathBuf),
}

/// Expression evaluator with support for custom symbols and bytecode compilation.
///
/// `Eval` is the main entry point for evaluating mathematical expressions. It supports
/// both quick one-off evaluations and reusable evaluators with custom symbol tables.
///
/// # Examples
///
/// ```
/// use expr_solver::Eval;
///
/// // Quick evaluation
/// let result = Eval::evaluate("2 + 3 * 4").unwrap();
/// assert_eq!(result.to_string(), "14");
///
/// // Reusable evaluator
/// let mut eval = Eval::new("sqrt(16) + pi");
/// let result = eval.run().unwrap();
/// ```
#[derive(Debug)]
pub struct Eval<'str> {
    source: EvalSource<'str>,
    table: SymTable,
}

impl<'str> Eval<'str> {
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
    pub fn evaluate(expression: &'str str) -> Result<Decimal, String> {
        Self::new(expression).run()
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
    pub fn evaluate_with_table(expression: &'str str, table: SymTable) -> Result<Decimal, String> {
        Self::with_table(expression, table).run()
    }

    /// Creates a new evaluator with the standard library.
    ///
    /// # Examples
    ///
    /// ```
    /// use expr_solver::Eval;
    ///
    /// let mut eval = Eval::new("sin(pi/2)");
    /// let result = eval.run().unwrap();
    /// ```
    pub fn new(string: &'str str) -> Self {
        Self::with_table(string, SymTable::stdlib())
    }

    /// Creates a new evaluator with a custom symbol table.
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
    /// let mut eval = Eval::with_table("x * 2", table);
    /// let result = eval.run().unwrap();
    /// assert_eq!(result, dec!(84));
    /// ```
    pub fn with_table(string: &'str str, table: SymTable) -> Self {
        let source = Source::new(string);
        Self {
            source: EvalSource::Source(Cow::Owned(source)),
            table,
        }
    }

    /// Creates a new evaluator from a [`Source`] reference.
    pub fn new_from_source(source: &'str Source<'str>) -> Self {
        Self::from_source_with_table(source, SymTable::stdlib())
    }

    /// Creates a new evaluator from a [`Source`] reference with a custom symbol table.
    pub fn from_source_with_table(source: &'str Source<'str>, table: SymTable) -> Self {
        Self {
            source: EvalSource::Source(Cow::Borrowed(source)),
            table,
        }
    }

    /// Creates a new evaluator from a compiled binary file.
    ///
    /// The file must have been created using [`compile_to_file`](Self::compile_to_file).
    pub fn new_from_file(path: PathBuf) -> Self {
        Self::from_file_with_table(path, SymTable::stdlib())
    }

    /// Creates a new evaluator from a compiled binary file with a custom symbol table.
    pub fn from_file_with_table(path: PathBuf, table: SymTable) -> Self {
        Self {
            source: EvalSource::File(path),
            table,
        }
    }

    /// Evaluates the expression and returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use expr_solver::Eval;
    ///
    /// let mut eval = Eval::new("2 + 3");
    /// assert_eq!(eval.run().unwrap().to_string(), "5");
    /// ```
    pub fn run(&mut self) -> Result<Decimal, String> {
        let program = self.build_program()?;
        Vm::default()
            .run(&program, &self.table)
            .map_err(|err| err.to_string())
    }

    /// Compiles the expression to a binary file.
    ///
    /// The compiled bytecode can later be loaded with [`new_from_file`](Self::new_from_file).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use expr_solver::Eval;
    /// use std::path::PathBuf;
    ///
    /// let mut eval = Eval::new("2 + 3 * 4");
    /// eval.compile_to_file(&PathBuf::from("expr.bin")).unwrap();
    /// ```
    pub fn compile_to_file(&mut self, path: &PathBuf) -> Result<(), String> {
        let program = self.build_program()?;
        let binary_data = program.compile().map_err(|err| err.to_string())?;
        fs::write(path, binary_data).map_err(|err| err.to_string())
    }

    /// Returns a human-readable assembly representation of the compiled expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use expr_solver::Eval;
    ///
    /// let mut eval = Eval::new("2 + 3");
    /// let assembly = eval.get_assembly().unwrap();
    /// assert!(assembly.contains("PUSH"));
    /// assert!(assembly.contains("ADD"));
    /// ```
    pub fn get_assembly(&mut self) -> Result<String, String> {
        let program = self.build_program()?;
        Ok(program.get_assembly(&self.table))
    }

    fn build_program(&mut self) -> Result<Program, String> {
        match &self.source {
            EvalSource::Source(source) => {
                let mut parser = Parser::new(source);
                let mut ast: Expr = match parser
                    .parse()
                    .map_err(|err| FormattedError::from((&err, source.as_ref())).to_string())?
                {
                    Some(ast) => ast,
                    None => return Ok(Program::default()),
                };
                Sema::new(&self.table)
                    .visit(&mut ast)
                    .map_err(|err| FormattedError::from((&err, source.as_ref())).to_string())?;
                IrBuilder::new().build(&ast).map_err(|err| err.to_string())
            }
            EvalSource::File(path) => {
                let binary_data = fs::read(path).map_err(|err| err.to_string())?;
                Program::load(&binary_data).map_err(|err| err.to_string())
            }
        }
    }
}
