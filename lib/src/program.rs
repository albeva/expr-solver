//! Type-state program implementation with improved architecture.

use super::ast::{BinOp, Expr, ExprKind, UnOp};
use super::error::{LinkError, ParseError, ProgramError};
use super::metadata::{SymbolKind, SymbolMetadata};
use super::parser::Parser;
use crate::ir::Instr;
use crate::span::{Span, SpanError};
use crate::symbol::{SymTable, Symbol};
use crate::vm::{Vm, VmError};
use colored::Colorize;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

/// Current version of the program format
const PROGRAM_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Binary format for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BinaryFormat {
    version: String,
    bytecode: Vec<Instr>,
    symbols: Vec<SymbolMetadata>,
}

/// Origin of a program (source code or compiled file)
#[derive(Debug, Clone)]
pub enum ProgramOrigin {
    File(String),
    Source,
    Bytecode,
}

/// Type-state program structure with optional source reference
#[derive(Debug)]
pub struct Program<'src, State> {
    source: Option<&'src str>,
    state: State,
}

/// Compiled state - AST compiled to bytecode with symbol metadata
#[derive(Debug)]
pub struct Compiled {
    origin: ProgramOrigin,
    version: String,
    bytecode: Vec<Instr>,
    symbols: Vec<SymbolMetadata>,
}

