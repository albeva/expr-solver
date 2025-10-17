use Cow::{Borrowed, Owned};
use rust_decimal::Decimal;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'src> {
    Number(Decimal),
    Ident(&'src str),
    Plus,
    Minus,
    Negate,
    Star,
    Slash,
    Caret,
    Bang,
    ParenOpen,
    ParenClose,
    Comma,
    // Comparison operators
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    EOF,
    Invalid(&'src str),
}

impl<'src> Token<'src> {
    pub fn precedence(&self) -> u8 {
        match self {
            Token::Equal
            | Token::NotEqual
            | Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual => 1,
            Token::Plus | Token::Minus => 2,
            Token::Star | Token::Slash => 3,
            Token::Negate => 4,
            Token::Caret => 5,
            Token::Bang => 6,
            _ => 0,
        }
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, Token::Caret)
    }

    pub fn is_postfix_unary(&self) -> bool {
        matches!(self, Token::Bang)
    }

    pub fn lexeme(&self) -> Cow<'src, str> {
        match self {
            Token::Number(decimal) => Owned(decimal.to_string()),
            Token::Ident(str) => Borrowed(str),
            Token::Plus => Borrowed("+"),
            Token::Minus => Borrowed("-"),
            Token::Negate => Borrowed("-"),
            Token::Star => Borrowed("*"),
            Token::Slash => Borrowed("/"),
            Token::Caret => Borrowed("^"),
            Token::Bang => Borrowed("!"),
            Token::ParenOpen => Borrowed("("),
            Token::ParenClose => Borrowed(")"),
            Token::Comma => Borrowed(","),
            Token::Equal => Borrowed("=="),
            Token::NotEqual => Borrowed("!="),
            Token::Less => Borrowed("<"),
            Token::LessEqual => Borrowed("<="),
            Token::Greater => Borrowed(">"),
            Token::GreaterEqual => Borrowed(">="),
            Token::EOF => Borrowed("EOF"),
            Token::Invalid(str) => match *str {
                "\n" => Borrowed("\\n"),
                "\r" => Borrowed("\\r"),
                _ => Borrowed(str),
            },
        }
    }
}
