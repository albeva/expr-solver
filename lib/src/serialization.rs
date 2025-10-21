//! Binary serialization and deserialization for compiled programs.
//!
//! This module provides the [`Serializer`] type for serializing and deserializing
//! compiled bytecode with symbol metadata.
//!
//! # Examples
//!
//! Using the OO API for serialization:
//!
//! ```
//! # #[cfg(feature = "serialization")]
//! # {
//! use expr_solver::{Program, SymTable, Serializer};
//!
//! // Create and link a program
//! let program = Program::new_from_source("2 + 3 * 4").unwrap();
//! let linked = program.link(SymTable::stdlib()).unwrap();
//!
//! // Serialize using the OO API
//! let serializer = Serializer::from_program(&linked);
//! let bytes = serializer.to_bytes().unwrap();
//!
//! // Or save directly to a file
//! // serializer.to_file("expr.bin").unwrap();
//!
//! // Deserialize using the OO API
//! let serializer = Serializer::from_bytes(&bytes).unwrap();
//! assert_eq!(serializer.version(), env!("CARGO_PKG_VERSION"));
//! assert!(serializer.bytecode().len() > 0);
//! # }
//! ```

use crate::error::ProgramError;
use crate::ir::Instr;
use crate::metadata::SymbolMetadata;
use crate::program::{Linked, Program};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Current version of the program format
const PROGRAM_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Binary format for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BinaryFormat {
    version: String,
    bytecode: Vec<Instr>,
    symbols: Vec<SymbolMetadata>,
}

/// Serializer for compiled programs.
///
/// This type provides an object-oriented API for program serialization,
/// keeping serialization concerns separate from the main Program type.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "serialization")]
/// # {
/// use expr_solver::{Program, SymTable, Serializer};
///
/// // Create and link a program
/// let program = Program::new_from_source("2 + 3").unwrap();
/// let linked = program.link(SymTable::stdlib()).unwrap();
///
/// // Serialize
/// let serializer = Serializer::from_program(&linked);
/// let bytes = serializer.to_bytes().unwrap();
///
/// // Deserialize
/// let serializer = Serializer::from_bytes(&bytes).unwrap();
/// let (version, bytecode, symbols) = serializer.into_parts();
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Serializer {
    version: String,
    bytecode: Vec<Instr>,
    symbols: Vec<SymbolMetadata>,
}

impl Serializer {
    /// Creates program data from a linked program.
    ///
    /// This involves reverse-mapping bytecode indices back to metadata indices.
    pub fn from_program(program: &Program<Linked>) -> Self {
        let mut reverse_map = HashMap::new();
        let mut symbols = Vec::new();

        // Helper closure to get or create metadata index
        let mut get_or_create_metadata = |idx: usize| -> usize {
            if let Some(&existing) = reverse_map.get(&idx) {
                existing
            } else {
                let symbol = program
                    .symtable()
                    .get_by_index(idx)
                    .expect("symbol index must be valid after linking");

                let new_idx = symbols.len();
                symbols.push(symbol.into());
                reverse_map.insert(idx, new_idx);
                new_idx
            }
        };

        // Single pass: build symbol mapping and rewrite bytecode
        let bytecode: Vec<Instr> = program
            .bytecode()
            .iter()
            .map(|instr| match instr {
                Instr::Load(idx) => Instr::Load(get_or_create_metadata(*idx)),
                Instr::Store(idx) => Instr::Store(get_or_create_metadata(*idx)),
                Instr::Call(idx, argc) => Instr::Call(get_or_create_metadata(*idx), *argc),
                other => other.clone(),
            })
            .collect();

        Serializer {
            version: program.version().to_string(),
            bytecode,
            symbols,
        }
    }

    /// Serializes the program to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ProgramError> {
        let binary = BinaryFormat {
            version: self.version.clone(),
            bytecode: self.bytecode.clone(),
            symbols: self.symbols.clone(),
        };

        let config = bincode::config::standard();
        Ok(bincode::serde::encode_to_vec(&binary, config)?)
    }

    /// Serializes the program and saves it to a file.
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), ProgramError> {
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Deserializes a program from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, ProgramError> {
        let config = bincode::config::standard();
        let (binary, _): (BinaryFormat, _) = bincode::serde::decode_from_slice(data, config)?;

        // Validate version
        if binary.version != PROGRAM_VERSION {
            return Err(ProgramError::IncompatibleVersion {
                expected: PROGRAM_VERSION.to_string(),
                found: binary.version,
            });
        }

        Ok(Serializer {
            version: binary.version,
            bytecode: binary.bytecode,
            symbols: binary.symbols,
        })
    }

    /// Deserializes a program from a file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ProgramError> {
        let data = std::fs::read(path)?;
        Self::from_bytes(&data)
    }

    /// Consumes the serializer and returns the program parts.
    pub fn into_parts(self) -> (String, Vec<Instr>, Vec<SymbolMetadata>) {
        (self.version, self.bytecode, self.symbols)
    }

    /// Returns a reference to the bytecode.
    pub fn bytecode(&self) -> &[Instr] {
        &self.bytecode
    }

    /// Returns a reference to the symbol metadata.
    pub fn symbols(&self) -> &[SymbolMetadata] {
        &self.symbols
    }

    /// Returns the version string.
    pub fn version(&self) -> &str {
        &self.version
    }
}
