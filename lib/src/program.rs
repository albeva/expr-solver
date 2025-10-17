use crate::ir::Instr;
use crate::symbol::{SymTable, Symbol};
use bincode::config;
use colored::Colorize;
use rust_decimal::Decimal;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Expression parsing and evaluation errors.
#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Compilation error: {0}")]
    CompileError(String),
    #[error("Decoding error: {0}")]
    DecodingError(#[from] bincode::error::DecodeError),
    #[error("incompatible program version: expected {0}, got {1}")]
    IncompatibleVersions(String, String),
    #[error("Unknown symbol: {0}")]
    UnknownSymbol(String),
    #[error("Symbol '{0}' is not a {1}")]
    SymbolKindMismatch(String, String),
    #[error("Function '{0}' incorrect arity")]
    InvalidFuncArity(String),
    #[error("Corrupted instruction: {0}")]
    CorrupedInstruction(String),
}

/// Executable program is still a sequence of `Instr<'sym>` referencing symbols
/// inside a provided `SymTable`.
#[derive(Default)]
pub struct Program<'sym> {
    pub version: String,
    pub code: Vec<Instr<'sym>>,
}

/// A compact, fully-owned form of a program that can be serialized without lifetimes.
/// Symbols are stored by name and kind; instructions reference symbols by index.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Binary {
    version: String,
    symbols: Vec<BinarySymbol>,
    code: Vec<BinaryInstr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum BinarySymbol {
    /// Named constant
    Const(String),
    /// Named function
    Func {
        name: String,
        args: usize,
        variadic: bool,
    },
}

