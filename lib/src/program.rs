//! Type-state program implementation for compile-link-execute workflow.
//!
//! This module provides the [`Program`] type which orchestrates the compilation pipeline:
//!
//! 1. **Parsing** - Source code is parsed into an AST using [`Parser`]
//! 2. **IR Generation** - AST is compiled to bytecode using [`IrBuilder`]
//! 3. **Linking** - Bytecode is linked with a symbol table using [`Linker`]
//! 4. **Execution** - Linked bytecode is executed on the VM
//!
//! The type-state pattern ensures these stages occur in the correct order at compile time.

use super::error::{ParseError, ProgramError};
use super::ir_builder::IrBuilder;
use super::linker::Linker;
use super::metadata::SymbolMetadata;
use super::parser::Parser;
use crate::ir::Instr;
use crate::number::Number;
use crate::span::SpanError;
use crate::symtable::SymTable;
use crate::vm::{Vm, VmError};
use colored::Colorize;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

/// Current version of the program format
const PROGRAM_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Binary format for serialization
#[cfg(feature = "serialization")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BinaryFormat {
    version: String,
    bytecode: Vec<Instr>,
    symbols: Vec<SymbolMetadata>,
}

/// Origin of a compiled program.
#[derive(Debug, Clone)]
pub enum ProgramOrigin {
    /// Loaded from a file (path stored)
    #[cfg(feature = "serialization")]
    File(String),
    /// Compiled from source string
    Source,
    /// Deserialized from bytecode bytes
    #[cfg(feature = "serialization")]
    Bytecode,
}

/// Type-state program using Rust's type system to enforce correct usage.
///
/// # Examples
///
/// ```
/// use expr_solver::{num, Program, SymTable};
///
/// // Compile from source
/// let program = Program::new_from_source("x * 2 + 1").unwrap();
///
/// // Link with symbol table
/// let mut table = SymTable::new();
/// table.add_const("x", num!(5), false).unwrap();
/// let mut linked = program.link(table).unwrap();
///
/// // Execute
/// assert_eq!(linked.execute().unwrap(), num!(11));
/// ```
#[derive(Debug)]
pub struct Program<'src, State> {
    source: Option<&'src str>,
    state: State,
}

/// Compiled state - bytecode ready for linking.
#[derive(Debug)]
pub struct Compiled {
    origin: ProgramOrigin,
    version: String,
    bytecode: Vec<Instr>,
    symbols: Vec<SymbolMetadata>,
}

/// Linked state - ready to execute.
#[derive(Debug)]
pub struct Linked {
    #[allow(dead_code)]
    origin: ProgramOrigin,
    version: String,
    bytecode: Vec<Instr>,
    symtable: SymTable,
}

// ============================================================================
// Program - Public constructors (return Compiled state directly)
// ============================================================================

impl<'src> Program<'src, Compiled> {
    // ========================================================================
    // Public API
    // ========================================================================

    /// Creates a compiled program from source code.
    ///
    /// # Examples
    ///
    /// ```
    /// use expr_solver::Program;
    ///
    /// let program = Program::new_from_source("2 + 3 * 4").unwrap();
    /// ```
    pub fn new_from_source(source: &'src str) -> Result<Self, ProgramError> {
        let trimmed = source.trim();

        // Parse
        let mut parser = Parser::new(trimmed);
        let ast_opt = parser.parse().map_err(|parse_err| {
            // Format error with source highlighting
            let highlighted = Self::highlight_error(trimmed, &parse_err);
            ProgramError::ParseError(format!("{}\n{}", parse_err, highlighted))
        })?;

        // Compile (handle empty input by creating empty bytecode)
        let (bytecode, symbols) = if let Some(ast) = ast_opt {
            IrBuilder::new().build(&ast)?
        } else {
            // Empty input -> empty program (VM will return 0)
            (Vec::new(), Vec::new())
        };

        Ok(Program {
            source: Some(trimmed),
            state: Compiled {
                origin: ProgramOrigin::Source,
                version: PROGRAM_VERSION.to_string(),
                bytecode,
                symbols,
            },
        })
    }

