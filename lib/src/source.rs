use crate::parser::ParseError;
use crate::span::Span;
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

/// Source code container with input validation.
#[derive(Debug, Clone)]
pub struct Source<'str> {
    pub input: &'str str,
}

impl<'str> Source<'str> {
    /// Create a new source from input string.
    ///
    /// Validates input length and trims whitespace.
    pub fn new(input: &'str str) -> Result<Self, ParseError> {
        let trimmed = input.trim();
        Ok(Self { input: trimmed })
    }

    /// Return source string with a highlighted section
    pub fn highlight(&self, span: &Span) -> String {
        let input = &self.input;
        let pre = Self::escape(&input[..span.start]);
        let tok = Self::escape(&input[span.start..span.end]);
        let post = Self::escape(&input[span.end..]);
        let line = format!("{}{}{}", pre, tok.red().bold(), post);

        let caret = "^".green().bold();
        let squiggly_len = UnicodeWidthStr::width(tok.as_str());
        let caret_offset = UnicodeWidthStr::width(pre.as_str()) + caret.len();

        format!(
            "1 | {0}\n  | {1: >2$}{3}",
            line,
            caret,
            caret_offset,
            "~".repeat(squiggly_len.saturating_sub(1)).green()
        )
    }

    fn escape(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                other => out.push(other),
            }
        }
        out
    }
}
