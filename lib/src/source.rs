//! Source code container with owned string (v2).

use crate::span::Span;
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

/// Source code container with input validation and error highlighting.
///
/// Unlike the v1 version, this owns the source string.
#[derive(Debug, Clone)]
pub struct Source {
    pub input: String,
}

impl Source {
    /// Creates a new source from an input string.
    ///
    /// The input is trimmed of leading and trailing whitespace.
    pub fn new(input: impl Into<String>) -> Self {
        let input = input.into();
        let trimmed = input.trim().to_string();
        Self { input: trimmed }
    }

    /// Returns a reference to the input string as a str slice.
    pub fn as_str(&self) -> &str {
        &self.input
    }

    /// Returns a formatted string with syntax highlighting for the given span.
    ///
    /// The output includes a caret and squiggly line pointing to the error location.
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
