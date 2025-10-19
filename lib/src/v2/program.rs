//! Type-state program implementation with improved architecture.

use super::ast::{BinOp, Expr, ExprKind, UnOp};
use super::error::{CompileError, LinkError, ProgramError};
use super::metadata::{SymbolKind, SymbolMetadata};
use super::parser::Parser;
use super::sema;
use super::source::Source;
use crate::ir::Instr;
use crate::symbol::{Symbol, SymTable};
use crate::vm::{Vm, VmError};
use colored::Colorize;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

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
        let (binary, _): (BinaryFormat, _) = bincode::serde::decode_from_slice(data, config)
            .map_err(|e| ProgramError::DeserializationError(e.to_string()))?;

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
    pub fn compile(self) -> Result<Program<Compiled>, ProgramError> {
        let mut ast = self.state.ast;

        // Step 1: Discover all symbols used in the AST
        let symbols = sema::discover_symbols(&ast);

        // Step 2: Annotate AST with indices (position in symbols vec)
        sema::annotate_ast_with_indices(&mut ast, &symbols)
            .map_err(|e| CompileError::SemanticError(e))?;

        // Step 3: Generate bytecode
        let bytecode = Self::generate_bytecode(&ast)?;

        Ok(Program {
            state: Compiled {
                origin: ProgramOrigin::Source(self.state.source),
                bytecode,
                symbols,
            },
        })
    }

    /// Generates bytecode from an annotated AST.
    fn generate_bytecode(ast: &Expr) -> Result<Vec<Instr>, CompileError> {
        let mut bytecode = Vec::new();
        Self::emit_instr(ast, &mut bytecode)?;
        Ok(bytecode)
    }

    fn emit_instr(expr: &Expr, bytecode: &mut Vec<Instr>) -> Result<(), CompileError> {
        match &expr.kind {
            ExprKind::Literal(v) => {
                bytecode.push(Instr::Push(*v));
            }
            ExprKind::Ident { name, sym_index } => {
                if let Some(idx) = sym_index {
                    bytecode.push(Instr::Load(*idx));
                } else {
                    return Err(CompileError::CodeGenError(format!(
                        "Undefined symbol: {}",
                        name
                    )));
                }
            }
            ExprKind::Unary { op, expr } => {
                Self::emit_instr(expr, bytecode)?;
                match op {
                    UnOp::Neg => bytecode.push(Instr::Neg),
                    UnOp::Fact => bytecode.push(Instr::Fact),
                }
            }
            ExprKind::Binary { op, left, right } => {
                Self::emit_instr(left, bytecode)?;
                Self::emit_instr(right, bytecode)?;
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
            ExprKind::Call {
                name,
                args,
                sym_index,
            } => {
                if let Some(idx) = sym_index {
                    for arg in args {
                        Self::emit_instr(arg, bytecode)?;
                    }
                    bytecode.push(Instr::Call(*idx, args.len()));
                } else {
                    return Err(CompileError::CodeGenError(format!(
                        "Undefined function: {}",
                        name
                    )));
                }
            }
        }
        Ok(())
    }
}

// ============================================================================
// Program<Compiled> - After compilation or deserialization
// ============================================================================

impl Program<Compiled> {
    /// Links the bytecode with a symbol table, validating and remapping indices.
    pub fn link(mut self, table: SymTable) -> Result<Program<Linked>, ProgramError> {
        // Build remapping table: metadata_index → symtable_index
        let mut remap = Vec::with_capacity(self.state.symbols.len());

        for metadata in &self.state.symbols {
            // Look up symbol in provided table
            let (new_idx, symbol) = table
                .get_with_index(&metadata.name)
                .ok_or_else(|| LinkError::MissingSymbol {
                    name: metadata.name.clone(),
                })?;

            // Validate kind matches
            Self::validate_symbol_kind(metadata, symbol)?;

            remap.push(new_idx);
        }

        // Rewrite all indices in bytecode
        for instr in &mut self.state.bytecode {
            match instr {
                Instr::Load(idx) => *idx = remap[*idx],
                Instr::Call(idx, _) => *idx = remap[*idx],
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
                        name: metadata.name.clone(),
                        expected: format!("function(arity={}, variadic={})", arity, variadic),
                        found: format!("function(arity={}, variadic={})", args, v),
                    })
                }
            }
            (SymbolKind::Const, Symbol::Func { .. }) => Err(LinkError::TypeMismatch {
                name: metadata.name.clone(),
                expected: "constant".to_string(),
                found: "function".to_string(),
            }),
            (SymbolKind::Func { .. }, Symbol::Const { .. }) => Err(LinkError::TypeMismatch {
                name: metadata.name.clone(),
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
        let mut reverse_remap = HashMap::new();
        let mut symbols = Vec::new();

        for (metadata_idx, symtable_idx) in used_indices.iter().enumerate() {
            let symbol = self
                .state
                .symtable
                .get_by_index(*symtable_idx)
                .ok_or(ProgramError::InvalidSymbolIndex(*symtable_idx))?;

            let kind = match symbol {
                Symbol::Const { .. } => SymbolKind::Const,
                Symbol::Func { args, variadic, .. } => SymbolKind::Func {
                    arity: *args,
                    variadic: *variadic,
                },
            };

            symbols.push(SymbolMetadata {
                name: symbol.name().to_string(),
                kind,
            });

            reverse_remap.insert(*symtable_idx, metadata_idx);
        }

        // Step 3: Rewrite bytecode to use metadata indices
        let mut bytecode = self.state.bytecode.clone();
        for instr in &mut bytecode {
            match instr {
                Instr::Load(idx) => *idx = reverse_remap[idx],
                Instr::Call(idx, _) => *idx = reverse_remap[idx],
                _ => {}
            }
        }

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
