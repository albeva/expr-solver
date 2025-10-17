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

use std::{borrow::Cow, fs, path::PathBuf};

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

#[derive(Debug)]
enum EvalSource<'str> {
    Source(Cow<'str, Source<'str>>),
    File(PathBuf),
}

#[derive(Debug)]
pub struct Eval<'str> {
    source: EvalSource<'str>,
    table: Option<SymTable>,
}

impl<'str> Eval<'str> {
    pub fn new(string: &'str str) -> Self {
        let source = Source::new(string);
        Self {
            source: EvalSource::Source(Cow::Owned(source)),
            table: None,
        }
    }

    pub fn new_from_source(source: &'str Source<'str>) -> Self {
        Self {
            source: EvalSource::Source(Cow::Borrowed(source)),
            table: None,
        }
    }

    pub fn new_from_file(path: PathBuf) -> Self {
        Self {
            source: EvalSource::File(path),
            table: None,
        }
    }

    pub fn with_table(&mut self, table: SymTable) {
        self.table = Some(table);
    }

    pub fn run(&mut self) -> Result<Decimal, String> {
        let program = self.build_program()?;
        Vm::default().run(&program).map_err(|err| err.to_string())
    }

    pub fn compile_to_file(&mut self, path: &PathBuf) -> Result<(), String> {
        let program = self.build_program()?;
        let binary_data = program.compile().map_err(|err| err.to_string())?;
        fs::write(path, binary_data).map_err(|err| err.to_string())
    }

    pub fn build_program(&mut self) -> Result<Program<'_>, String> {
        let table = self.table.get_or_insert_with(SymTable::stdlib);
        match &self.source {
            EvalSource::Source(source) => {
                let mut parser = Parser::new(source);
                let mut ast: Expr = match parser
                    .parse()
                    .map_err(|err| Self::error_with_source(&err, source))?
                {
                    Some(ast) => ast,
                    None => return Ok(Program::default()),
                };
                Sema::new(table)
                    .visit(&mut ast)
                    .map_err(|err| Self::error_with_source(&err, source))?;
                IrBuilder::new().build(&ast).map_err(|err| err.to_string())
            }
            EvalSource::File(path) => {
                let binary_data = fs::read(path).map_err(|err| err.to_string())?;
                Program::load(&binary_data, table).map_err(|err| err.to_string())
            }
        }
    }

    fn error_with_source<T: SpanError>(error: &T, source: &Source) -> String {
        format!("{}\n{}", error, source.highlight(&error.span()))
    }
}
