//! Type-state program implementation with improved architecture.

use super::ast::{BinOp, Expr, ExprKind, UnOp};
use super::error::{CompileError, LinkError, ProgramError};
use super::metadata::{SymbolKind, SymbolMetadata};
use super::parser::Parser;
use super::source::Source;
use crate::ir::Instr;
use crate::symbol::{Symbol, SymTable};
use crate::vm::{Vm, VmError};
use colored::Colorize;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
    Source(Source),
}

/// Type-state program structure
#[derive(Debug)]
pub struct Program<State> {
    state: State,
}

/// Initial state - program just created from source or file path
#[derive(Debug)]
pub struct Initial {
    origin: ProgramOrigin,
}

/// Parsed state - source has been parsed to AST
#[derive(Debug)]
pub struct Parsed {
    source: Source,
    ast: Expr,
}

/// Compiled state - AST compiled to bytecode with symbol metadata
#[derive(Debug)]
pub struct Compiled {
    origin: ProgramOrigin,
    bytecode: Vec<Instr>,
    symbols: Vec<SymbolMetadata>,
}

/// Linked state - bytecode linked with symbol table, ready to execute
#[derive(Debug)]
pub struct Linked {
    origin: ProgramOrigin,
    bytecode: Vec<Instr>,
    symtable: SymTable,
}

// ============================================================================
// Program<Initial> - Entry point
// ============================================================================

impl Program<Initial> {
    /// Creates a new program from source code.
    pub fn new_from_source(source: Source) -> Self {
        Program {
            state: Initial {
                origin: ProgramOrigin::Source(source),
            },
        }
    }

    /// Creates a new program from a file path (to be loaded later).
    pub fn new_from_file(path: String) -> Self {
        Program {
            state: Initial {
                origin: ProgramOrigin::File(path),
            },
        }
    }

    /// Parses source code into an AST.
    ///
    /// Only valid for programs created from source.
    pub fn parse(self) -> Result<Program<Parsed>, ProgramError> {
        match self.state.origin {
            ProgramOrigin::Source(source) => {
                let mut parser = Parser::new(&source);
                let ast = parser
                    .parse()
                    .map_err(ProgramError::ParseError)?
                    .ok_or_else(|| {
                        ProgramError::ParseError(super::error::ParseError::UnexpectedEof {
                            span: crate::span::Span::new(0, 0),
                        })
                    })?;

                Ok(Program {
                    state: Parsed { source, ast },
                })
            }
            ProgramOrigin::File(_) => Err(ProgramError::ParseError(
                super::error::ParseError::UnexpectedToken {
                    message: "Cannot parse a file-based program. Use deserialize instead."
                        .to_string(),
                    span: crate::span::Span::new(0, 0),
                },
            )),
        }
    }

    /// Deserializes a program from binary data (for file-based programs).
    ///
    /// Returns a `Program<Compiled>` state directly.
    pub fn deserialize(self, data: &[u8]) -> Result<Program<Compiled>, ProgramError> {
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
            state: Compiled {
                origin: self.state.origin,
                bytecode: binary.bytecode,
                symbols: binary.symbols,
            },
        })
    }
}

// ============================================================================
// Program<Parsed> - After parsing
// ============================================================================

impl Program<Parsed> {
    /// Compiles the AST to bytecode with symbol metadata.
    ///
    /// Does everything in a single AST traversal: generates bytecode and collects
    /// symbol metadata simultaneously.
    pub fn compile(self) -> Result<Program<Compiled>, ProgramError> {
        let ast = self.state.ast;

        // Generate bytecode and collect symbols in one pass
        let (bytecode, symbols) = Self::generate_bytecode(&ast)?;

        Ok(Program {
            state: Compiled {
                origin: ProgramOrigin::Source(self.state.source),
                bytecode,
                symbols,
            },
        })
    }

