// use crate::fragments::Fragment;
use crate::{Code, DynContext, Span};
use nom::AsBytes;

/// Null Context.
pub struct NoContext;

impl NoContext {
    /// Creates a span with the correct context for NoContext.
    pub fn span<'s, T: AsBytes + Copy + 's, C: Code>(&'s self, txt: T) -> Span<'s, T, C> {
        Span::new_extra(txt, DynContext(None))
    }
}
