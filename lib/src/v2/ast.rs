//! Abstract Syntax Tree (v2) with owned strings.

use crate::span::Span;
use crate::token::Token;
use rust_decimal::Decimal;

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
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

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    // Comparison operators
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
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

/// Expression node in the AST.
///
/// Unlike v1, this version uses owned strings (no lifetime parameter).
#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// Expression kind with owned strings
#[derive(Debug, Clone)]
pub enum ExprKind {
    Literal(Decimal),
    Ident {
        name: String,
        sym_index: Option<usize>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
        sym_index: Option<usize>,
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
            kind: ExprKind::Ident {
                name,
                sym_index: None,
            },
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
            kind: ExprKind::Call {
                name,
                args,
                sym_index: None,
            },
            span,
        }
    }
}
