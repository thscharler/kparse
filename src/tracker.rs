//!
//! Everything related to tracking in a parser.
//!

use crate::error::ParserError;
use crate::Code;
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
use std::fmt::{Debug, Formatter};
use std::ops::{RangeFrom, RangeTo};

mod std_tracker;

pub use std_tracker::*;

/// Standard input type for tracking.
///
/// This uses the LocatedSpan.extra field to pass the tracking context
/// through the parser.
pub type TrackSpan<'s, C, T> = LocatedSpan<T, DynTracker<'s, C, T>>;

/// Standard Result type for tracking.
/// Equivalent to [nom::IResult]<(I, O), ParserError<C, I>>
pub type TrackResult<C, I, O, Y> = Result<(I, O), nom::Err<ParserError<C, I, Y>>>;

/// This trait defines the tracker functions.
/// Create an [StdTracker] and use it's span() function to get the input for your
/// parser.
///
/// This trait is only used to implement the tracker, use [Context] to add tracking
/// to your parser.
pub trait Tracker<C, T>
where
    C: Code,
{
    /// Tracks entering a parser function.
    fn enter(&self, func: C, span: &LocatedSpan<T, ()>);

    /// Debugging
    fn debug(&self, span: &LocatedSpan<T, ()>, debug: String);

    /// Track something.
    fn info(&self, span: &LocatedSpan<T, ()>, info: &'static str);

    /// Track something more important.
    fn warn(&self, span: &LocatedSpan<T, ()>, warn: &'static str);

    /// Tracks an Ok result of a parser function.
    fn exit_ok(&self, span: &LocatedSpan<T, ()>, parsed: &LocatedSpan<T, ()>);

    /// Tracks an Err result of a parser function.    
    fn exit_err(&self, span: &LocatedSpan<T, ()>, code: C, err_str: String);
}

/// An instance of this struct ist kept in the extra field of LocatedSpan.
/// This way it's propagated all the way through the parser.
///
/// Access the tracking functions via [Context].
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DynTracker<'c, C, T>(pub(crate) &'c dyn Tracker<C, T>)
where
    C: Code;

impl<'c, C, T> Debug for DynTracker<'c, C, T>
where
    C: Code,
{
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

/// This trait is implemented on Context for various input types.
/// This allows it to switch seamlessly between input types.
pub trait FindTracker<C>
where
    C: Code,
    Self: Sized,
{
    /// Creates an Ok() Result from the parameters and tracks the result.
    fn ok<O, Y>(
        self,
        parsed: Self,
        value: O,
    ) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>>;

    /// Tracks the error and creates a Result.
    fn err<O, Y>(
        &self,
        err: nom::Err<ParserError<C, Self, Y>>,
    ) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>>
    where
        Y: Copy + Debug;

    /// Enter a parser function.
    fn enter(&self, func: C);

    /// Track some debug info.
    fn debug(&self, debug: String);

    /// Track some other info.
    fn info(&self, info: &'static str);

    /// Track some warning.
    fn warn(&self, warn: &'static str);

    /// Calls exit_ok() on the ParseContext. You might want to use ok() instead.
    fn exit_ok(&self, parsed: Self);

    /// Calls exit_err() on the ParseContext. You might want to use err() instead.
    fn exit_err<Y>(&self, code: C, err: &nom::Err<ParserError<C, Self, Y>>)
    where
        Y: Copy + Debug;
}

/// This trait is used for error tracking.
///
/// It is implemented for Result<(I,O), nom::Err<ParserError<>>, so it's
/// methods can be squeezed between the call to the parser and the ? operator.
///
/// Calls the tracking functions in the error case.
///
/// ```rust ignore
/// let (rest, h0) = nom_header(input).track_as(APCHeader)?;
/// let (rest, _) = nom_tag_plan(rest).track_as(APCPlan)?;
/// let (rest, plan) = token_name(rest).track()?;
/// let (rest, h1) = nom_header(rest).track_as(APCHeader)?;
/// ```
pub trait TrackError<C, I, O, E, Y>
where
    C: Code,
    I: Copy + FindTracker<C> + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    E: Into<ParserError<C, I, Y>>,
    Y: Copy + Debug,
{
    /// Keep a track if self is an error.
    fn track(self) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>>;

    /// Keep track if self is an error, and set an error code too.
    fn track_as(self, code: C) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>>;

    /// Keep track of self, either as error or as ok result.
    fn track_ok(self, parsed: I) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>>;
}

impl<C, I, O, E, Y> TrackError<C, I, O, E, Y> for Result<(I, O), nom::Err<E>>
where
    C: Code,
    I: Copy + FindTracker<C> + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    E: Into<ParserError<C, I, Y>>,
    Y: Copy + Debug,
{
    /// Keep a track if self is an error.
    fn track(self) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                let err: ParserError<C, I, Y> = e.into();
                let code = err.code;
                let span = err.span;
                let nom_err = nom::Err::Error(err);
                span.exit_err(code, &nom_err);
                Err(nom_err)
            }
        }
    }

    /// Keep track if self is an error, and set an error code too.
    fn track_as(self, code: C) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                let err: ParserError<C, I, Y> = e.into();
                let err = err.with_code(code);
                let code = err.code;
                let span = err.span;
                let nom_err = nom::Err::Error(err);
                span.exit_err(code, &nom_err);
                Err(nom_err)
            }
        }
    }

    /// Keep track of self, either as error or as ok result.
    fn track_ok(self, parsed: I) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        match self {
            Ok((span, v)) => {
                span.exit_ok(parsed);
                Ok((span, v))
            }
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                let err: ParserError<C, I, Y> = e.into();
                let code = err.code;
                let span = err.span;
                let nom_err = nom::Err::Error(err);
                span.exit_err(code, &nom_err);
                Err(nom_err)
            }
        }
    }
}