    /// Creates a compiled program from a binary file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use expr_solver::Program;
    ///
    /// let program = Program::new_from_file("expr.bin").unwrap();
    /// ```
    #[cfg(feature = "serialization")]
    pub fn new_from_file(path: impl Into<String>) -> Result<Self, ProgramError> {
        let path_str = path.into();
        let data = std::fs::read(&path_str)?;
        Self::from_bytecode(&data, ProgramOrigin::File(path_str))
    }

    /// Creates a compiled program from bytecode bytes.
    ///
    /// Deserializes the bytecode and validates the version.
    #[cfg(feature = "serialization")]
    pub fn new_from_bytecode(data: &[u8]) -> Result<Self, ProgramError> {
        Self::from_bytecode(data, ProgramOrigin::Bytecode)
    }

    /// Links the bytecode with a symbol table.
    ///
    /// Validates that all required symbols are present and compatible.
    ///
    /// # Examples
    ///
    /// ```
    /// use expr_solver::{Program, SymTable};
    ///
    /// let program = Program::new_from_source("sin(pi)").unwrap();
    /// let linked = program.link(SymTable::stdlib()).unwrap();
    /// ```
    pub fn link(self, table: SymTable) -> Result<Program<'src, Linked>, ProgramError> {
        let linker = Linker::new(self.state.bytecode, self.state.symbols, table);
        let (bytecode, symtable) = linker.link()?;

        Ok(Program {
            source: self.source,
            state: Linked {
                origin: self.state.origin,
                version: self.state.version,
                bytecode,
                symtable,
            },
        })
    }

    // ========================================================================
    // Private helpers
    // ========================================================================

    /// Internal helper to create program from bytecode with a specific origin.
    #[cfg(feature = "serialization")]
    fn from_bytecode(data: &[u8], origin: ProgramOrigin) -> Result<Self, ProgramError> {
        let config = bincode::config::standard();
        let (binary, _): (BinaryFormat, _) = bincode::serde::decode_from_slice(data, config)?;

        // Validate version
        if binary.version != PROGRAM_VERSION {
            return Err(ProgramError::IncompatibleVersion {
                expected: PROGRAM_VERSION.to_string(),
                found: binary.version,
            });
        }

        Ok(Program {
            source: None, // No source for bytecode
            state: Compiled {
                origin,
                version: binary.version,
                bytecode: binary.bytecode,
                symbols: binary.symbols,
            },
        })
    }

    /// Highlights an error in the source code.
    fn highlight_error(input: &str, error: &ParseError) -> String {
        let span = error.span();
        let pre = Self::escape(&input[..span.start]);
        let tok = Self::escape(&input[span.start..span.end]);
        let post = Self::escape(&input[span.end..]);
        let line = format!("{}{}{}", pre, tok.red().bold(), post);

        let caret = "^".green().bold();
        let squiggly_len = UnicodeWidthStr::width(tok.as_str());
        let caret_offset = UnicodeWidthStr::width(pre.as_str()) + caret.len();

        format!(
            "1 | {0}\n  | {1: >2$}{3}",
            line,
            caret,
            caret_offset,
            "~".repeat(squiggly_len.saturating_sub(1)).green()
        )
    }

    /// Escapes special characters for display.
    fn escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                other => out.push(other),
            }
        }
        out
    }
}

// ============================================================================
// Program<Linked> - After linking, ready to execute
// ============================================================================

impl<'src> Program<'src, Linked> {
    // ========================================================================
    // Public API
    // ========================================================================

    /// Executes the program and returns the result.
    pub fn execute(&mut self) -> Result<Number, VmError> {
        Vm::run(&self.state.bytecode, &mut self.state.symtable)
    }

    /// Returns a mutable reference to the symbol table.
    pub fn symtable_mut(&mut self) -> &mut SymTable {
        &mut self.state.symtable
    }

