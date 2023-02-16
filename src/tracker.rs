//!
//! Everything related to tracking in a parser.
//!

use crate::error::ParserError;
use crate::Code;
use nom::{AsBytes, InputLength, InputTake};
use nom_locate::LocatedSpan;
use std::fmt::{Debug, Formatter};

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
    fn ok<O, E>(self, parsed: Self, value: O) -> Result<(Self, O), nom::Err<E>>;

    /// Tracks the error and creates a Result.
    fn err<O, E: Debug>(&self, code: C, err: nom::Err<E>) -> Result<(Self, O), nom::Err<E>>;

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
    fn exit_err<E: Debug>(&self, code: C, err: &nom::Err<E>);
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
pub trait TrackError<C, I>
where
    C: Code,
    I: Copy + Debug,
    I: FindTracker<C>,
    I: InputTake + InputLength + AsBytes,
{
    /// Keep a track if self is an error.
    fn track(self) -> Self;

    /// Keep track if self is an error, and set an error code too.
    fn track_as(self, code: C) -> Self;

    /// Keep track of self, either as error or as ok result.
    fn track_ok(self, parsed: I) -> Self;
}

impl<C, I, O, Y> TrackError<C, I> for Result<(I, O), nom::Err<ParserError<C, I, Y>>>
where
    C: Code,
    I: Copy + Debug,
    I: FindTracker<C>,
    I: InputTake + InputLength + AsBytes,
    Y: Copy + Debug,
{
    /// Keep a track if self is an error.
    fn track(self) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let span = e.span;
                let code = e.code;
                let err = nom::Err::Error(e);
                span.exit_err(code, &err);
                Err(err)
            }
            Err(nom::Err::Failure(e)) => {
                let span = e.span;
                let code = e.code;
                let err = nom::Err::Failure(e);
                span.exit_err(code, &err);
                Err(err)
            }
        }
    }

    /// Keep track if self is an error, and set an error code too.
    fn track_as(self, code: C) -> Self {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let e = e.with_code(code);
                let span = e.span;
                let err = nom::Err::Error(e);
                span.exit_err(code, &err);
                Err(err)
            }
            Err(nom::Err::Failure(e)) => {
                let e = e.with_code(code);
                let span = e.span;
                let err = nom::Err::Failure(e);
                span.exit_err(code, &err);
                Err(err)
            }
        }
    }

    /// Keep track of self, either as error or as ok result.
    fn track_ok(self, parsed: I) -> Self {
        match self {
            Ok((span, v)) => {
                span.exit_ok(parsed);
                Ok((span, v))
            }
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let span = e.span;
                let code = e.code;
                let err = nom::Err::Error(e);
                span.exit_err(code, &err);
                Err(err)
            }
            Err(nom::Err::Failure(e)) => {
                let span = e.span;
                let code = e.code;
                let err = nom::Err::Failure(e);
                span.exit_err(code, &err);
                Err(err)
            }
        }
    }
}
