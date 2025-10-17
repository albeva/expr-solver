use crate::span::Span;
use crate::symbol::Symbol;
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
pub struct Expr<'src, 'sym> {
    pub kind: ExprKind<'src, 'sym>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind<'src, 'sym> {
    Literal(Decimal),
    Ident {
        name: &'src str,
        sym: Option<&'sym Symbol>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr<'src, 'sym>>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr<'src, 'sym>>,
        right: Box<Expr<'src, 'sym>>,
    },
    Call {
        name: &'src str,
        args: Vec<Expr<'src, 'sym>>,
        sym: Option<&'sym Symbol>,
    },
}

impl<'src, 'sym> Expr<'src, 'sym> {
    pub fn literal(value: Decimal, span: Span) -> Self {
        Self {
            kind: ExprKind::Literal(value),
            span,
        }
    }

    pub fn ident(name: &'src str, span: Span) -> Self {
        Self {
            kind: ExprKind::Ident { name, sym: None },
            span,
        }
    }

    pub fn unary(op: UnOp, expr: Expr<'src, 'sym>, span: Span) -> Self {
        Self {
            kind: ExprKind::Unary {
                op,
                expr: Box::new(expr),
            },
            span,
        }
    }

    pub fn binary(op: BinOp, left: Expr<'src, 'sym>, right: Expr<'src, 'sym>, span: Span) -> Self {
        Self {
            kind: ExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        }
    }

    pub fn call(name: &'src str, args: Vec<Expr<'src, 'sym>>, span: Span) -> Self {
        Self {
            kind: ExprKind::Call {
                name,
                args,
                sym: None,
            },
            span,
        }
    }
}