/// Linked state - bytecode linked with symbol table, ready to execute
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
    /// Creates a compiled program from source code.
    ///
    /// Parses and compiles the source in one step.
    pub fn new_from_source(source: &'src str) -> Result<Self, ProgramError> {
        let trimmed = source.trim();

        // Parse
        let mut parser = Parser::new(trimmed);
        let ast = parser
            .parse()
            .map_err(|parse_err| {
                // Format error with source highlighting
                let highlighted = Self::highlight_error(trimmed, &parse_err);
                ProgramError::ParseError(format!("{}\n{}", parse_err, highlighted))
            })?
            .ok_or_else(|| {
                let parse_err = ParseError::UnexpectedEof {
                    span: Span::new(0, 0),
                };
                let highlighted = Self::highlight_error(trimmed, &parse_err);
                ProgramError::ParseError(format!("{}\n{}", parse_err, highlighted))
            })?;

        // Compile
        let (bytecode, symbols) = Self::generate_bytecode(&ast);

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

    /// Highlights an error in the source code (private helper).
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

    /// Escapes special characters for display (private helper).
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

    /// Creates a compiled program from a binary file.
    ///
    /// Reads and deserializes the bytecode from the file.
    pub fn new_from_file(path: impl Into<String>) -> Result<Self, ProgramError> {
        let path_str = path.into();
        let data = std::fs::read(&path_str)?;
        Self::new_from_bytecode(&data)
    }

    /// Creates a compiled program from bytecode bytes.
    ///
    /// Deserializes the bytecode and validates the version.
    pub fn new_from_bytecode(data: &[u8]) -> Result<Self, ProgramError> {
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
                origin: ProgramOrigin::Bytecode,
                version: binary.version,
                bytecode: binary.bytecode,
                symbols: binary.symbols,
            },
        })
    }

    /// Generates bytecode and collects symbol metadata in a single AST traversal (private).
    fn generate_bytecode(ast: &Expr) -> (Vec<Instr>, Vec<SymbolMetadata>) {
        let mut bytecode = Vec::new();
        let mut symbols = Vec::new();
        Self::emit_instr(ast, &mut bytecode, &mut symbols);
        (bytecode, symbols)
    }

    fn emit_instr(expr: &Expr, bytecode: &mut Vec<Instr>, symbols: &mut Vec<SymbolMetadata>) {
        match &expr.kind {
            ExprKind::Literal(v) => {
                bytecode.push(Instr::Push(*v));
            }
            ExprKind::Ident { name } => {
                // Get or create index for this constant
                let idx = Self::get_or_create_symbol(name, SymbolKind::Const, symbols);
                bytecode.push(Instr::Load(idx));
            }
            ExprKind::Unary { op, expr } => {
                Self::emit_instr(expr, bytecode, symbols);
                match op {
                    UnOp::Neg => bytecode.push(Instr::Neg),
                    UnOp::Fact => bytecode.push(Instr::Fact),
                }
            }
            ExprKind::Binary { op, left, right } => {
                Self::emit_instr(left, bytecode, symbols);
                Self::emit_instr(right, bytecode, symbols);
                bytecode.push(match op {
                    BinOp::Add => Instr::Add,
                    BinOp::Sub => Instr::Sub,
                    BinOp::Mul => Instr::Mul,
                    BinOp::Div => Instr::Div,
                    BinOp::Pow => Instr::Pow,
                    BinOp::Equal => Instr::Equal,
                    BinOp::NotEqual => Instr::NotEqual,
                    BinOp::Less => Instr::Less,
                    BinOp::LessEqual => Instr::LessEqual,
                    BinOp::Greater => Instr::Greater,
                    BinOp::GreaterEqual => Instr::GreaterEqual,
                });
            }
            ExprKind::Call { name, args } => {
                // Emit arguments first
                for arg in args {
                    Self::emit_instr(arg, bytecode, symbols);
                }

                // Get or create index for this function
                let idx = Self::get_or_create_symbol(
                    name,
                    SymbolKind::Func {
                        arity: args.len(),
                        variadic: false, // Will be validated during linking
                    },
                    symbols,
                );
                bytecode.push(Instr::Call(idx, args.len()));
            }
        }
    }

    /// Gets existing symbol index or creates a new one.
    /// For ~50 symbols, linear search is faster than HashMap overhead.
    fn get_or_create_symbol(
        name: &str,
        kind: SymbolKind,
        symbols: &mut Vec<SymbolMetadata>,
    ) -> usize {
        // Check if symbol already exists
        if let Some(pos) = symbols.iter().position(|s| s.name == name) {
            return pos;
        }

        // Create new symbol entry
        symbols.push(SymbolMetadata {
            name: name.to_string().into(),
            kind,
            index: None,
        });
        symbols.len() - 1
    }

    /// Links the bytecode with a symbol table, validating and remapping indices.
    pub fn link(mut self, table: SymTable) -> Result<Program<'src, Linked>, ProgramError> {
        // Validate symbols and fill in their resolved indices
        for metadata in &mut self.state.symbols {
            let (resolved_idx, symbol) =
                table
                    .get_with_index(&metadata.name)
                    .ok_or_else(|| LinkError::MissingSymbol {
                        name: metadata.name.to_string(),
                    })?;

            // Validate kind matches
            Self::validate_symbol_kind(metadata, symbol)?;

            // Store resolved index in metadata
            metadata.index = Some(resolved_idx);
        }

        // Rewrite all indices in bytecode using resolved indices from metadata
        for instr in &mut self.state.bytecode {
            match instr {
                Instr::Load(idx) => {
                    *idx = self.state.symbols[*idx]
                        .index
                        .expect("Symbol should have been resolved during linking");
                }
                Instr::Call(idx, _) => {
                    *idx = self.state.symbols[*idx]
                        .index
                        .expect("Symbol should have been resolved during linking");
                }
                _ => {}
            }
        }

        Ok(Program {
            source: self.source,
            state: Linked {
                origin: self.state.origin,
                version: self.state.version,
                bytecode: self.state.bytecode,
                symtable: table,
            },
        })
    }

    /// Validates that a symbol matches the expected kind.
    fn validate_symbol_kind(metadata: &SymbolMetadata, symbol: &Symbol) -> Result<(), LinkError> {
        match (&metadata.kind, symbol) {
            (SymbolKind::Const, Symbol::Const { .. }) => Ok(()),
            (
                SymbolKind::Func { arity, .. },
                Symbol::Func {
                    args: min_args,
                    variadic,
                    ..
                },
            ) => {
                // Check if the call is valid:
                // - For non-variadic: arity must match exactly
                // - For variadic: arity must be >= min_args
                let valid = if *variadic {
                    arity >= min_args
                } else {
                    arity == min_args
                };

                if valid {
                    Ok(())
                } else {
                    let expected_msg = if *variadic {
                        format!("at least {} arguments", min_args)
                    } else {
                        format!("exactly {} arguments", min_args)
                    };
                    Err(LinkError::TypeMismatch {
                        name: metadata.name.to_string(),
                        expected: expected_msg,
                        found: format!("{} arguments provided", arity),
                    })
                }
            }
            (SymbolKind::Const, Symbol::Func { .. }) => Err(LinkError::TypeMismatch {
                name: metadata.name.to_string(),
                expected: "constant".to_string(),
                found: "function".to_string(),
            }),
            (SymbolKind::Func { .. }, Symbol::Const { .. }) => Err(LinkError::TypeMismatch {
                name: metadata.name.to_string(),
                expected: "function".to_string(),
                found: "constant".to_string(),
            }),
        }
    }

    /// Returns the symbol metadata required by this program.
    pub fn symbols(&self) -> &[SymbolMetadata] {
        &self.state.symbols
    }

    /// Returns the version of this program.
    pub fn version(&self) -> &str {
        &self.state.version
    }
}

