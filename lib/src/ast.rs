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

impl UnOp {
    pub fn from_token(token: &Token) -> Self {
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

impl BinOp {
    pub fn from_token(token: &Token) -> Self {
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
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// Expression kind representing different types of expressions.
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// Numeric literal
    Literal(Decimal),
    /// Identifier (constant or variable)
    Ident { name: String },
    /// Unary operation
    Unary { op: UnOp, expr: Box<Expr> },
    /// Binary operation
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Function call
    Call { name: String, args: Vec<Expr> },
    /// Conditional expression
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}

impl Expr {
    pub fn literal(value: Decimal, span: Span) -> Self {
        Self {
            kind: ExprKind::Literal(value),
            span,
        }
    }

    pub fn ident(name: String, span: Span) -> Self {
        Self {
            kind: ExprKind::Ident { name },
            span,
        }
    }

    pub fn unary(op: UnOp, expr: Expr, span: Span) -> Self {
        Self {
            kind: ExprKind::Unary {
                op,
                expr: Box::new(expr),
            },
            span,
        }
    }

    pub fn binary(op: BinOp, left: Expr, right: Expr, span: Span) -> Self {
        Self {
            kind: ExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        }
    }

    pub fn call(name: String, args: Vec<Expr>, span: Span) -> Self {
        Self {
            kind: ExprKind::Call { name, args },
            span,
        }
    }

    pub fn if_expr(cond: Expr, then_branch: Expr, else_branch: Expr, span: Span) -> Self {
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
