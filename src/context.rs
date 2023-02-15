//!
//! Provides [Context] to access the tracker.
//!

use crate::tracker::{DynTracker, FindTracker};
use crate::{Code, ParserError};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};

/// Provides access to the tracker functions for various input types.
pub struct Context;

impl Context {
    /// Creates an Ok() Result from the parameters and tracks the result.
    pub fn ok<C, I, O, Y>(
        &self,
        rest: I,
        input: I,
        value: O,
    ) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>>
    where
        C: Code,
        I: FindTracker<C>,
    {
        rest.ok(input, value)
    }

    /// When multiple Context.enter() calls are used within one function
    /// (to denote some separation), this can be used to exit such a compartment
    /// with an ok track.
    pub fn ok_section<C, I>(&self, rest: I, input: I)
    where
        C: Code,
        I: FindTracker<C>,
    {
        rest.exit_ok(input);
    }

    /// Tracks the error and creates a Result.
    pub fn err<C, I, O, E, Y>(&self, err: E) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>>
    where
        C: Code,
        I: Copy + FindTracker<C>,
        E: Into<nom::Err<ParserError<C, I, Y>>>,
        Y: Copy + Debug,
    {
        let err: nom::Err<ParserError<C, I, Y>> = err.into();
        let span = match &err {
            nom::Err::Incomplete(_) => return Err(err),
            nom::Err::Error(e) | nom::Err::Failure(e) => e.span,
        };
        span.err(err)
    }

    /// Enter a parser function.
    pub fn enter<C, I>(&self, func: C, span: I)
    where
        C: Code,
        I: FindTracker<C>,
    {
        span.enter(func);
    }

    /// Track some debug info.
    pub fn debug<C, I>(&self, span: I, debug: String)
    where
        C: Code,
        I: FindTracker<C>,
    {
        span.debug(debug);
    }

    /// Track some other info.
    pub fn info<C, I>(&self, span: I, info: &'static str)
    where
        C: Code,
        I: FindTracker<C>,
    {
        span.info(info);
    }

    /// Track some warning.
    pub fn warn<C, I>(&self, span: I, warn: &'static str)
    where
        C: Code,
        I: FindTracker<C>,
    {
        span.warn(warn);
    }
}

type DynSpan<'s, C, T> = LocatedSpan<T, DynTracker<'s, C, T>>;

impl<'s, C, T> FindTracker<C> for DynSpan<'s, C, T>
where
    T: Copy + Debug,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    C: Code,
{
    fn ok<O, Y>(
        self,
        parsed: DynSpan<'s, C, T>,
        value: O,
    ) -> Result<(DynSpan<'s, C, T>, O), nom::Err<ParserError<C, DynSpan<'s, C, T>, Y>>> {
        self.extra
            .0
            .exit_ok(&clear_span(&self), &clear_span(&parsed));
        Ok((self, value))
    }

    fn err<O, E, Y>(
        &self,
        err: E,
    ) -> Result<(DynSpan<'s, C, T>, O), nom::Err<ParserError<C, Self, Y>>>
    where
        E: Into<nom::Err<ParserError<C, DynSpan<'s, C, T>, Y>>>,
        Y: Copy + Debug,
    {
        let err: nom::Err<ParserError<C, DynSpan<'s, C, T>, Y>> = err.into();
        match &err {
            nom::Err::Incomplete(_) => {}
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                self.extra
                    .0
                    .exit_err(&clear_span(&e.span), e.code, e.to_string());
            }
        }
        Err(err)
    }

    fn enter(&self, func: C) {
        self.extra.0.enter(func, &clear_span(self));
    }

    fn debug(&self, debug: String) {
        self.extra.0.debug(&clear_span(self), debug);
    }

    fn info(&self, info: &'static str) {
        self.extra.0.info(&clear_span(self), info);
    }

    fn warn(&self, warn: &'static str) {
        self.extra.0.warn(&clear_span(self), warn);
    }

    fn exit_ok(&self, parsed: DynSpan<'s, C, T>) {
        self.extra
            .0
            .exit_ok(&clear_span(self), &clear_span(&parsed));
    }

    fn exit_err(&self, code: C, err: String) {
        self.extra.0.exit_err(&clear_span(self), code, err);
    }
}

fn clear_span<C, T>(span: &DynSpan<'_, C, T>) -> LocatedSpan<T, ()>
where
    C: Code,
    T: AsBytes + Copy,
{
    unsafe {
        LocatedSpan::new_from_raw_offset(
            span.location_offset(),
            span.location_line(),
            *span.fragment(),
            (),
        )
    }
}

type PlainSpan<'s, T> = LocatedSpan<T, ()>;

impl<'s, C, T> FindTracker<C> for PlainSpan<'s, T>
where
    T: Copy + Debug,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    C: Code,
{
    fn ok<O, Y>(
        self,
        _parsed: PlainSpan<'s, T>,
        value: O,
    ) -> Result<(PlainSpan<'s, T>, O), nom::Err<ParserError<C, PlainSpan<'s, T>, Y>>> {
        Ok((self, value))
    }

    fn err<O, E, Y>(
        &self,
        err: E,
    ) -> Result<(PlainSpan<'s, T>, O), nom::Err<ParserError<C, PlainSpan<'s, T>, Y>>>
    where
        E: Into<nom::Err<ParserError<C, PlainSpan<'s, T>, Y>>>,
        Y: Copy + Debug,
    {
        Err(err.into())
    }

    fn enter(&self, _func: C) {}

    fn debug(&self, _debug: String) {}

    fn info(&self, _info: &'static str) {}

    fn warn(&self, _warn: &'static str) {}

    fn exit_ok(&self, _parsed: PlainSpan<'s, T>) {}

    fn exit_err(&self, _func: C, _err: String) {}
}

impl<'s, C> FindTracker<C> for &'s str
where
    C: Code,
{
    fn ok<O, Y>(
        self,
        _parsed: &'s str,
        value: O,
    ) -> Result<(&'s str, O), nom::Err<ParserError<C, &'s str, Y>>> {
        Ok((self, value))
    }

    fn err<O, E, Y>(&self, err: E) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>>
    where
        E: Into<nom::Err<ParserError<C, Self, Y>>>,
        Y: Copy + Debug,
    {
        Err(err.into())
    }

    fn enter(&self, _func: C) {}

    fn debug(&self, _debug: String) {}

    fn info(&self, _info: &'static str) {}

    fn warn(&self, _warn: &'static str) {}

    fn exit_ok(&self, _input: Self) {}

    fn exit_err(&self, _func: C, _err: String) {}
}

impl<'s, C> FindTracker<C> for &'s [u8]
where
    C: Code,
{
    fn ok<O, Y>(
        self,
        _input: Self,
        value: O,
    ) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>> {
        Ok((self, value))
    }

    fn err<O, E, Y>(&self, err: E) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>>
    where
        E: Into<nom::Err<ParserError<C, Self, Y>>>,
        Y: Copy + Debug,
    {
        Err(err.into())
    }

    fn enter(&self, _func: C) {}

    fn debug(&self, _debug: String) {}

    fn info(&self, _info: &'static str) {}

    fn warn(&self, _warn: &'static str) {}

    fn exit_ok(&self, _input: Self) {}

    fn exit_err(&self, _func: C, _err: String) {}
}
