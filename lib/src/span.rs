#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize, // inclusive byte offset
    pub end: usize,   // exclusive byte offset
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

pub trait SpanError
where
    Self: std::error::Error,
{
    fn span(&self) -> Span;
}
