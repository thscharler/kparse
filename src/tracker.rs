//!
//! Everything related to tracking in a parser.
//!

use crate::context::Context;
use crate::error::ParserError;
use crate::Code;
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
use std::error::Error;
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
/// Equivalent to [nom::IResult]<(T, O), Err<ParserError<..>>>
pub type TrackParserResult<'s, C, T, Y, O> =
    Result<(TrackSpan<'s, C, T>, O), nom::Err<ParserError<C, TrackSpan<'s, C, T>, Y>>>; // todo move Y to end

/// Standard Result type for tracking, if the result is a simple span.
/// Equivalent to [nom::IResult]<(I,I), Err<ParserError<..>>
pub type TrackParserResultSpan<'s, C, T, Y> = Result<
    (TrackSpan<'s, C, T>, TrackSpan<'s, C, T>),
    nom::Err<ParserError<C, TrackSpan<'s, C, T>, Y>>,
>;

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
    fn exit_err(&self, span: &LocatedSpan<T, ()>, code: C, err: &dyn Error);
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
pub trait ContextTrait<C, I>
where
    I: Copy + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    C: Code,
{
    // todo: move impl to context directly
    /// Creates an Ok() Result from the parameters and tracks the result.
    fn ok<O, Y>(
        &self,
        remainder: I,
        parsed: I,
        value: O,
    ) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>>
    where
        Y: Copy,
    {
        self.exit_ok(remainder, parsed);
        Ok((remainder, value))
    }

    // todo: move impl to context directly
    /// Tracks the error and creates a Result.
    fn err<O, Y, E>(&self, err: E) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>>
    where
        E: Into<nom::Err<ParserError<C, I, Y>>>,
        Y: Copy,
        C: Code,
    {
        let err: nom::Err<ParserError<C, I, Y>> = err.into();
        match &err {
            nom::Err::Incomplete(_) => {}
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                self.exit_err(e.span, e.code, &e);
            }
        }
        Err(err)
    }

    /// Enter a parser function.
    fn enter(&self, func: C, span: I);

    /// Track some debug info.
    fn debug(&self, span: I, debug: String);

    /// Track some other info.
    fn info(&self, span: I, info: &'static str);

    /// Track some warning.
    fn warn(&self, span: I, warn: &'static str);

    /// Calls exit_ok() on the ParseContext. You might want to use ok() instead.
    fn exit_ok(&self, span: I, parsed: I);

    /// Calls exit_err() on the ParseContext. You might want to use err() instead.
    fn exit_err(&self, span: I, code: C, err: &dyn Error);
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
pub trait TrackParserError<'s, C, I, Y, O, E>
where
    C: Code,
    I: AsBytes + Copy + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    Y: Copy,
    E: Into<ParserError<C, I, Y>>,
    Self: Into<Result<(I, O), nom::Err<E>>>,
{
    /// Keep a track if self is an error.
    fn track(self) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        let ego = self.into();
        match ego {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<C, I, Y> = e.into();
                Self::exit_err(p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    /// Keep track if self is an error, and set an error code too.
    fn track_as(self, code: C) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        let ego = self.into();
        match ego {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<C, I, Y> = e.into();
                let p_err = p_err.with_code(code);
                Self::exit_err(p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    /// Keep track of self, either as error or as ok result.
    fn track_ok(self, parsed: I) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        let ego = self.into();
        match ego {
            Ok((span, v)) => {
                Self::exit_ok(parsed, span);
                Ok((span, v))
            }
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<C, I, Y> = e.into();
                Self::exit_err(p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    /// Used to implement the bridge to [Context].
    fn exit_ok(span: I, parsed: I);

    /// Used to implement the bridge to [Context].
    fn exit_err(span: I, code: C, err: &dyn Error);
}

impl<'s, C, Y, O, E> TrackParserError<'s, C, &'s str, Y, O, E> for Result<(&'s str, O), nom::Err<E>>
where
    E: Into<ParserError<C, &'s str, Y>>,
    C: Code,
    Y: Copy,
{
    fn exit_ok(_span: &'s str, _parsed: &'s str) {}

    fn exit_err(_span: &'s str, _code: C, _err: &dyn Error) {}
}

impl<'s, C, Y, O, E> TrackParserError<'s, C, &'s [u8], Y, O, E>
    for Result<(&'s [u8], O), nom::Err<E>>
where
    E: Into<ParserError<C, &'s [u8], Y>>,
    C: Code,
    Y: Copy,
{
    fn exit_ok(_span: &'s [u8], _parsed: &'s [u8]) {}

    fn exit_err(_span: &'s [u8], _code: C, _err: &dyn Error) {}
}

impl<'s, C, T, Y, O, E> TrackParserError<'s, C, LocatedSpan<T, ()>, Y, O, E>
    for Result<(LocatedSpan<T, ()>, O), nom::Err<E>>
where
    E: Into<ParserError<C, LocatedSpan<T, ()>, Y>>,
    C: Code,
    Y: Copy,
    T: Copy + Debug,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    fn exit_ok(_span: LocatedSpan<T, ()>, _parsed: LocatedSpan<T, ()>) {}

    fn exit_err(_span: LocatedSpan<T, ()>, _code: C, _err: &dyn Error) {}
}

impl<'s, C, T, Y, O, E> TrackParserError<'s, C, LocatedSpan<T, DynTracker<'s, C, T>>, Y, O, E>
    for Result<(LocatedSpan<T, DynTracker<'s, C, T>>, O), nom::Err<E>>
where
    E: Into<ParserError<C, LocatedSpan<T, DynTracker<'s, C, T>>, Y>>,
    C: Code,
    Y: Copy,
    T: Copy + Debug,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    fn exit_ok(
        span: LocatedSpan<T, DynTracker<'s, C, T>>,
        parsed: LocatedSpan<T, DynTracker<'s, C, T>>,
    ) {
        <Context as ContextTrait<C, LocatedSpan<T, DynTracker<'s, C, T>>>>::exit_ok(
            &Context, span, parsed,
        );
    }

    fn exit_err(span: LocatedSpan<T, DynTracker<'s, C, T>>, code: C, err: &dyn Error) {
        <Context as ContextTrait<C, LocatedSpan<T, DynTracker<'s, C, T>>>>::exit_err(
            &Context, span, code, err,
        );
    }
}
