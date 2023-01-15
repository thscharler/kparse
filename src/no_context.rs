use crate::data_frame::undo_take_str_slice_unchecked;
use crate::{Code, DynContext, ParseContext, Span};
use std::error::Error;

/// Null Context.
pub struct NoContext;

impl NoContext {
    pub fn span<'s, C: Code>(&'s self, txt: &'s str) -> Span<'s, C> {
        Span::new_extra(txt, DynContext(self))
    }
}

/// Null Context
impl<'s, C: Code> ParseContext<'s, C> for NoContext {
    fn original(&self, span: &Span<'s, C>) -> Span<'s, C> {
        unsafe {
            let buf = undo_take_str_slice_unchecked(span.fragment(), span.location_offset());
            Span::new_from_raw_offset(0, 1, buf, span.extra)
        }
    }

    fn enter(&self, _: C, _: &Span<'s, C>) {}

    fn debug(&self, _: &Span<'s, C>, _: String) {}

    fn info(&self, _: &Span<'s, C>, _: &'static str) {}

    fn warn(&self, _: &Span<'s, C>, _: &'static str) {}

    fn exit_ok(&self, _: &Span<'s, C>, _: &Span<'s, C>) {}

    fn exit_err(&self, _: &Span<'s, C>, _: C, _: &dyn Error) {}
}
