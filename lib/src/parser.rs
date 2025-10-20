//! Recursive descent parser for mathematical expressions.

use super::ast::{BinOp, Expr, UnOp};
use super::error::ParseError;
use super::lexer::Lexer;
use crate::span::Span;
use crate::token::Token;

pub type ParseResult = Result<Expr, ParseError>;

/// Recursive descent parser for mathematical expressions.
///
/// Uses operator precedence climbing for efficient binary operator parsing.
///
/// # Examples
///
/// ```
/// use expr_solver::Parser;
///
/// let mut parser = Parser::new("2 + 3 * 4");
/// let ast = parser.parse().unwrap();
/// assert!(ast.is_some());
/// ```
pub struct Parser<'src> {
    input: &'src str,
}

impl<'src> Parser<'src> {
    /// Creates a new parser from a string slice.
    pub fn new(input: &'src str) -> Self {
        Self { input }
    }

    /// Parses the input into an abstract syntax tree.
    ///
    /// Returns `None` for empty input, or an expression on success.
    pub fn parse(&mut self) -> Result<Option<Expr>, ParseError> {
        let mut lexer = Lexer::new(self.input);
        let mut lookahead = lexer.next();
        let mut span = lexer.span();

        if lookahead == Token::Eof {
            return Ok(None);
        }

        let expr = Self::expression(&mut lexer, &mut lookahead, &mut span)?;
        Self::expect_token(&mut lexer, &mut lookahead, &mut span, &Token::Eof)?;
        Ok(Some(expr))
    }

    fn expression<'lex>(
        lexer: &mut Lexer<'lex>,
        lookahead: &mut Token<'lex>,
        span: &mut Span,
    ) -> ParseResult {
        let lhs = Self::primary(lexer, lookahead, span)?;
        Self::climb(lexer, lookahead, span, lhs, 1)
    }

    fn primary<'lex>(
        lexer: &mut Lexer<'lex>,
        lookahead: &mut Token<'lex>,
        span: &mut Span,
    ) -> ParseResult {
        let current_span = *span;
        match *lookahead {
            Token::Number(n) => {
                Self::advance(lexer, lookahead, span);
                Ok(Expr::literal(n, current_span))
            }
            Token::Ident(id) => {
                let id_string = id.to_string();
                Self::advance(lexer, lookahead, span);
                if *lookahead == Token::ParenOpen {
                    return Self::call(lexer, lookahead, span, id_string, current_span);
                }
                Ok(Expr::ident(id_string, current_span))
            }
            Token::Minus => {
                Self::advance(lexer, lookahead, span);
                let expr = Self::primary(lexer, lookahead, span)?;
                let expr = Self::climb(lexer, lookahead, span, expr, Token::Negate.precedence())?;
                let full_span = current_span.merge(expr.span);
                Ok(Expr::unary(UnOp::Neg, expr, full_span))
            }
            Token::ParenOpen => {
                Self::advance(lexer, lookahead, span);
                let expr = Self::expression(lexer, lookahead, span)?;
                Self::expect_token(lexer, lookahead, span, &Token::ParenClose)?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken {
                message: format!(
                    "unexpected token '{}', expected an expression",
                    lookahead.lexeme()
                ),
                span: current_span,
            }),
        }
    }

    fn call<'lex>(
        lexer: &mut Lexer<'lex>,
        lookahead: &mut Token<'lex>,
        span: &mut Span,
        id: String,
        start_span: Span,
    ) -> ParseResult {
        // assume lookahead is '('
        Self::advance(lexer, lookahead, span);

        let mut args: Vec<Expr> = Vec::new();
        while *lookahead != Token::ParenClose {
            let arg = Self::expression(lexer, lookahead, span)?;
            args.push(arg);
            if *lookahead == Token::Comma {
                Self::advance(lexer, lookahead, span);
            } else {
                break;
            }
        }
        Self::expect_token(lexer, lookahead, span, &Token::ParenClose)?;

        let full_span = start_span.merge(*span);
        Ok(Expr::call(id, args, full_span))
    }

    fn climb<'lex>(
        lexer: &mut Lexer<'lex>,
        lookahead: &mut Token<'lex>,
        span: &mut Span,
        mut lhs: Expr,
        min_prec: u8,
    ) -> ParseResult {
        let mut prec = lookahead.precedence();
        while prec >= min_prec {
            // Handle postfix unary operators
            if lookahead.is_postfix_unary() {
                let op = lookahead.clone();
                let op_span = *span;
                Self::advance(lexer, lookahead, span);
                prec = lookahead.precedence();

                let unary_op = UnOp::from_token(&op);
                let full_span = lhs.span.merge(op_span);
                lhs = Expr::unary(unary_op, lhs, full_span);
                continue;
            }

            let op = lookahead.clone();

            Self::advance(lexer, lookahead, span);
            let mut rhs = Self::primary(lexer, lookahead, span)?;
            prec = lookahead.precedence();

            while prec > op.precedence()
                || (lookahead.is_right_associative() && prec == op.precedence())
            {
                rhs = Self::climb(lexer, lookahead, span, rhs, prec)?;
                prec = lookahead.precedence();
            }

            let binop = BinOp::from_token(&op);
            let full_span = lhs.span.merge(rhs.span);
            lhs = Expr::binary(binop, lhs, rhs, full_span);
        }
        Ok(lhs)
    }

    fn advance<'lex>(lexer: &mut Lexer<'lex>, lookahead: &mut Token<'lex>, span: &mut Span) {
        *lookahead = lexer.next();
        *span = lexer.span();
    }

    fn expect_token<'lex>(
        lexer: &mut Lexer<'lex>,
        lookahead: &mut Token<'lex>,
        span: &mut Span,
        expected: &Token<'lex>,
    ) -> Result<(), ParseError> {
        if lookahead == expected {
            Self::advance(lexer, lookahead, span);
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                message: format!(
                    "unexpected token '{}', expected '{}'",
                    lookahead.lexeme(),
                    expected.lexeme()
                ),
                span: *span,
            })
        }
    }
}
