//! Symbol metadata for bytecode validation and linking.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Metadata about a symbol required by compiled bytecode.
///
/// This is used to validate and remap symbol indices when linking
/// bytecode with a symbol table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolMetadata {
    /// The name of the symbol
    pub name: Cow<'static, str>,
    /// The kind and requirements of the symbol
    pub kind: SymbolKind,
    /// The resolved index in the linked symbol table (None until linked)
    #[serde(skip)]
    pub index: Option<usize>,
}

/// The kind of symbol (constant or function) with its requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    /// A constant value
    Const,
    /// A function with specified arity
    Func {
        /// Minimum number of arguments
        arity: usize,
        /// Whether the function accepts additional arguments
        variadic: bool,
    },
}

impl SymbolMetadata {
    /// Creates metadata for a constant symbol.
    pub fn constant(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            kind: SymbolKind::Const,
            index: None,
        }
    }

    /// Creates metadata for a function symbol.
    pub fn function(name: impl Into<Cow<'static, str>>, arity: usize, variadic: bool) -> Self {
        Self {
            name: name.into(),
            kind: SymbolKind::Func { arity, variadic },
            index: None,
        }
    }
}
