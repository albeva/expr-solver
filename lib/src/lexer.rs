use crate::source::Source;
use crate::span::Span;
use crate::token::Token;
use rust_decimal::Decimal;
use std::iter::Peekable;
use std::str::Chars;
use std::str::FromStr;

/// A hand-written lexer for the mini expression language.
pub struct Lexer<'src> {
    input: &'src str,
    iter: Peekable<Chars<'src>>,
    start: usize,
    pos: usize,
}

impl<'src> Lexer<'src> {
    /// Create a new lexer from a source.
    pub fn new(source: &'src Source) -> Self {
        Self {
            input: source.input,
            iter: source.input.chars().peekable(),
            start: 0,
            pos: 0,
        }
    }

    /// Get the next token from the input.
    pub fn next(&mut self) -> Token<'src> {
        self.skip_spaces();
        self.start = self.pos;
        let ch = match self.read() {
            Some(c) => c,
            None => return Token::EOF,
        };
        match ch {
            '0'..='9' => self.number(false),
            '.' => self.number(true),
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '^' => Token::Caret,
            '!' => self.exclamation(),
            '=' => self.equals(),
            '<' => self.less(),
            '>' => self.greater(),
            '(' => Token::ParenOpen,
            ')' => Token::ParenClose,
            ',' => Token::Comma,
            ch if Self::is_ident_start(ch) => self.identifier(),
            _ => self.invalid(),
        }
    }

    /// Get the span of the current token.
    pub fn span(&self) -> Span {
        Span::new(self.start, self.pos)
    }

    fn skip_spaces(&mut self) {
        while let Some(ch) = self.peek() {
            if Self::is_space(ch) {
                self.read();
            } else {
                break;
            }
        }
    }

    fn invalid(&self) -> Token<'src> {
        Token::Invalid(&self.input[self.start..self.pos])
    }

    fn number(&mut self, mut seen_dot: bool) -> Token<'src> {
        let mut is_invalid = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.read();
            } else if ch == '.' {
                self.read();
                if seen_dot {
                    is_invalid = true;
                } else {
                    seen_dot = true;
                }
            } else {
                break;
            }
        }

        if is_invalid {
            return self.invalid();
        }

        let s = &self.input[self.start..self.pos];
        match Decimal::from_str(s) {
            Ok(n) => Token::Number(n),
            Err(_) => Token::Invalid(s),
        }
    }

    fn identifier(&mut self) -> Token<'src> {
        while let Some(ch) = self.peek() {
            if Self::is_ident_continue(ch) {
                self.read();
                continue;
            } else {
                break;
            }
        }
        let s = &self.input[self.start..self.pos];
        Token::Ident(s)
    }

    fn peek(&mut self) -> Option<char> {
        self.iter.peek().map(|ch| *ch)
    }

    fn read(&mut self) -> Option<char> {
        self.iter.next().inspect(|ch| self.pos += ch.len_utf8())
    }

    fn is_space(ch: char) -> bool {
        ch == ' ' || ch == '\t'
    }

    fn is_ident_start(ch: char) -> bool {
        ch == '_' || ch.is_alphabetic() || Self::is_emoji(ch)
    }

    fn is_ident_continue(ch: char) -> bool {
        ch == '_' || ch.is_alphanumeric() || Self::is_emoji(ch)
    }

    fn is_emoji(ch: char) -> bool {
        let u = ch as u32;
        matches!(u,
            0x1F300..=0x1FAFF   // Misc emoji blocks
            | 0x1F1E6..=0x1F1FF // Regional Indicator Symbols (flags)
            | 0x1F000..=0x1F02F // Mahjong / Domino
            | 0x2600..=0x26FF   // Misc symbols
            | 0x2700..=0x27BF   // Dingbats
            | 0xFE0F..=0xFE0F   // Variation Selector-16 used in emoji presentation
        )
    }

    fn exclamation(&mut self) -> Token<'src> {
        if self.peek() == Some('=') {
            self.read(); // consume '='
            Token::NotEqual
        } else {
            Token::Bang
        }
    }

    fn equals(&mut self) -> Token<'src> {
        if self.peek() == Some('=') {
            self.read(); // consume second '='
            Token::Equal
        } else {
            self.invalid() // single '=' is not valid
        }
    }

    fn less(&mut self) -> Token<'src> {
        if self.peek() == Some('=') {
            self.read(); // consume '='
            Token::LessEqual
        } else {
            Token::Less
        }
    }

    fn greater(&mut self) -> Token<'src> {
        if self.peek() == Some('=') {
            self.read(); // consume '='
            Token::GreaterEqual
        } else {
            Token::Greater
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::Source;
    use crate::token::Token;
    use rust_decimal::dec;

    fn lex_all<'src>(source: &'src Source) -> Vec<Token<'src>> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next();
            if matches!(tok, Token::EOF) {
                break;
            }
            tokens.push(tok);
        }
        tokens
    }

    #[test]
    fn test_number_with_multiple_dots() {
        let source = Source { input: "1.2.3" };
        let tokens = lex_all(&source);
        assert_eq!(tokens[0], Token::Invalid("1.2.3"));
    }

    #[test]
    fn test_identifier_with_emoji() {
        let source = Source {
            input: "foo😀 bar🚀",
        };
        let tokens = lex_all(&source);
        assert_eq!(tokens[0], Token::Ident("foo😀"));
        assert_eq!(tokens[1], Token::Ident("bar🚀"));
    }

    #[test]
    fn test_unknown_token() {
        let source = Source { input: "$" };
        let tokens = lex_all(&source);
        assert_eq!(tokens[0], Token::Invalid("$"));
    }

    #[test]
    fn test_whitespace_handling() {
        let source = Source {
            input: "  1 + 2\t\t*",
        };
        let tokens = lex_all(&source);
        assert_eq!(tokens[0], Token::Number(dec!(1)));
        assert_eq!(tokens[1], Token::Plus);
        assert_eq!(tokens[2], Token::Number(dec!(2)));
        assert_eq!(tokens[3], Token::Star);
    }

    #[test]
    fn test_comparison_operators() {
        let source = Source {
            input: "== != < <= > >=",
        };
        let tokens = lex_all(&source);
        assert_eq!(tokens[0], Token::Equal);
        assert_eq!(tokens[1], Token::NotEqual);
        assert_eq!(tokens[2], Token::Less);
        assert_eq!(tokens[3], Token::LessEqual);
        assert_eq!(tokens[4], Token::Greater);
        assert_eq!(tokens[5], Token::GreaterEqual);
    }

    #[test]
    fn test_factorial_vs_not_equal() {
        // Test that ! is factorial but != is not equal
        let source = Source { input: "5! != 100" };
        let tokens = lex_all(&source);
        assert_eq!(tokens[0], Token::Number(dec!(5)));
        assert_eq!(tokens[1], Token::Bang);
        assert_eq!(tokens[2], Token::NotEqual);
        assert_eq!(tokens[3], Token::Number(dec!(100)));
    }

    #[test]
    fn test_invalid_single_equals() {
        // Single '=' should be invalid since we only support '=='
        let source = Source { input: "=" };
        let tokens = lex_all(&source);
        assert_eq!(tokens[0], Token::Invalid("="));
    }
}
