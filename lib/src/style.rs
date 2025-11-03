//! Configurable syntax highlighting styles for expression and assembly printing.
//!
//! This module provides style configuration for pretty-printing expressions
//! and assembly output with syntax highlighting. Colors and formatting can be customized.

use colored::{Color, ColoredString, Colorize};

/// Style configuration for syntax highlighting.
///
/// Controls the colors and formatting used when printing expressions and assembly.
/// Use [`ExprStyle::default()`] for sensible defaults, or customize
/// individual colors as needed.
///
/// # Notes
///
/// - Assembly instructions use the same `keyword_color` and `keyword_bold` settings
///   as expression keywords (if, let, then) for consistency.
/// - The `comment_color` is used for assembly comments and will be used for
///   future source code comment support.
#[derive(Debug, Clone)]
pub struct ExprStyle {
    // Expression printing
    /// Keywords (if, let, then)
    pub keyword_color: Color,
    /// Whether keywords should be bold
    pub keyword_bold: bool,
    /// Operators (+, -, *, /, ^, !, etc.)
    pub operator_color: Color,
    /// Numeric literals
    pub number_color: Color,
    /// Local symbols
    pub local_symbol_color: Color,
    /// Global symbols
    pub global_symbol_color: Color,
    /// Functions
    pub function_color: Color,
    /// Delimiters (parentheses, commas)
    pub delimiter_color: Color,

    // Assembly printing
    /// Assembly instruction addresses
    pub asm_address_color: Color,
    /// Comments (assembly and future source comments)
    pub comment_color: Color,
    /// Function labels in assembly
    pub asm_label_color: Color,
}

impl Default for ExprStyle {
    fn default() -> Self {
        Self {
            keyword_color: Color::Magenta,
            keyword_bold: true,
            operator_color: Color::White,
            number_color: Color::Green,
            local_symbol_color: Color::Cyan,
            global_symbol_color: Color::BrightCyan,
            function_color: Color::BrightYellow,
            delimiter_color: Color::White,
            asm_address_color: Color::Yellow,
            comment_color: Color::BrightBlack,
            asm_label_color: Color::BrightBlue,
        }
    }
}

impl ExprStyle {
    /// Creates a new style with default colors.
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies keyword styling to a string.
    pub fn keyword(&self, s: &str) -> ColoredString {
        let styled = s.color(self.keyword_color);
        if self.keyword_bold {
            styled.bold()
        } else {
            styled
        }
    }

    /// Applies operator styling to a string.
    pub fn operator(&self, s: &str) -> ColoredString {
        s.color(self.operator_color)
    }

    /// Applies number styling to a string.
    pub fn number(&self, s: &str) -> ColoredString {
        s.color(self.number_color)
    }

    /// Applies local symbol styling to a string.
    pub fn local_symbol(&self, s: &str) -> ColoredString {
        s.color(self.local_symbol_color)
    }

    /// Applies global symbol styling to a string.
    pub fn global_symbol(&self, s: &str) -> ColoredString {
        s.color(self.global_symbol_color)
    }

    /// Applies function styling to a string.
    pub fn function(&self, s: &str) -> ColoredString {
        s.color(self.function_color)
    }

    /// Applies delimiter styling to a string.
    pub fn delimiter(&self, s: &str) -> ColoredString {
        s.color(self.delimiter_color)
    }

    /// Applies assembly address styling to a string.
    pub fn asm_address(&self, s: &str) -> ColoredString {
        s.color(self.asm_address_color)
    }

    /// Applies comment styling to a string.
    pub fn comment(&self, s: &str) -> ColoredString {
        s.color(self.comment_color)
    }

    /// Applies assembly label styling to a string.
    pub fn asm_label(&self, s: &str) -> ColoredString {
        s.color(self.asm_label_color)
    }
}
