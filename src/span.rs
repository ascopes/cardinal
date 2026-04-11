use std::fmt::Debug;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

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
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn span(&self) -> Span {
        self.span
    }
}
