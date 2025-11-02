//! Symbol metadata for bytecode validation and linking.

use crate::symbol::Symbol;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Metadata about a symbol required by compiled bytecode.
///
/// This is used to validate and remap symbol indices when linking
/// bytecode with a symbol table.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct SymbolMetadata {
    /// The name of the symbol
    pub name: Cow<'static, str>,
    /// The kind and requirements of the symbol
    pub kind: SymbolKind,
    /// Local or global?
    pub local: bool,

    /// The resolved index in the linked symbol table (None until linked)
    #[cfg_attr(feature = "serialization", serde(skip))]
    pub index: Option<usize>,
}

/// The kind of symbol (constant or function) with its requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum SymbolKind {
    /// A constant value
    Const,
    /// A function with specified arity
    Func {
        /// Minimum number of arguments
        arity: usize,
        /// Whether the function accepts additional arguments
        variadic: bool,
        /// Parameter names
        params: Option<Vec<Cow<'static, str>>>,
    },
}

impl From<&Symbol> for SymbolKind {
    fn from(symbol: &Symbol) -> Self {
        match symbol {
            Symbol::Const { .. } => SymbolKind::Const,
            Symbol::Func { args, variadic, .. } => SymbolKind::Func {
                arity: *args,
                variadic: *variadic,
                params: None,
            },
        }
    }
}

impl From<&Symbol> for SymbolMetadata {
    fn from(symbol: &Symbol) -> Self {
        SymbolMetadata {
            name: symbol.name().to_string().into(),
            kind: symbol.into(),
            local: symbol.is_local(),
            index: None,
        }
    }
}
