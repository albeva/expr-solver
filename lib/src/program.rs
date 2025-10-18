use crate::ir::Instr;
use bincode::config;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Current version of the program format
const PROGRAM_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Expression parsing and evaluation errors.
#[derive(Error, Debug)]
pub enum ProgramError {
    #[error("Compilation error: {0}")]
    CompileError(String),
    #[error("Decoding error: {0}")]
    DecodingError(#[from] bincode::error::DecodeError),
    #[error("incompatible program version: expected {0}, got {1}")]
    IncompatibleVersions(String, String),
}

/// Executable program containing bytecode instructions.
///
/// Programs reference symbols by index into a [`SymTable`] and can be serialized
/// to binary format for storage or transmission.
#[derive(Default)]
pub struct Program {
    pub version: String,
    pub code: Vec<Instr>,
}

/// Binary format for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Binary {
    version: String,
    code: Vec<Instr>,
}

impl Program {
    /// Creates a new empty program.
    pub fn new() -> Self {
        Self {
            version: PROGRAM_VERSION.to_string(),
            code: Vec::new(),
        }
    }

    /// Compiles the program to binary format for serialization.
    pub fn compile(&self) -> Result<Vec<u8>, ProgramError> {
        let binary = Binary {
            version: self.version.clone(),
            code: self.code.clone(),
        };
        let config = config::standard();
        bincode::serde::encode_to_vec(&binary, config)
            .map_err(|err| ProgramError::CompileError(format!("failed to encode program: {}", err)))
    }

    /// Loads a program from binary data.
    ///
    /// The binary data must have been created with [`compile`](Self::compile).
    pub fn load(data: &[u8]) -> Result<Program, ProgramError> {
        let config = config::standard();
        let (decoded, _): (Binary, usize) = bincode::serde::decode_from_slice(&data, config)
            .map_err(ProgramError::DecodingError)?;

        Self::validate_version(&decoded.version)?;

        Ok(Program {
            version: decoded.version,
            code: decoded.code,
        })
    }

    fn validate_version(version: &String) -> Result<(), ProgramError> {
        if version != PROGRAM_VERSION {
            return Err(ProgramError::IncompatibleVersions(
                PROGRAM_VERSION.to_string(),
                version.clone(),
            ));
        }
        Ok(())
    }

    /// Returns a human-readable assembly representation of the program.
    pub fn get_assembly(&self, table: &crate::symbol::SymTable) -> String {
        use std::fmt::Write as _;

        let mut out = String::new();
        out += &format!("; VERSION {}\n", self.version)
            .bright_black()
            .to_string();

        let emit = |mnemonic: &str| -> String { format!("{}", mnemonic.magenta()) };
        let emit1 = |mnemonic: &str, op: &str| -> String {
            format!("{} {}", mnemonic.magenta(), op.green())
        };

        for (i, instr) in self.code.iter().enumerate() {
            let _ = write!(out, "{} ", format!("{:04X}", i).yellow());
            let line = match instr {
                Instr::Push(v) => emit1("PUSH", &v.to_string().green()),
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
