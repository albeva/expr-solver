use crate::ast::{BinOp, Expr, UnOp};
use crate::lexer::Lexer;
use crate::source::Source;
use crate::span::{Span, SpanError};
use crate::token::Token;
use thiserror::Error;

/// Expression parsing errors.
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Unexpected token '{found}', expected '{expected}'")]
    UnexpectedToken {
        found: String,
        expected: String,
        span: Span,
    },
}

impl SpanError for ParseError {
    fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => *span,
        }
    }
}

pub type ParseResult<'src, 'sym> = Result<Expr<'src, 'sym>, ParseError>;

/// Recursive descent parser for mathematical expressions.
///
/// Uses operator precedence climbing for efficient binary operator parsing.
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    lookahead: Token<'src>,
    span: Span,
}

impl<'src, 'sym> Parser<'src> {
    /// Creates a new parser from a source.
    pub fn new(source: &'src Source) -> Self {
        let mut lexer = Lexer::new(source);
        let lookahead = lexer.next();
        let span = lexer.span();
        Self {
            lexer,
            lookahead,
            span,
        }
    }

    /// Parses the source into an abstract syntax tree.
    ///
    /// Returns `None` for empty input, or an expression AST on success.
    pub fn parse(&mut self) -> Result<Option<Expr<'src, 'sym>>, ParseError> {
        if self.lookahead == Token::EOF {
            return Ok(None);
        }
        let expr = self.expression()?;
        self.expect(&Token::EOF)?;
        Ok(Some(expr))
    }

    fn expression(&mut self) -> ParseResult<'src, 'sym> {
        let lhs = self.primary()?;
        self.climb(lhs, 1)
    }

    fn primary(&mut self) -> ParseResult<'src, 'sym> {
        let span = self.span;
        match self.lookahead {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::literal(n, span))
            }
            Token::Ident(id) => {
                self.advance();
                if self.lookahead == Token::ParenOpen {
                    return self.call(id, span);
                }
                Ok(Expr::ident(id, span))
            }
            Token::Minus => {
                self.advance();
                let expr = self.primary()?;
                let expr = self.climb(expr, Token::Negate.precedence())?;
                let span = self.span.merge(expr.span);
                Ok(Expr::unary(UnOp::Neg, expr, span))
            }
            Token::ParenOpen => {
                self.advance();
                let expr = self.expression()?;
                self.expect(&Token::ParenClose)?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken {
                found: self.lookahead.lexeme().to_string(),
                expected: "an expression".to_string(),
                span,
            }),
        }
    }

    fn call(&mut self, id: &'src str, span: Span) -> ParseResult<'src, 'sym> {
        // assume lookahead is '('
        self.advance();

        let mut args: Vec<Expr<'src, 'sym>> = Vec::new();
        while self.lookahead != Token::ParenClose {
            let arg = self.expression()?;
            args.push(arg);
            if self.lookahead == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::ParenClose)?;

        let span = span.merge(self.span);
        Ok(Expr::call(id, args, span))
    }

    fn climb(&mut self, mut lhs: Expr<'src, 'sym>, min_prec: u8) -> ParseResult<'src, 'sym> {
        let mut prec = self.lookahead.precedence();
        while prec >= min_prec {
            // Handle postfix unary operators
            if self.lookahead.is_postfix_unary() {
                let op = self.lookahead.clone();
                let op_span = self.span;
                self.advance();
                prec = self.lookahead.precedence();

                let unary_op = UnOp::from_token(&op);
                let span = lhs.span.merge(op_span);
                lhs = Expr::unary(unary_op, lhs, span);
                continue;
            }

            let op = self.lookahead.clone();

            self.advance();
            let mut rhs = self.primary()?;
            prec = self.lookahead.precedence();

            while prec > op.precedence()
                || (self.lookahead.is_right_associative() && prec == op.precedence())
            {
                rhs = self.climb(rhs, prec)?;
                prec = self.lookahead.precedence();
            }

            let op = BinOp::from_token(&op);
            let span = lhs.span.merge(rhs.span);
            lhs = Expr::binary(op, lhs, rhs, span);
        }
        Ok(lhs)
    }

    fn advance(&mut self) {
        self.lookahead = self.lexer.next();
        self.span = self.lexer.span();
    }

    fn accept(&mut self, t: &Token<'src>) -> bool {
        if self.lookahead == *t {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, tkn: &Token<'src>) -> Result<(), ParseError> {
        if !self.accept(tkn) {
            return Err(ParseError::UnexpectedToken {
                found: self.lookahead.lexeme().to_string(),
                expected: tkn.lexeme().to_string(),
                span: self.span,
            });
        }
        Ok(())
    }
}
