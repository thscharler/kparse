use crate::{Code, DynContext, ParseContext, Span};
use nom::AsBytes;
use std::error::Error;
use std::marker::PhantomData;

/// Just for tests.
#[derive(Debug)]
pub struct StrContext<'s, T: 's> {
    span: T,
    _phantom: PhantomData<&'s ()>,
}

impl<'s, T: AsBytes + Copy + 's> StrContext<'s, T> {
    /// Creates a new Context for the given parse text.
    pub fn new(span: T) -> Self {
        Self {
            span,
            _phantom: Default::default(),
        }
    }

    /// Returns a span with the correct context for StrContext.
    pub fn span<C: Code>(&'s self) -> Span<'s, T, C> {
        Span::new_extra(self.span, DynContext(Some(self)))
    }
}

impl<'s, T: AsBytes + Copy + 's, C: Code> ParseContext<'s, T, C> for StrContext<'s, T> {
    fn original(&self, span: &Span<'s, T, C>) -> Span<'s, T, C> {
        Span::new_extra(self.span, span.extra)
    }

    fn enter(&self, _: C, _: &Span<'s, T, C>) {}

    fn debug(&self, _: &Span<'s, T, C>, _: String) {}

    fn info(&self, _: &Span<'s, T, C>, _: &'static str) {}

    fn warn(&self, _: &Span<'s, T, C>, _: &'static str) {}

    fn exit_ok(&self, _: &Span<'s, T, C>, _: &Span<'s, T, C>) {}

    fn exit_err(&self, _: &Span<'s, T, C>, _: C, _: &dyn Error) {}
}
