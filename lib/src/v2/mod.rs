//! V2 implementation of the expression solver with improved architecture.
//!
//! This version introduces a type-state pattern for Program with clear state transitions:
//! - `Program<Initial>` - Created from source or file
//! - `Program<Parsed>` - After parsing source to AST
//! - `Program<Compiled>` - After compiling to bytecode with symbol metadata
//! - `Program<Linked>` - After linking with a symbol table (ready to execute)
//!
//! Key improvements:
//! - Program owns its symbol table after linking
//! - Symbol table can be modified via `symtable_mut()`
//! - Binary deserialization includes validation and index remapping
//! - Type-safe state transitions prevent invalid operations

mod ast;
mod error;
mod lexer;
mod metadata;
mod parser;
mod program;
mod source;

// Public API exports
pub use ast::{BinOp, Expr, ExprKind, UnOp};
pub use error::{LinkError, ParseError, ProgramError};
pub use metadata::{SymbolKind, SymbolMetadata};
pub use parser::Parser;
pub use program::{Compiled, Initial, Linked, Parsed, Program, ProgramOrigin};
pub use source::Source;
