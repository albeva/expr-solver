//! Semantic analyzer for v2 with direct symbol collection.

use super::ast::*;
use super::error::SemanticError;
use super::metadata::{SymbolKind, SymbolMetadata};
use crate::span::Span;
use crate::symbol::{Symbol, SymTable};

/// Semantic analyzer for type checking and symbol resolution.
pub struct Sema<'sym> {
    table: &'sym SymTable,
}

impl<'sym> Sema<'sym> {
    /// Creates a new semantic analyzer with the given symbol table.
    pub fn new(table: &'sym SymTable) -> Self {
        Self { table }
    }

    /// Analyzes an AST expression, resolving symbols and checking types.
    pub fn visit(&mut self, ast: &mut Expr) -> Result<(), SemanticError> {
        match &mut ast.kind {
            ExprKind::Literal(_) => Ok(()),
            ExprKind::Ident { name, sym_index } => self.visit_ident(name, sym_index, ast.span),
            ExprKind::Unary { op: _, expr } => self.visit_unary(expr),
            ExprKind::Binary { op: _, left, right } => self.visit_binary(left, right),
            ExprKind::Call {
                name,
                args,
                sym_index,
            } => self.visit_call(name, args, sym_index, ast.span),
        }
    }

    fn visit_ident(
        &mut self,
        name: &str,
        sym_index: &mut Option<usize>,
        span: Span,
    ) -> Result<(), SemanticError> {
        let (idx, sym) = self.get_symbol_with_index(name, span)?;

        let Symbol::Const { .. } = sym else {
            return Err(SemanticError::SymbolIsNotAConstant {
                name: name.to_string(),
                span,
            });
        };

        *sym_index = Some(idx);
        Ok(())
    }

    fn visit_unary(&mut self, expr: &mut Expr) -> Result<(), SemanticError> {
        self.visit(expr)
    }

    fn visit_binary(&mut self, left: &mut Expr, right: &mut Expr) -> Result<(), SemanticError> {
        self.visit(left)?;
        self.visit(right)
    }

    fn visit_call(
        &mut self,
        name: &str,
        args: &mut Vec<Expr>,
        sym_index: &mut Option<usize>,
        span: Span,
    ) -> Result<(), SemanticError> {
        let sym_span = Span::new(span.start, span.start + name.len());
        let (idx, sym) = self.get_symbol_with_index(name, sym_span)?;

        let Symbol::Func {
            args: min_args,
            variadic,
            ..
        } = sym
        else {
            return Err(SemanticError::SymbolIsNotAFunction {
                name: name.to_string(),
                span: sym_span,
            });
        };

        self.validate_arity(name, args.len(), *min_args, *variadic, span)?;
        self.analyse_arguments(args)?;

        *sym_index = Some(idx);
        Ok(())
    }

    fn validate_arity(
        &self,
        name: &str,
        args: usize,
        min_args: usize,
        variadic: bool,
        span: Span,
    ) -> Result<(), SemanticError> {
        if args == min_args || variadic && args > min_args {
            return Ok(());
        }
        if variadic {
            Err(SemanticError::InsufficientArguments {
                name: name.to_string(),
                expected: min_args,
                actual: args,
                span,
            })
        } else {
            Err(SemanticError::ArgumentCountMismatch {
                name: name.to_string(),
                expected: min_args,
                actual: args,
                span,
            })
        }
    }

    fn analyse_arguments(&mut self, args: &mut [Expr]) -> Result<(), SemanticError> {
        args.iter_mut().try_for_each(|a| self.visit(a))
    }

    fn get_symbol_with_index(
        &self,
        name: &str,
        span: Span,
    ) -> Result<(usize, &Symbol), SemanticError> {
        self.table
            .get_with_index(name)
            .ok_or_else(|| SemanticError::UndefinedSymbol {
                name: name.to_string(),
                span,
            })
    }
}

/// Discovers symbols from an AST and returns them as metadata vector.
///
/// This scans the AST and collects all unique symbols, creating metadata
/// for each with the appropriate kind (Const or Func). Symbols are returned
/// in the order they were first encountered.
pub fn discover_symbols(ast: &Expr) -> Vec<SymbolMetadata> {
    let mut symbols = Vec::new();
    collect_symbols(ast, &mut symbols);
    symbols
}

fn collect_symbols(expr: &Expr, symbols: &mut Vec<SymbolMetadata>) {
    match &expr.kind {
        ExprKind::Literal(_) => {}
        ExprKind::Ident { name, .. } => {
            // Add if not already present
            if !symbols.iter().any(|s| s.name == *name) {
                symbols.push(SymbolMetadata {
                    name: name.clone(),
                    kind: SymbolKind::Const,
                });
            }
        }
        ExprKind::Unary { expr, .. } => {
            collect_symbols(expr, symbols);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_symbols(left, symbols);
            collect_symbols(right, symbols);
        }
        ExprKind::Call { name, args, .. } => {
            // For functions, we need to determine arity from usage
            // We'll take the first occurrence's arity
            if !symbols.iter().any(|s| s.name == *name) {
                symbols.push(SymbolMetadata {
                    name: name.clone(),
                    kind: SymbolKind::Func {
                        arity: args.len(),
                        variadic: false, // Will be validated during linking
                    },
                });
            }
            for arg in args {
                collect_symbols(arg, symbols);
            }
        }
    }
}

/// Annotates an AST with symbol indices based on a metadata vector.
///
/// This is used during compilation to fill in sym_index fields in the AST
/// based on positions in the metadata vector.
pub fn annotate_ast_with_indices(ast: &mut Expr, symbols: &[SymbolMetadata]) -> Result<(), SemanticError> {
    match &mut ast.kind {
        ExprKind::Literal(_) => Ok(()),
        ExprKind::Ident { name, sym_index } => {
            let idx = symbols
                .iter()
                .position(|s| s.name == *name)
                .ok_or_else(|| SemanticError::UndefinedSymbol {
                    name: name.clone(),
                    span: ast.span,
                })?;
            *sym_index = Some(idx);
            Ok(())
        }
        ExprKind::Unary { expr, .. } => annotate_ast_with_indices(expr, symbols),
        ExprKind::Binary { left, right, .. } => {
            annotate_ast_with_indices(left, symbols)?;
            annotate_ast_with_indices(right, symbols)
        }
        ExprKind::Call {
            name,
            args,
            sym_index,
        } => {
            let idx = symbols
                .iter()
                .position(|s| s.name == *name)
                .ok_or_else(|| SemanticError::UndefinedSymbol {
                    name: name.clone(),
                    span: ast.span,
                })?;
            *sym_index = Some(idx);
            for arg in args {
                annotate_ast_with_indices(arg, symbols)?;
            }
            Ok(())
        }
    }
}
