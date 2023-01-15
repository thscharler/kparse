use crate::data_frame::undo_take_str_slice_unchecked;
use crate::{Code, DynContext, Span};

/// Null Context.
pub struct NoContext;

impl NoContext {
    pub fn span<'s, C: Code>(&'s self, txt: &'s str) -> Span<'s, C> {
        Span::new_extra(txt, DynContext(None))
    }

    pub fn original<'s, C: Code>(&self, span: &Span<'s, C>) -> Span<'s, C> {
        unsafe {
            let buf = undo_take_str_slice_unchecked(span.fragment(), span.location_offset());
            Span::new_from_raw_offset(0, 1, buf, span.extra)
        }
    }
}
