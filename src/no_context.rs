use crate::fragments::Fragment;
use crate::{Code, DynContext, Span};
use nom::AsBytes;

/// Null Context.
pub struct NoContext;

impl NoContext {
    /// Creates a span with the correct context for NoContext.
    pub fn span<'s, T: AsBytes + Copy + 's, C: Code>(&'s self, txt: T) -> Span<'s, T, C> {
        Span::new_extra(txt, DynContext(None))
    }

    /// Tries to reconstruct the original span from the given span.
    /// Uses the same heuristic as LocatedSpan, which is probably the maximum possible.
    pub fn original<'s, T: AsBytes + Copy + Fragment + 's, C: Code>(
        &self,
        span: &Span<'s, T, C>,
    ) -> Span<'s, T, C> {
        unsafe {
            let buf = span.fragment().undo_subslice(span.location_offset());
            Span::new_from_raw_offset(0, 1, buf, span.extra)
        }
    }
}
