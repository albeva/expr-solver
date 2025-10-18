use crate::ast::*;
use crate::span::{Span, SpanError};
use crate::symbol::{SymTable, Symbol};
use thiserror::Error;

/// Expression parsing and evaluation errors.
#[derive(Error, Debug, Clone)]
pub enum SemaError {
    #[error("Undefined symbol '{name}'")]
    UndefinedSymbol { name: String, span: Span },
    #[error("Symbol '{name}' is not a constant")]
    SymbolIsNotAConstant { name: String, span: Span },
    #[error("Symbol '{name}' is not a function")]
    SymbolIsNotAFunction { name: String, span: Span },
    #[error("Function '{name}' expects exactly {expected} arguments but got {got}")]
    ArgumentCountMismatch {
        name: String,
        expected: usize,
        got: usize,
        span: Span,
    },
    #[error("Function '{name}' expects at least {min} arguments but got {got}")]
    InsufficientArguments {
        name: String,
        min: usize,
        got: usize,
        span: Span,
    },
}

impl SpanError for SemaError {
    fn span(&self) -> Span {
        match self {
            SemaError::UndefinedSymbol { span, .. } => *span,
            SemaError::SymbolIsNotAConstant { span, .. } => *span,
            SemaError::SymbolIsNotAFunction { span, .. } => *span,
            SemaError::ArgumentCountMismatch { span, .. } => *span,
            SemaError::InsufficientArguments { span, .. } => *span,
        }
    }
}

/// Semantic analyzer for type checking and symbol resolution.
///
/// Validates that identifiers reference valid symbols and that function
/// calls have the correct number of arguments.
#[derive(Debug)]
pub struct Sema<'sym> {
    table: &'sym SymTable,
}

impl<'src, 'sym> Sema<'sym> {
    /// Creates a new semantic analyzer with the given symbol table.
    pub fn new(table: &'sym SymTable) -> Self {
        Self { table }
    }

    /// Analyzes an AST expression, resolving symbols and checking types.
    pub fn visit(&mut self, ast: &mut Expr<'src>) -> Result<(), SemaError> {
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
    ) -> Result<(), SemaError> {
        let (idx, sym) = self.get_symbol_with_index(name, span)?;

        let Symbol::Const { .. } = sym else {
            return Err(SemaError::SymbolIsNotAConstant {
                name: name.to_string(),
                span,
            });
        };

        *sym_index = Some(idx);
        Ok(())
    }

    fn visit_unary(&mut self, expr: &mut Expr<'src>) -> Result<(), SemaError> {
        self.visit(expr)
    }

    fn visit_binary(
        &mut self,
        left: &mut Expr<'src>,
        right: &mut Expr<'src>,
    ) -> Result<(), SemaError> {
        self.visit(left)?;
        self.visit(right)
    }

    fn visit_call(
        &mut self,
        name: &str,
        args: &mut Vec<Expr<'src>>,
        sym_index: &mut Option<usize>,
        span: Span,
    ) -> Result<(), SemaError> {
        // span here will include a whole call expression,
        // but is guaranteed to start with the symbol
        let sym_span = Span::new(span.start, span.start + name.len());
        let (idx, sym) = self.get_symbol_with_index(name, sym_span)?;

        let Symbol::Func {
            args: min_args,
            variadic,
            ..
        } = sym
        else {
            return Err(SemaError::SymbolIsNotAFunction {
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
    ) -> Result<(), SemaError> {
        if args == min_args || variadic && args > min_args {
            return Ok(());
        }
        if variadic {
            Err(SemaError::InsufficientArguments {
                name: name.to_string(),
                min: min_args,
                got: args,
                span,
            })
        } else {
            Err(SemaError::ArgumentCountMismatch {
                name: name.to_string(),
                expected: min_args,
                got: args,
                span,
            })
        }
    }

    fn analyse_arguments(&mut self, args: &mut [Expr<'src>]) -> Result<(), SemaError> {
        args.iter_mut().try_for_each(|a| self.visit(a))
    }

    fn get_symbol_with_index(
        &self,
        name: &str,
        span: Span,
    ) -> Result<(usize, &Symbol), SemaError> {
        self.table
            .get_with_index(name)
            .ok_or_else(|| SemaError::UndefinedSymbol {
                name: name.to_string(),
                span,
            })
    }
}
