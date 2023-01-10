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

/// Standard input type.
pub type Span<'s, C> = LocatedSpan<&'s str, HoldContext<'s, C>>;

/// Result type.
pub type ParserResult<'s, C, X, O> = Result<(Span<'s, C>, O), nom::Err<ParserError<'s, C, X>>>;

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
    fn span(&self) -> &Span<'s, C>;

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
/// Make the ParseContext accessible for a Span.
///
pub trait ParseContextForSpan<'s, C: Code> {
    /// Returns a span that encloses all of the current parser.
    fn span(&self) -> &Span<'s, C>;

    /// Tracks entering a parser function.
    fn enter(&self, func: C);

    /// Debugging
    fn debug(&self, debug: String);

    /// Track something.
    fn info(&self, info: &'static str);

    /// Track something more important.
    fn warn(&self, warn: &'static str);

    /// Tracks an Ok result of a parser function.
    fn exit_ok(&self, parsed: &Span<'s, C>);

    /// Creates and tracks an Ok result of a parser function.
    fn ok<T, X: Copy>(&self, parsed: Span<'s, C>, value: T) -> ParserResult<'s, C, X, T>;

    /// Tracks an Err result of a parser function.    
    fn exit_err(&self, code: C, err: &dyn Error);

    /// Creates and tracks an Err result of a parser function.
    /// This creates a nom::Err::Error variant.
    fn err<T, X: Copy>(&self, err: ParserError<'s, C, X>) -> ParserResult<'s, C, X, T>;

    /// Creates and tracks an Err result of a parser function.
    fn err_nom<T, X: Copy>(
        &self,
        err: nom::Err<ParserError<'s, C, X>>,
    ) -> ParserResult<'s, C, X, T>;
}

impl<'s, C: Code> ParseContextForSpan<'s, C> for Span<'s, C> {
    fn span(&self) -> &Span<'s, C> {
        self.extra.span()
    }

    fn enter(&self, func: C) {
        self.extra.enter(self, func);
    }

    fn debug(&self, debug: String) {
        self.extra.debug(self, debug);
    }

    fn info(&self, info: &'static str) {
        self.extra.info(self, info);
    }

    fn warn(&self, warn: &'static str) {
        self.extra.warn(self, warn);
    }

    fn exit_ok(&self, parsed: &Span<'s, C>) {
        self.extra.exit_ok(self, parsed);
    }

    fn ok<T, X: Copy>(&self, parsed: Span<'s, C>, value: T) -> ParserResult<'s, C, X, T> {
        self.extra.exit_ok(&self, &parsed);
        Ok((*self, value))
    }

    fn exit_err(&self, code: C, err: &dyn Error) {
        self.extra.exit_err(self, code, err)
    }

    fn err<T, X: Copy>(&self, err: ParserError<'s, C, X>) -> ParserResult<'s, C, X, T> {
        self.extra.exit_err(&self, err.code, &err);
        Err(nom::Err::Error(err))
    }

    fn err_nom<T, X: Copy>(
        &self,
        err: nom::Err<ParserError<'s, C, X>>,
    ) -> ParserResult<'s, C, X, T> {
        match &err {
            nom::Err::Incomplete(_) => {}
            nom::Err::Error(e) => self.extra.exit_err(&self, e.code, &e),
            nom::Err::Failure(e) => self.extra.exit_err(&self, e.code, &e),
        }
        Err(err)
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
