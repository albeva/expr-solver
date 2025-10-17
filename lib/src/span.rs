/// A span representing a range in the source code.
///
/// Spans are used for error reporting and source highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Inclusive byte offset of the start
    pub start: usize,
    /// Exclusive byte offset of the end
    pub end: usize,
}

impl Span {
    /// Creates a new span from start and end byte offsets.
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Merges two spans, returning a span that covers both.
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Trait for errors that have an associated span in the source code.
pub trait SpanError
where
    Self: std::error::Error,
{
    /// Returns the span associated with this error.
    fn span(&self) -> Span;
}
