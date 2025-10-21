//! Linker for resolving symbols and linking bytecode with a symbol table.

use crate::error::{LinkError, LinkerError};
use crate::ir::Instr;
use crate::metadata::{SymbolKind, SymbolMetadata};
use crate::num;
use crate::symbol::Symbol;
use crate::symtable::SymTable;

/// Linker for resolving symbols and linking bytecode.
///
/// The linker takes compiled bytecode with unresolved symbol references
/// and a symbol table, validates all symbols exist and match their expected
/// types, then produces linked bytecode ready for execution.
#[derive(Debug)]
pub struct Linker {
    bytecode: Vec<Instr>,
    symbols: Vec<SymbolMetadata>,
    symtable: SymTable,
}

impl Linker {
    /// Creates a new linker.
    ///
    /// # Parameters
    /// - `bytecode`: The compiled bytecode with unresolved symbol indices
    /// - `symbols`: Symbol metadata collected during compilation
    /// - `symtable`: The symbol table to link against
    pub fn new(bytecode: Vec<Instr>, symbols: Vec<SymbolMetadata>, symtable: SymTable) -> Self {
        Self {
            bytecode,
            symbols,
            symtable,
        }
    }

    /// Links the bytecode with the symbol table.
    ///
    /// This validates that all symbols exist and match their expected types,
    /// resolves symbol indices, and rewrites the bytecode to use the resolved indices.
    ///
    /// Returns the linked bytecode and symbol table ready for execution.
    pub fn link(mut self) -> Result<(Vec<Instr>, SymTable), LinkerError> {
        self.resolve_symbols()?;
        self.rewrite_bytecode();
        Ok((self.bytecode, self.symtable))
    }

    /// Resolves all symbols and fills in their resolved indices.
    fn resolve_symbols(&mut self) -> Result<(), LinkerError> {
        for metadata in &mut self.symbols {
            let resolved_idx = if metadata.local {
                // Add local symbol to the symbol table
                let idx = self.symtable.symbols().count();
                self.symtable
                    .add_const(metadata.name.to_string(), num!(0), true)?;
                idx
            } else {
                // Resolve external symbol
                let (idx, symbol) = self.symtable.get_with_index(&metadata.name)?;
                Self::validate_symbol_kind(metadata, symbol)?;
                idx
            };

            // Store resolved index in metadata
            metadata.index = Some(resolved_idx);
        }
        Ok(())
    }

    /// Rewrites bytecode to use resolved symbol indices.
    fn rewrite_bytecode(&mut self) {
        for instr in &mut self.bytecode {
            match instr {
                Instr::Load(idx) | Instr::Store(idx) | Instr::Call(idx, _) => {
                    *idx = self.symbols[*idx]
                        .index
                        .expect("Symbol should have been resolved during linking");
                }
                _ => {}
            }
        }
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
}
