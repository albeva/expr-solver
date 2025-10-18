use crate::span::Span;
use crate::token::Token;
use rust_decimal::Decimal;

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

#[derive(Debug, Clone)]
pub struct Expr<'src> {
    pub kind: ExprKind<'src>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind<'src> {
    Literal(Decimal),
    Ident {
        name: &'src str,
        sym_index: Option<usize>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr<'src>>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr<'src>>,
        right: Box<Expr<'src>>,
    },
    Call {
        name: &'src str,
        args: Vec<Expr<'src>>,
        sym_index: Option<usize>,
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
            kind: ExprKind::Ident {
                name,
                sym_index: None,
            },
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
            kind: ExprKind::Call {
                name,
                args,
                sym_index: None,
            },
            span,
        }
    }
}
