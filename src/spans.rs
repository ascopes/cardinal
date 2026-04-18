use std::fmt::Debug;

/// Representation of a span of bytes within a source file.
///
/// Spans start at the start offset and end at the end offset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    #[inline(always)]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline(always)]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline(always)]
    pub fn end(&self) -> usize {
        self.end
    }
}

/// Type that holds a value and an associated span.
#[derive(Clone, Debug)]
pub struct Spanned<T>
where
    T: Clone + Debug,
{
    value: T,
    span: Span,
}

impl<T> Spanned<T>
where
    T: Clone + Debug,
{
    #[inline(always)]
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }

    #[inline(always)]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[inline(always)]
    pub fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_is_created_as_expected() {
        // Given
        let start = 67usize;
        let end = 420usize;

        // When
        let span = Span::new(start, end);

        // Then
        assert_eq!(span.start(), start, "start");
        assert_eq!(span.end(), end, "end");
    }

    #[test]
    fn spanned_is_created_as_expected() {
        #[derive(Clone, Copy, Debug, PartialEq)]
        struct Something {
            data: u64,
        }

        // Given
        let start = 34usize;
        let end = 180usize;
        let span = Span::new(start, end);
        let thing = Something { data: 123 };

        // When
        let spanned = Spanned::new(thing, span);

        // Then
        assert_eq!(spanned.value(), &thing, "value");
        assert_eq!(spanned.span().start(), start, "span.start");
        assert_eq!(spanned.span().end(), end, "span.end");
    }
}
