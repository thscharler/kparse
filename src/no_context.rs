//! No tracking context. Does nothing but producing a suitable Span.
//!
//! ```rust ignore
//! use kparse::NoContext;
//!
//! let txt = "asdf";
//!
//! let span = NoContext.span(txt);
//!
//! // ... run the parser
//!
//! ```
//!

use crate::{Code, CtxSpan, DynContext};
use nom::AsBytes;

/// No tracking context. Does nothing but producing a suitable Span.
///
/// ```rust ignore
/// use kparse::NoContext;
///
/// let txt = "asdf";
///
/// let span = NoContext.span(txt);
///
/// // ... run the parser
///
/// ```
///
pub struct NoContext;

impl NoContext {
    /// Creates a span with the correct context for NoContext.
    pub fn span<'s, T, C>(&'s self, txt: T) -> CtxSpan<'s, T, C>
    where
        T: AsBytes + Copy + 's,
        C: Code,
    {
        CtxSpan::new_extra(txt, DynContext(None))
    }
}