impl BinarySymbol {
    fn name(&self) -> String {
        match self {
            BinarySymbol::Const(name) => name.clone(),
            BinarySymbol::Func { name, .. } => name.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum BinaryInstr {
    Push(Decimal),
    Load(u32), // index into `symbols`
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Fact,
    Call(u32, usize), // index into `symbols` and argument count
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl<'sym> Program<'sym> {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            code: Vec::new(),
        }
    }

    pub fn compile(&self) -> Result<Vec<u8>, ProgramError> {
        let binary = self.to_binary();
        let config = config::standard();
        bincode::serde::encode_to_vec(&binary, config)
            .map_err(|err| ProgramError::CompileError(format!("failed to encode program: {}", err)))
    }

    pub fn load(data: &[u8], table: &'sym SymTable) -> Result<Program<'sym>, ProgramError> {
        let config = config::standard();
        let (decoded, _): (Binary, usize) = bincode::serde::decode_from_slice(&data, config)
            .map_err(ProgramError::DecodingError)?;

        Self::validate_version(&decoded.version)?;

        let get_sym = |bin_sym: &BinarySymbol| -> Result<&'sym Symbol, ProgramError> {
            let name = bin_sym.name();
            table.get(&name).ok_or(ProgramError::UnknownSymbol(name))
        };

        let mut program = Program::new();
        program.version = decoded.version.clone();

        for instr in &decoded.code {
            match instr {
                BinaryInstr::Push(v) => {
                    program.code.push(Instr::Push(*v));
                }
                BinaryInstr::Load(idx) => {
                    let bin_sym = &decoded.symbols[*idx as usize];
                    let sym = get_sym(&bin_sym)?;
                    match bin_sym {
                        BinarySymbol::Const(_) => {
                            if !matches!(sym, Symbol::Const { .. }) {
                                return Err(ProgramError::SymbolKindMismatch(
                                    sym.name().to_string(),
                                    "constant".to_string(),
                                ));
                            }
                        }
                        _ => {
                            return Err(ProgramError::CorrupedInstruction("LOAD".to_string()));
                        }
                    }
                    program.code.push(Instr::Load(sym))
                }
                BinaryInstr::Neg => program.code.push(Instr::Neg),
                BinaryInstr::Add => program.code.push(Instr::Add),
                BinaryInstr::Sub => program.code.push(Instr::Sub),
                BinaryInstr::Mul => program.code.push(Instr::Mul),
                BinaryInstr::Div => program.code.push(Instr::Div),
                BinaryInstr::Pow => program.code.push(Instr::Pow),
                BinaryInstr::Fact => program.code.push(Instr::Fact),
                BinaryInstr::Call(idx, argc) => {
                    let bin_sym = &decoded.symbols[*idx as usize];
                    let sym = get_sym(&bin_sym)?;
                    if !matches!(sym, Symbol::Func { .. }) {
                        return Err(ProgramError::SymbolKindMismatch(
                            sym.name().to_string(),
                            "function".to_string(),
                        ));
                    }
                    program.code.push(Instr::Call(sym, *argc));
                }
                // Comparison operators
                BinaryInstr::Equal => program.code.push(Instr::Equal),
                BinaryInstr::NotEqual => program.code.push(Instr::NotEqual),
                BinaryInstr::Less => program.code.push(Instr::Less),
                BinaryInstr::LessEqual => program.code.push(Instr::LessEqual),
                BinaryInstr::Greater => program.code.push(Instr::Greater),
                BinaryInstr::GreaterEqual => program.code.push(Instr::GreaterEqual),
            }
        }

        Ok(program)
    }

    fn validate_version(version: &String) -> Result<(), ProgramError> {
        let current_version = env!("CARGO_PKG_VERSION");
        if version != current_version {
            return Err(ProgramError::IncompatibleVersions(
                current_version.to_string(),
                version.clone(),
            ));
        }
        Ok(())
    }

    pub fn get_assembly(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        out += &format!("; VERSION {}\n", self.version).bright_black().to_string();

        let emit = |mnemonic: &str| -> String { format!("{}", mnemonic.magenta()) };
        let emit1 = |mnemonic: &str, op: &str| -> String {
            format!("{} {}", mnemonic.magenta(), op.green())
        };

        for (i, instr) in self.code.iter().enumerate() {
            let _ = write!(out, "{} ", format!("{:04X}", i).yellow());
            let line = match instr {
                Instr::Push(v) => emit1("PUSH", &v.to_string().green()),
                Instr::Load(sym) => emit1("LOAD", &sym.name().blue()),
                Instr::Neg => emit("NEG"),
                Instr::Add => emit("ADD"),
                Instr::Sub => emit("SUB"),
                Instr::Mul => emit("MUL"),
                Instr::Div => emit("DIV"),
                Instr::Pow => emit("POW"),
                Instr::Fact => emit("FACT"),
                Instr::Call(sym, argc) => format!(
                    "{} {} args: {}",
                    emit("CALL"),
                    sym.name().cyan(),
                    argc.to_string().bright_blue()
                ),
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

    fn to_binary(&self) -> Binary {
        let mut map: HashMap<String, u32> = HashMap::new();
        let mut binary = Binary {
            version: self.version.clone(),
            symbols: Vec::new(),
            code: Vec::new(),
        };

        let mut get_index = |sym: &'sym Symbol| -> u32 {
            map.get(sym.name()).map(|val| *val).unwrap_or_else(|| {
                let i = binary.symbols.len() as u32;
                map.insert(sym.name().to_string(), i);
                binary.symbols.push(match sym {
                    Symbol::Const { .. } => BinarySymbol::Const(sym.name().to_string()),
                    Symbol::Func { args, variadic, .. } => BinarySymbol::Func {
                        name: sym.name().to_string(),
                        args: *args,
                        variadic: *variadic,
                    },
                });
                i
            })
        };

        for instr in &self.code {
            match instr {
                Instr::Push(v) => {
                    binary.code.push(BinaryInstr::Push(*v));
                }
                Instr::Load(sym) => {
                    let idx = get_index(sym);
                    binary.code.push(BinaryInstr::Load(idx));
                }
                Instr::Neg => {
                    binary.code.push(BinaryInstr::Neg);
                }
                Instr::Add => {
                    binary.code.push(BinaryInstr::Add);
                }
                Instr::Sub => {
                    binary.code.push(BinaryInstr::Sub);
                }
                Instr::Mul => {
                    binary.code.push(BinaryInstr::Mul);
                }
                Instr::Div => {
                    binary.code.push(BinaryInstr::Div);
                }
                Instr::Pow => {
                    binary.code.push(BinaryInstr::Pow);
                }
                Instr::Fact => {
                    binary.code.push(BinaryInstr::Fact);
                }
                Instr::Call(sym, argc) => {
                    let idx = get_index(sym);
                    binary.code.push(BinaryInstr::Call(idx, *argc));
                }
                Instr::Equal => {
                    binary.code.push(BinaryInstr::Equal);
                }
                Instr::NotEqual => {
                    binary.code.push(BinaryInstr::NotEqual);
                }
                Instr::Less => {
                    binary.code.push(BinaryInstr::Less);
                }
                Instr::LessEqual => {
                    binary.code.push(BinaryInstr::LessEqual);
                }
                Instr::Greater => {
                    binary.code.push(BinaryInstr::Greater);
                }
                Instr::GreaterEqual => {
                    binary.code.push(BinaryInstr::GreaterEqual);
                }
            }
        }

        binary
    }
}