    /// Returns a human-readable assembly representation of the program.
    pub fn get_assembly(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        out += &format!("; VERSION {}\n", self.state.version)
            .bright_black()
            .to_string();

        for (i, instr) in self.state.bytecode.iter().enumerate() {
            let _ = write!(out, "{} ", format!("{:04X}", i).yellow());
            let line = match instr {
                Instr::Push(v) => format!("{} {}", "PUSH".magenta(), v.to_string().green()),
                Instr::Load(idx) => {
                    let sym_name = self
                        .state
                        .symtable
                        .get_by_index(*idx)
                        .map(|s| s.name())
                        .expect("Symbol not found in assembly");
                    format!("{} {}", "LOAD".magenta(), sym_name.blue())
                }
                Instr::Store(idx) => {
                    let sym_name = self
                        .state
                        .symtable
                        .get_by_index(*idx)
                        .map(|s| s.name())
                        .expect("Symbol not found in assembly");
                    format!("{} {}", "STORE".magenta(), sym_name.blue())
                }
                Instr::Neg => format!("{}", "NEG".magenta()),
                Instr::Add => format!("{}", "ADD".magenta()),
                Instr::Sub => format!("{}", "SUB".magenta()),
                Instr::Mul => format!("{}", "MUL".magenta()),
                Instr::Div => format!("{}", "DIV".magenta()),
                Instr::Pow => format!("{}", "POW".magenta()),
                Instr::Fact => format!("{}", "FACT".magenta()),
                Instr::Call(idx, argc) => {
                    let sym_name = self
                        .state
                        .symtable
                        .get_by_index(*idx)
                        .map(|s| s.name())
                        .expect("Symbol not found in assembly");
                    format!(
                        "{} {} args: {}",
                        "CALL".magenta(),
                        sym_name.cyan(),
                        argc.to_string().bright_blue()
                    )
                }
                Instr::Equal => format!("{}", "EQ".magenta()),
                Instr::NotEqual => format!("{}", "NEQ".magenta()),
                Instr::Less => format!("{}", "LT".magenta()),
                Instr::LessEqual => format!("{}", "LTE".magenta()),
                Instr::Greater => format!("{}", "GT".magenta()),
                Instr::GreaterEqual => format!("{}", "GTE".magenta()),
                Instr::Jmp(target) => {
                    format!("{} {}", "JMP".magenta(), format!("{:04X}", target).yellow())
                }
                Instr::Jz(target) => {
                    format!("{} {}", "JZ".magenta(), format!("{:04X}", target).yellow())
                }
            };
            let _ = writeln!(out, "{}", line);
        }
        out
    }

    /// Converts the program to bytecode bytes.
    ///
    /// This involves reverse-mapping the bytecode indices back to metadata indices.
    #[cfg(feature = "serialization")]
    pub fn to_bytecode(&self) -> Result<Vec<u8>, ProgramError> {
        use std::collections::HashMap;

        let mut reverse_map = HashMap::new();
        let mut symbols = Vec::new();

        // Helper closure to get or create metadata index
        // All indices are valid since we successfully linked
        let mut get_or_create_metadata = |idx: usize| -> usize {
            if let Some(&existing) = reverse_map.get(&idx) {
                existing
            } else {
                let symbol = self
                    .state
                    .symtable
                    .get_by_index(idx)
                    .expect("symbol index must be valid after linking");

                let new_idx = symbols.len();
                symbols.push(symbol.into());
                reverse_map.insert(idx, new_idx);
                new_idx
            }
        };

        // Single pass: build symbol mapping and rewrite bytecode
        let bytecode: Vec<Instr> = self
            .state
            .bytecode
            .iter()
            .map(|instr| match instr {
                Instr::Load(idx) => Instr::Load(get_or_create_metadata(*idx)),
                Instr::Store(idx) => Instr::Store(get_or_create_metadata(*idx)),
                Instr::Call(idx, argc) => Instr::Call(get_or_create_metadata(*idx), *argc),
                other => other.clone(),
            })
            .collect();

        // Serialize
        let binary = BinaryFormat {
            version: self.state.version.clone(),
            bytecode,
            symbols,
        };

        let config = bincode::config::standard();
        Ok(bincode::serde::encode_to_vec(&binary, config)?)
    }

    /// Saves the program bytecode to a file.
    #[cfg(feature = "serialization")]
    pub fn save_bytecode_to_file(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), ProgramError> {
        let bytecode = self.to_bytecode()?;
        std::fs::write(path, bytecode)?;
        Ok(())
    }
}
