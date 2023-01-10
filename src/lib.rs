//!
//! Additional functionality surrounding nom.
//!

use nom_locate::LocatedSpan;
use std::fmt::{Debug, Display};

mod conversion;
mod data_frame;
mod error;
mod tracker;

pub use conversion::*;
pub use data_frame::*;
pub use error::*;
pub use tracker::*;

/// Standard input type.
pub type Span<'s, C> = LocatedSpan<&'s str, &'s dyn ParseContext<'s, C>>;

/// Result type.
pub type ParserResult<'s, C, X, O> = Result<(Span<'s, C>, O), nom::Err<ParserError<'s, C, X>>>;

/// Type alias for a nom parser. Use this to create a ParserError directly in nom.
pub type ParserNomResult<'s, C, X> =
    Result<(Span<'s, C>, Span<'s, C>), nom::Err<ParserError<'s, C, X>>>;

/// Parser state codes.
///
/// These are used for error handling and parser results and
/// everything else.
pub trait Code: Copy + Display + Debug + Eq {}

pub trait ParseContext<'s, C: Code> {
    /// Returns a span that encloses all of the current parser.
    fn span(&self) -> Span<'s, C>;

    /// Tracks entering a parser function.
    fn enter(span: Span<'s, C>, func: C);

    /// Tracks a result of a parser function.
    fn exit<X: Copy, O>(result: ParserResult<'s, C, X, O>) -> ParserResult<'s, C, X, O>;

    /// Tracks a result of a parser function.
    /// Only jumps the hoop on the Ok branch.
    fn exit_ok<X: Copy, O>(result: ParserResult<'s, C, X, O>) -> ParserResult<'s, C, X, O>;

    /// Tracks a result of a parser function.
    /// Only jumps the hoop on the Err branch.
    fn exit_err<X: Copy, O>(result: ParserResult<'s, C, X, O>) -> ParserResult<'s, C, X, O>;
}

/// Tracks the error path with the context.
pub trait TrackParseErr<'s, 't, C: Code, X: Copy> {
    type Result;

    /// Track if this is an error.
    fn track(self) -> Self::Result;

    /// Track if this is an error. Set a new code too.
    fn track_as(self, code: C) -> Self::Result;

    /// Track if this is an error. And if this is ok.
    fn track_ok(self) -> Self::Result;
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