// ============================================================================
// Program<Linked> - After linking, ready to execute
// ============================================================================

impl<'src> Program<'src, Linked> {
    /// Executes the program and returns the result.
    pub fn execute(&self) -> Result<Decimal, VmError> {
        Vm::default().run_bytecode(&self.state.bytecode, &self.state.symtable)
    }

    /// Returns a reference to the symbol table.
    pub fn symtable(&self) -> &SymTable {
        &self.state.symtable
    }

    /// Returns a mutable reference to the symbol table.
    pub fn symtable_mut(&mut self) -> &mut SymTable {
        &mut self.state.symtable
    }

    /// Returns the version of this program.
    pub fn version(&self) -> &str {
        &self.state.version
    }

    /// Returns a human-readable assembly representation of the program.
    pub fn get_assembly(&self) -> String {
        Self::format_assembly(
            &self.state.version,
            &self.state.bytecode,
            &self.state.symtable,
        )
    }

    /// Converts the program to bytecode bytes.
    ///
    /// This involves reverse-mapping the bytecode indices back to metadata indices.
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
    pub fn save_bytecode_to_file(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), ProgramError> {
        let bytecode = self.to_bytecode()?;
        std::fs::write(path, bytecode)?;
        Ok(())
    }

    /// Formats bytecode as human-readable assembly.
    fn format_assembly(version: &str, bytecode: &[Instr], table: &SymTable) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        out += &format!("; VERSION {}\n", version)
            .bright_black()
            .to_string();

        for (i, instr) in bytecode.iter().enumerate() {
            let _ = write!(out, "{} ", format!("{:04X}", i).yellow());
            let line = match instr {
                Instr::Push(v) => format!("{} {}", "PUSH".magenta(), v.to_string().green()),
                Instr::Load(idx) => {
                    let sym_name = table.get_by_index(*idx).map(|s| s.name()).unwrap_or("???");
                    format!("{} {}", "LOAD".magenta(), sym_name.blue())
                }
                Instr::Neg => format!("{}", "NEG".magenta()),
                Instr::Add => format!("{}", "ADD".magenta()),
                Instr::Sub => format!("{}", "SUB".magenta()),
                Instr::Mul => format!("{}", "MUL".magenta()),
                Instr::Div => format!("{}", "DIV".magenta()),
                Instr::Pow => format!("{}", "POW".magenta()),
                Instr::Fact => format!("{}", "FACT".magenta()),
                Instr::Call(idx, argc) => {
                    let sym_name = table.get_by_index(*idx).map(|s| s.name()).unwrap_or("???");
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
            };
            let _ = writeln!(out, "{}", line);
        }
        out
    }
}
