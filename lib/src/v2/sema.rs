//! Semantic analyzer for v2.
//!
//! In v2, semantic analysis happens during linking (validation) rather than
//! during compilation. The Sema struct provides validation methods.

use super::ast::*;
use super::error::SemanticError;
use crate::span::Span;
use crate::symbol::{Symbol, SymTable};

/// Semantic analyzer for type checking and symbol resolution.
///
/// Used during linking to validate symbols against a symbol table.
pub struct Sema<'sym> {
    table: &'sym SymTable,
}

impl<'sym> Sema<'sym> {
    /// Creates a new semantic analyzer with the given symbol table.
    pub fn new(table: &'sym SymTable) -> Self {
        Self { table }
    }

    /// Analyzes an AST expression, validating symbols and types.
    pub fn validate(&mut self, ast: &Expr) -> Result<(), SemanticError> {
        self.visit(ast)
    }

    fn visit(&mut self, ast: &Expr) -> Result<(), SemanticError> {
        match &ast.kind {
            ExprKind::Literal(_) => Ok(()),
            ExprKind::Ident { name, .. } => self.visit_ident(name, ast.span),
            ExprKind::Unary { expr, .. } => self.visit(expr),
            ExprKind::Binary { left, right, .. } => {
                self.visit(left)?;
                self.visit(right)
            }
            ExprKind::Call { name, args, .. } => self.visit_call(name, args, ast.span),
        }
    }

    fn visit_ident(&self, name: &str, span: Span) -> Result<(), SemanticError> {
        let symbol = self.get_symbol(name, span)?;

        match symbol {
            Symbol::Const { .. } => Ok(()),
            Symbol::Func { .. } => Err(SemanticError::SymbolIsNotAConstant {
                name: name.to_string(),
                span,
            }),
        }
    }

    fn visit_call(&mut self, name: &str, args: &[Expr], span: Span) -> Result<(), SemanticError> {
        let sym_span = Span::new(span.start, span.start + name.len());
        let symbol = self.get_symbol(name, sym_span)?;

        let Symbol::Func {
            args: min_args,
            variadic,
            ..
        } = symbol
        else {
            return Err(SemanticError::SymbolIsNotAFunction {
                name: name.to_string(),
                span: sym_span,
            });
        };

        self.validate_arity(name, args.len(), *min_args, *variadic, span)?;

        // Validate arguments recursively
        for arg in args {
            self.visit(arg)?;
        }

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

    fn get_symbol(&self, name: &str, span: Span) -> Result<&Symbol, SemanticError> {
        self.table.get(name).ok_or_else(|| SemanticError::UndefinedSymbol {
            name: name.to_string(),
            span,
        })
    }
}
