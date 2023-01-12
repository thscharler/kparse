use crate::{Code, HoldContext, ParseContext, Span, SpanOffset};
use std::error::Error;
use std::marker::PhantomData;

pub struct RawContext<'s, C: Code> {
    span: &'s str,
    _phantom: PhantomData<C>,
}

impl<'s, C: Code> RawContext<'s, C> {
    pub fn new(span: &'s str) -> Self {
        Self {
            span,
            _phantom: Default::default(),
        }
    }

    pub fn span(&'s self) -> Span<'s, C> {
        Span::new_extra(self.span, HoldContext { 0: self })
    }
}

impl<'s, C: Code> ParseContext<'s, C> for RawContext<'s, C> {
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
