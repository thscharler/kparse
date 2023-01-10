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
pub type Span<'s> = LocatedSpan<&'s str>;

/// Result type.
pub type ParserResult<'s, C, X, O> = Result<O, ParserError<'s, C, X>>;

/// Type alias for a nom parser. Use this to create a ParserError directly in nom.
pub type ParserNomResult<'s, C, X> = Result<(Span<'s>, Span<'s>), nom::Err<ParserError<'s, C, X>>>;

/// Parser state codes.
///
/// These are used for error handling and parser results and
/// everything else.
pub trait Code: Copy + Display + Debug + Eq {
    /// Mapping for nom::Err::Error
    const NOM_ERROR: Self;
    /// Mapping for nom::Err::Failure
    const NOM_FAILURE: Self;
    /// Mapping for nom::Err::Incomplete
    const NOM_INCOMPLETE: Self;

    fn is_nom_special(&self) -> bool {
        *self == Self::NOM_ERROR || *self == Self::NOM_FAILURE || *self == Self::NOM_INCOMPLETE
    }
}

pub trait ParseContext<'s, C: Code> {
    // Returns a span that encloses all of the current parser.
    fn complete(&self) -> Span<'s>;

    // Tracks entering a parser function.
    fn enter(&mut self, func: C, span: Span<'s>);

    // Tracks an ok result of a parser function.
    fn ok<X: Copy, O>(
        &mut self,
        rest: Span<'s>,
        span: Span<'s>,
        value: O,
    ) -> ParserResult<'s, C, X, (Span<'s>, O)>;

    fn err<X: Copy, O>(&mut self, err: ParserError<'s, C, X>) -> ParserResult<'s, C, X, O>;
}

/// Tracks the error path with the context.
///
pub trait TrackParseErr<'s, 't, C: Code, X: Copy> {
    type Result;

    fn track(self, ctx: &'t mut impl ParseContext<'s, C>) -> Self::Result;

    fn track_as(self, ctx: &'t mut impl ParseContext<'s, C>, code: C) -> Self::Result;
}

/// Convert an external error into a ParserError.
pub trait WithSpan<'s, C: Code, R> {
    /// Convert an external error into a ParserError.
    fn with_span(self, code: C, span: Span<'s>) -> R;
}

/// Translate the error code to a new one.
pub trait WithCode<'s, C: Code, R> {
    /// Translate the error code to a new one.
    fn with_code(self, code: C) -> R;
}
