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

use crate::{Code, DynContext, Span};
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
    pub fn span<'s, T: AsBytes + Copy + 's, C: Code>(&'s self, txt: T) -> Span<'s, T, C> {
        Span::new_extra(txt, DynContext(None))
    }
}