    /// Generates bytecode and collects symbol metadata in a single AST traversal.
    fn generate_bytecode(ast: &Expr) -> Result<(Vec<Instr>, Vec<SymbolMetadata>), CompileError> {
        let mut bytecode = Vec::new();
        let mut symbols = Vec::new();
        Self::emit_instr(ast, &mut bytecode, &mut symbols)?;
        Ok((bytecode, symbols))
    }

    fn emit_instr(
        expr: &Expr,
        bytecode: &mut Vec<Instr>,
        symbols: &mut Vec<SymbolMetadata>,
    ) -> Result<(), CompileError> {
        match &expr.kind {
            ExprKind::Literal(v) => {
                bytecode.push(Instr::Push(*v));
            }
            ExprKind::Ident { name, .. } => {
                // Get or create index for this constant
                let idx = Self::get_or_create_symbol(name, SymbolKind::Const, symbols);
                bytecode.push(Instr::Load(idx));
            }
            ExprKind::Unary { op, expr } => {
                Self::emit_instr(expr, bytecode, symbols)?;
                match op {
                    UnOp::Neg => bytecode.push(Instr::Neg),
                    UnOp::Fact => bytecode.push(Instr::Fact),
                }
            }
            ExprKind::Binary { op, left, right } => {
                Self::emit_instr(left, bytecode, symbols)?;
                Self::emit_instr(right, bytecode, symbols)?;
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
            ExprKind::Call { name, args, .. } => {
                // Emit arguments first
                for arg in args {
                    Self::emit_instr(arg, bytecode, symbols)?;
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
        Ok(())
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
}

// ============================================================================
// Program<Compiled> - After compilation or deserialization
// ============================================================================

impl Program<Compiled> {
    /// Links the bytecode with a symbol table, validating and remapping indices.
    pub fn link(mut self, table: SymTable) -> Result<Program<Linked>, ProgramError> {
        // Validate symbols and fill in their resolved indices
        for metadata in &mut self.state.symbols {
            let (resolved_idx, symbol) = table
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
            state: Linked {
                origin: self.state.origin,
                bytecode: self.state.bytecode,
                symtable: table,
            },
        })
    }

    /// Validates that a symbol matches the expected kind.
    fn validate_symbol_kind(
        metadata: &SymbolMetadata,
        symbol: &Symbol,
    ) -> Result<(), LinkError> {
        match (&metadata.kind, symbol) {
            (SymbolKind::Const, Symbol::Const { .. }) => Ok(()),
            (
                SymbolKind::Func { arity, variadic },
                Symbol::Func {
                    args,
                    variadic: v,
                    ..
                },
            ) => {
                if arity == args && variadic == v {
                    Ok(())
                } else {
                    Err(LinkError::TypeMismatch {
                        name: metadata.name.to_string(),
                        expected: format!("function(arity={}, variadic={})", arity, variadic),
                        found: format!("function(arity={}, variadic={})", args, v),
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
}

// ============================================================================
// Program<Linked> - After linking, ready to execute
// ============================================================================

impl Program<Linked> {
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

    /// Returns a human-readable assembly representation of the program.
    pub fn get_assembly(&self) -> String {
        Self::format_assembly(&self.state.bytecode, &self.state.symtable)
    }

    /// Serializes the program to binary format.
    ///
    /// This involves reverse-mapping the bytecode indices back to metadata indices.
    pub fn serialize(&self) -> Result<Vec<u8>, ProgramError> {
        // Step 1: Find all symbol indices used in bytecode
        let mut used_indices = BTreeSet::new();
        for instr in &self.state.bytecode {
            match instr {
                Instr::Load(idx) | Instr::Call(idx, _) => {
                    used_indices.insert(*idx);
                }
                _ => {}
            }
        }

        // Step 2: Build reverse mapping: symtable_idx → metadata_idx
        // We use Vec since we need index-based lookup
        let max_idx = used_indices.iter().max().copied().unwrap_or(0);
        let mut reverse_remap = vec![None; max_idx + 1];
        let mut symbols = Vec::with_capacity(used_indices.len());

        for (metadata_idx, &symtable_idx) in used_indices.iter().enumerate() {
            let symbol = self
                .state
                .symtable
                .get_by_index(symtable_idx)
                .ok_or(ProgramError::InvalidSymbolIndex(symtable_idx))?;

            let kind = match symbol {
                Symbol::Const { .. } => SymbolKind::Const,
                Symbol::Func { args, variadic, .. } => SymbolKind::Func {
                    arity: *args,
                    variadic: *variadic,
                },
            };

            symbols.push(SymbolMetadata {
                name: symbol.name().to_string().into(),
                kind,
                index: None,
            });

            reverse_remap[symtable_idx] = Some(metadata_idx);
        }

        // Step 3: Rewrite bytecode to use metadata indices
        let bytecode: Vec<Instr> = self
            .state
            .bytecode
            .iter()
            .map(|instr| match instr {
                Instr::Load(idx) => Instr::Load(
                    reverse_remap[*idx].expect("Symbol should have been mapped"),
                ),
                Instr::Call(idx, argc) => Instr::Call(
                    reverse_remap[*idx].expect("Symbol should have been mapped"),
                    *argc,
                ),
                other => other.clone(),
            })
            .collect();

        // Step 4: Serialize
        let binary = BinaryFormat {
            version: PROGRAM_VERSION.to_string(),
            bytecode,
            symbols,
        };

        let config = bincode::config::standard();
        bincode::serde::encode_to_vec(&binary, config)
            .map_err(|e| ProgramError::SerializationError(e.to_string()))
    }

    /// Returns a list of all symbols used by this program.
    pub fn emit_symbols(&self) -> Vec<String> {
        let mut used_indices = BTreeSet::new();
        for instr in &self.state.bytecode {
            match instr {
                Instr::Load(idx) | Instr::Call(idx, _) => {
                    used_indices.insert(*idx);
                }
                _ => {}
            }
        }

        used_indices
            .iter()
            .filter_map(|idx| {
                self.state
                    .symtable
                    .get_by_index(*idx)
                    .map(|s| s.name().to_string())
            })
            .collect()
    }

    /// Formats bytecode as human-readable assembly.
    fn format_assembly(bytecode: &[Instr], table: &SymTable) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        out += &format!("; VERSION {}\n", PROGRAM_VERSION)
            .bright_black()
            .to_string();

        let emit = |mnemonic: &str| -> String { format!("{}", mnemonic.magenta()) };
        let emit1 = |mnemonic: &str, op: &str| -> String {
            format!("{} {}", mnemonic.magenta(), op.green())
        };

        for (i, instr) in bytecode.iter().enumerate() {
            let _ = write!(out, "{} ", format!("{:04X}", i).yellow());
            let line = match instr {
                Instr::Push(v) => emit1("PUSH", &v.to_string()),
                Instr::Load(idx) => {
                    let sym_name = table.get_by_index(*idx).map(|s| s.name()).unwrap_or("???");
                    emit1("LOAD", &sym_name.blue())
                }
                Instr::Neg => emit("NEG"),
                Instr::Add => emit("ADD"),
                Instr::Sub => emit("SUB"),
                Instr::Mul => emit("MUL"),
                Instr::Div => emit("DIV"),
                Instr::Pow => emit("POW"),
                Instr::Fact => emit("FACT"),
                Instr::Call(idx, argc) => {
                    let sym_name = table.get_by_index(*idx).map(|s| s.name()).unwrap_or("???");
                    format!(
                        "{} {} args: {}",
                        emit("CALL"),
                        sym_name.cyan(),
                        argc.to_string().bright_blue()
                    )
                }
                Instr::Equal => emit("EQ"),
                Instr::NotEqual => emit("NEQ"),
                Instr::Less => emit("LT"),
                Instr::LessEqual => emit("LTE"),
                Instr::Greater => emit("GT"),
                Instr::GreaterEqual => emit("GTE"),
            };
            let _ = writeln!(out, "{}", line);
        }
        out
    }
}
