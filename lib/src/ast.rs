//! Abstract Syntax Tree for mathematical expressions.

use crate::span::Span;
use crate::token::Token;
use rust_decimal::Decimal;

/// Unary operators: negation and factorial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    /// Negation (`-`)
    Neg,
    /// Factorial (`!`)
    Fact,
}

impl From<Token<'_>> for UnOp {
    fn from(token: Token) -> Self {
        match token {
            Token::Minus => UnOp::Neg,
            Token::Bang => UnOp::Fact,
            _ => unreachable!("Invalid token for unary operator"),
        }
    }
}

/// Binary operators: arithmetic and comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    /// Addition (`+`)
    Add,
    /// Subtraction (`-`)
    Sub,
    /// Multiplication (`*`)
    Mul,
    /// Division (`/`)
    Div,
    /// Exponentiation (`^`)
    Pow,
    /// Equality (`==`)
    Equal,
    /// Inequality (`!=`)
    NotEqual,
    /// Less than (`<`)
    Less,
    /// Less than or equal (`<=`)
    LessEqual,
    /// Greater than (`>`)
    Greater,
    /// Greater than or equal (`>=`)
    GreaterEqual,
}

impl From<Token<'_>> for BinOp {
    fn from(token: Token) -> Self {
        match token {
            Token::Plus => BinOp::Add,
            Token::Minus => BinOp::Sub,
            Token::Star => BinOp::Mul,
            Token::Slash => BinOp::Div,
            Token::Caret => BinOp::Pow,
            Token::Equal => BinOp::Equal,
            Token::NotEqual => BinOp::NotEqual,
            Token::Less => BinOp::Less,
            Token::LessEqual => BinOp::LessEqual,
            Token::Greater => BinOp::Greater,
            Token::GreaterEqual => BinOp::GreaterEqual,
            _ => unreachable!("Invalid token for binary operator"),
        }
    }
}

/// Expression node in the AST with source location.
#[derive(Debug, Clone)]
pub struct Expr<'src> {
    pub kind: ExprKind<'src>,
    pub span: Span,
}

/// Expression kind representing different types of expressions.
#[derive(Debug, Clone)]
pub enum ExprKind<'src> {
    /// Numeric literal
    Literal(Decimal),
    /// Identifier (constant or variable)
    Ident { name: &'src str },
    /// Unary operation
    Unary { op: UnOp, expr: Box<Expr<'src>> },
    /// Binary operation
    Binary {
        op: BinOp,
        left: Box<Expr<'src>>,
        right: Box<Expr<'src>>,
    },
    /// Function call
    Call {
        name: &'src str,
        args: Vec<Expr<'src>>,
    },
    /// Conditional expression
    If {
        cond: Box<Expr<'src>>,
        then_branch: Box<Expr<'src>>,
        else_branch: Box<Expr<'src>>,
    },
}

impl<'src> Expr<'src> {
    pub fn literal(value: Decimal, span: Span) -> Self {
        Self {
            kind: ExprKind::Literal(value),
            span,
        }
    }

    pub fn ident(name: &'src str, span: Span) -> Self {
        Self {
            kind: ExprKind::Ident { name },
            span,
        }
    }

    pub fn unary(op: UnOp, expr: Expr<'src>, span: Span) -> Self {
        Self {
            kind: ExprKind::Unary {
                op,
                expr: Box::new(expr),
            },
            span,
        }
    }

    pub fn binary(op: BinOp, left: Expr<'src>, right: Expr<'src>, span: Span) -> Self {
        Self {
            kind: ExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        }
    }

    pub fn call(name: &'src str, args: Vec<Expr<'src>>, span: Span) -> Self {
        Self {
            kind: ExprKind::Call { name, args },
            span,
        }
    }

    pub fn if_expr(
        cond: Expr<'src>,
        then_branch: Expr<'src>,
        else_branch: Expr<'src>,
        span: Span,
    ) -> Self {
        Self {
            kind: ExprKind::If {
                cond: Box::new(cond),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            },
            span,
        }
    }
}
