use crate::{Code, DynContext, ParseContext, Span};
use std::error::Error;

/// Just for tests.
pub struct StrContext<'s> {
    span: &'s str,
}

impl<'s> StrContext<'s> {
    pub fn new(span: &'s str) -> Self {
        Self { span }
    }

    pub fn span<C: Code>(&'s self) -> Span<'s, C> {
        Span::new_extra(self.span, DynContext(Some(self)))
    }
}

impl<'s, C: Code> ParseContext<'s, C> for StrContext<'s> {
    fn original(&self, span: &Span<'s, C>) -> Span<'s, C> {
        Span::new_extra(self.span, span.extra)
    }

    fn enter(&self, _: C, _: &Span<'s, C>) {}

    fn debug(&self, _: &Span<'s, C>, _: String) {}

    fn info(&self, _: &Span<'s, C>, _: &'static str) {}

    fn warn(&self, _: &Span<'s, C>, _: &'static str) {}

    fn exit_ok(&self, _: &Span<'s, C>, _: &Span<'s, C>) {}

    fn exit_err(&self, _: &Span<'s, C>, _: C, _: &dyn Error) {}
}
