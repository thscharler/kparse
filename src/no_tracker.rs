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

use crate::{Code, DynTracker, TrackSpan};
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
pub struct NoTracker;

impl NoTracker {
    /// Creates a span with the correct context for NoContext.
    pub fn span<'s, T, C>(&'s self, txt: T) -> TrackSpan<'s, T, C>
    where
        T: AsBytes + Copy + 's,
        C: Code,
    {
        TrackSpan::new_extra(txt, DynTracker(None))
    }
}
