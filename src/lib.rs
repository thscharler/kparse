//!
//! Additional functionality surrounding nom.
//!

use nom_locate::LocatedSpan;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

mod conversion;
mod data_frame;
mod debug;
mod error;
mod tracker;
mod tracking_context;

pub use conversion::*;
pub use data_frame::*;
pub use error::*;
pub use tracker::*;
pub use tracking_context::*;

pub mod prelude {
    pub use crate::ParserError;
    pub use crate::{Code, ParseContext, TrackParseErr, WithCode, WithSpan};
    pub use crate::{ParserNomResult, ParserResult, Span};
}

/// Standard input type.
pub type Span<'s, C> = LocatedSpan<&'s str, HoldContext<'s, C>>;

/// Result type.
pub type ParserResult<'s, O, C, X> = Result<(Span<'s, C>, O), nom::Err<ParserError<'s, C, X>>>;

/// Type alias for a nom parser. Use this to create a ParserError directly in nom.
pub type ParserNomResult<'s, C, X> =
    Result<(Span<'s, C>, Span<'s, C>), nom::Err<ParserError<'s, C, X>>>;

/// Parser state codes.
///
/// These are used for error handling and parser results and
/// everything else.
pub trait Code: Copy + Display + Debug + Eq {
    const NOM_ERROR: Self;
}

///
/// Context and tracking for a parser.
///
pub trait ParseContext<'s, C: Code> {
    /// Returns a span that encloses all of the current parser.
    fn original(&self, span: &Span<'s, C>) -> Span<'s, C>;

    /// Create a span that goes from the start of the first to the
    /// end of the second span.
    ///
    /// Safety
    /// It's unsafe because you must only use spans derived from
    /// the original span.
    ///
    /// Panics
    /// If first is after second in the original span.
    /// If any of the calculated offsets lies outsides the original span.
    unsafe fn span_union(&self, first: &Span<'s, C>, second: &Span<'s, C>) -> Span<'s, C>;

    /// Tracks entering a parser function.
    fn enter(&self, span: &Span<'s, C>, func: C);

    /// Debugging
    fn debug(&self, span: &Span<'s, C>, debug: String);

    /// Track something.
    fn info(&self, span: &Span<'s, C>, info: &'static str);

    /// Track something more important.
    fn warn(&self, span: &Span<'s, C>, warn: &'static str);

    /// Tracks an Ok result of a parser function.
    fn exit_ok(&self, span: &Span<'s, C>, parsed: &Span<'s, C>);

    /// Tracks an Err result of a parser function.    
    fn exit_err(&self, span: &Span<'s, C>, code: C, err: &dyn Error);
}

/// Hold the context.
/// Needed to block the debug implementation for LocatedSpan.
#[derive(Clone, Copy)]
pub struct HoldContext<'s, C: Code>(&'s dyn ParseContext<'s, C>);

impl<'s, C: Code> Deref for HoldContext<'s, C> {
    type Target = &'s dyn ParseContext<'s, C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s, C: Code> Debug for HoldContext<'s, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

///
/// Makes the Context hidden in the Span more accessible.
///
pub struct Context;

impl Context {
    /// Creates an Ok-Result from the parameters.
    /// Tracks an exit_ok with the ParseContext.
    pub fn ok<'s, C: Code, T, X: Copy>(
        &self,
        remainder: Span<'s, C>,
        parsed: Span<'s, C>,
        value: T,
    ) -> ParserResult<'s, T, C, X> {
        remainder.extra.exit_ok(&remainder, &parsed);
        Ok((remainder, value))
    }

    /// Creates a Err-ParserResult from the given ParserError.
    /// Tracks an exit_err with the ParseContext.
    fn err<'s, C: Code, T, X: Copy, E: Into<nom::Err<ParserError<'s, C, X>>>>(
        &self,
        err: E,
    ) -> ParserResult<'s, T, C, X> {
        let err: nom::Err<ParserError<'s, C, X>> = err.into();
        match &err {
            nom::Err::Incomplete(_) => {}
            nom::Err::Error(e) => e.span.extra.exit_err(&e.span, e.code, &e),
            nom::Err::Failure(e) => e.span.extra.exit_err(&e.span, e.code, &e),
        }
        Err(err)
    }
}

impl<'s, C: Code> ParseContext<'s, C> for Context {
    fn original(&self, span: &Span<'s, C>) -> Span<'s, C> {
        span.extra.original(span)
    }

    unsafe fn span_union(&self, first: &Span<'s, C>, second: &Span<'s, C>) -> Span<'s, C> {
        first.extra.span_union(first, second)
    }

    fn enter(&self, span: &Span<'s, C>, func: C) {
        span.extra.enter(span, func)
    }

    fn debug(&self, span: &Span<'s, C>, debug: String) {
        span.extra.debug(span, debug)
    }

    fn info(&self, span: &Span<'s, C>, info: &'static str) {
        span.extra.info(span, info)
    }

    fn warn(&self, span: &Span<'s, C>, warn: &'static str) {
        span.extra.warn(span, warn)
    }

    fn exit_ok(&self, span: &Span<'s, C>, parsed: &Span<'s, C>) {
        span.extra.exit_ok(span, parsed)
    }

    fn exit_err(&self, span: &Span<'s, C>, code: C, err: &dyn Error) {
        span.extra.exit_err(span, code, err)
    }
}

/// Tracks the error path with the context.
pub trait TrackParseErr<'s, 't, C: Code, X: Copy> {
    type Result;

    /// Track if this is an error.
    fn track(self) -> Self::Result;

    /// Track if this is an error. Set a new code too.
    fn track_as(self, code: C) -> Self::Result;

    /// Track if this is an error. And if this is ok.
    fn track_ok(self, parsed: Span<'s, C>) -> Self::Result;
}

/// Convert an external error into a ParserError.
pub trait WithSpan<'s, C: Code, R> {
    /// Convert an external error into a ParserError.
    /// Usually uses nom::Err::Failure to indicate the finality of the error.
    fn with_span(self, code: C, span: Span<'s, C>) -> R;
}

/// Translate the error code to a new one.
pub trait WithCode<'s, C: Code, R> {
    /// Translate the error code to a new one.
    fn with_code(self, code: C) -> R;
}
