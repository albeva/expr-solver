//! Recursive descent parser for mathematical expressions.

use super::ast::{Decl, Expr, UnOp};
use super::error::ParseError;
use super::lexer::Lexer;
use super::span::Span;
use super::token::Token;

pub type ParseResult<'src> = Result<Expr<'src>, ParseError>;

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
    lexer: Lexer<'src>,
    lookahead: Token<'src>,
    span: Span,
}

impl<'src> Parser<'src> {
    /// Creates a new parser from a string slice.
    pub fn new(input: &'src str) -> Self {
        let mut lexer = Lexer::new(input);
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
    pub fn parse(&mut self) -> Result<Option<Expr<'src>>, ParseError> {
        if self.lookahead == Token::Eof {
            return Ok(None);
        }
        let expr = self.expression()?;
        self.expect(&Token::Eof)?;
        Ok(Some(expr))
    }

    fn expression(&mut self) -> ParseResult<'src> {
        let lhs = self.primary()?;
        self.climb(lhs, 1)
    }

    fn primary(&mut self) -> ParseResult<'src> {
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
            Token::If => self.if_expr(span),
            Token::Let => self.let_expr(span),
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
                message: format!(
                    "unexpected token '{}', expected an expression",
                    self.lookahead.lexeme()
                ),
                span,
            }),
        }
    }

    fn call(&mut self, id: &'src str, span: Span) -> ParseResult<'src> {
        // assume lookahead is '('
        self.advance();

        let mut args: Vec<Expr> = Vec::new();
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

    fn climb(&mut self, mut lhs: Expr<'src>, min_prec: u8) -> ParseResult<'src> {
        let mut prec = self.lookahead.precedence();
        while prec >= min_prec {
            // Handle postfix unary operators
            if self.lookahead.is_postfix_unary() {
                let op = self.lookahead.clone();
                let op_span = self.span;
                self.advance();
                prec = self.lookahead.precedence();

                let span = lhs.span.merge(op_span);
                lhs = Expr::unary(op.into(), lhs, span);
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

            let span = lhs.span.merge(rhs.span);
            lhs = Expr::binary(op.into(), lhs, rhs, span);
        }
        Ok(lhs)
    }

    fn if_expr(&mut self, span: Span) -> ParseResult<'src> {
        // Expect: if(cond, then_branch, else_branch)

        // assume lookahead is 'if'
        self.advance();

        // Self::expect_token(lexer, lookahead, span, &Token::ParenOpen)?;
        self.expect(&Token::ParenOpen)?;

        // Parse condition
        let cond = self.expression()?;
        self.expect(&Token::Comma)?;

        // Parse then branch
        let then_branch = self.expression()?;
        self.expect(&Token::Comma)?;

        // Parse else branch
        let else_branch = self.expression()?;
        self.expect(&Token::ParenClose)?;

        let span = span.merge(self.span);
        Ok(Expr::if_expr(cond, then_branch, else_branch, span))
    }

    fn let_expr(&mut self, span: Span) -> ParseResult<'src> {
        // Expect: let name = expr, name = expr, ... then body
        self.advance(); // consume 'let'

        // Parse at least one declaration
        let mut decls = vec![self.parse_let_decl()?];

        // Parse remaining declarations separated by commas
        while self.accept(&Token::Comma) {
            decls.push(self.parse_let_decl()?);
        }

        // Expect 'then'
        self.expect(&Token::Then)?;

        // Parse body
        let body = self.expression()?;

        let span = span.merge(self.span);
        Ok(Expr::let_expr(decls, body, span))
    }

    fn parse_let_decl(&mut self) -> Result<Decl<'src>, ParseError> {
        let name = self.ident()?;
        if self.accept(&Token::ParenOpen) {
            self.parse_func_decl(name)
        } else {
            self.parse_var_decl(name)
        }
    }

    fn parse_var_decl(&mut self, name: &'src str) -> Result<Decl<'src>, ParseError> {
        self.expect(&Token::Assign)?;
        let body = self.expression()?;
        Ok(Decl::Var { name, body })
    }

    fn parse_func_decl(&mut self, name: &'src str) -> Result<Decl<'src>, ParseError> {
        // [ id { "," id] ")" ]
        let mut params: Vec<&'src str> = Vec::new();
        if !self.accept(&Token::ParenClose) {
            loop {
                let param = self.ident()?;
                params.push(param);
                if self.accept(&Token::Comma) {
                    continue;
                }
                self.expect(&Token::ParenClose)?;
                break;
            }
        }

        // "=" expression .
        self.expect(&Token::Assign)?;
        let body = self.expression()?;

        Ok(Decl::Func { name, params, body })
    }

    fn ident(&mut self) -> Result<&'src str, ParseError> {
        match self.lookahead {
            Token::Ident(id) => {
                self.advance();
                Ok(id)
            }
            _ => Err(ParseError::UnexpectedToken {
                message: format!("expected identifier, found '{}'", self.lookahead.lexeme()),
                span: self.span,
            }),
        }
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
                message: format!(
                    "unexpected token '{}', expected '{}'",
                    self.lookahead.lexeme(),
                    tkn.lexeme()
                ),
                span: self.span,
            });
        }
        Ok(())
    }
}
