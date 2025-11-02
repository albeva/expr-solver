//! Linker for resolving symbols and linking bytecode with a symbol table.
//!
//! The [`Linker`] is responsible for the third stage of compilation:
//! resolving symbol references and producing executable bytecode.
//!
//! # Process
//!
//! 1. Validate all symbol references exist in the symbol table
//! 2. Verify symbol types match their usage (constant vs function, arity checking)
//! 3. Resolve symbol indices (map compile-time indices to runtime indices)
//! 4. Rewrite bytecode to use resolved indices
//! 5. Return linked bytecode and symbol table ready for execution
//!
//! # Example
//!
//! ```ignore
//! use expr_solver::linker::Linker;
//! use expr_solver::SymTable;
//!
//! // Assume we have bytecode and symbols from IrBuilder
//! let linker = Linker::new(bytecode, symbols, SymTable::stdlib());
//! let (linked_bytecode, symtable) = linker.link().unwrap();
//! // linked_bytecode is ready to execute on the VM
//! ```

use crate::error::LinkerError;
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
                let idx = self.symtable.symbols().count();
                match &metadata.kind {
                    SymbolKind::Const => {
                        self.symtable
                            .add_const(metadata.name.to_string(), num!(0), true)?;
                    }
                    SymbolKind::LocalFunc { offset, params, .. } => {
                        self.symtable.add_local_func(
                            metadata.name.to_string(),
                            params.clone(),
                            *offset,
                        )?;
                    }
                    _ => {
                        unreachable!()
                    }
                };
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
    fn validate_symbol_kind(metadata: &SymbolMetadata, symbol: &Symbol) -> Result<(), LinkerError> {
        match (&metadata.kind, symbol) {
            (SymbolKind::Const, Symbol::Const { .. }) => Ok(()),
            (
                SymbolKind::GlobalFunc { arity, .. },
                Symbol::Func {
                    args: min_args,
                    variadic,
                    ..
                },
            ) => Self::valildate_args(&metadata, *arity, *min_args, *variadic),
            (SymbolKind::LocalFunc { arity, .. }, Symbol::LocalFunc { params, .. }) => {
                Self::valildate_args(&metadata, *arity, params.len(), false)
            }
            (SymbolKind::Const, Symbol::Func { .. } | Symbol::LocalFunc { .. }) => {
                Err(LinkerError::TypeMismatch {
                    name: metadata.name.to_string(),
                    expected: "constant".to_string(),
                    found: "function".to_string(),
                })
            }
            (
                SymbolKind::GlobalFunc { .. } | SymbolKind::LocalFunc { .. },
                Symbol::Const { .. },
            ) => Err(LinkerError::TypeMismatch {
                name: metadata.name.to_string(),
                expected: "function".to_string(),
                found: "constant".to_string(),
            }),
            _ => unreachable!(),
        }
    }

    fn valildate_args(
        metadata: &SymbolMetadata,
        arity: usize,
        min_args: usize,
        variadic: bool,
    ) -> Result<(), LinkerError> {
        // Check if the call is valid:
        // - For non-variadic: arity must match exactly
        // - For variadic: arity must be >= min_args
        let valid = if variadic {
            arity >= min_args
        } else {
            arity == min_args
        };

        if valid {
            Ok(())
        } else {
            let expected_msg = if variadic {
                format!("at least {} arguments", min_args)
            } else {
                format!("exactly {} arguments", min_args)
            };
            Err(LinkerError::TypeMismatch {
                name: metadata.name.to_string(),
                expected: expected_msg,
                found: format!("{} arguments provided", arity),
            })
        }
    }
}
