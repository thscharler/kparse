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

    /// When multiple Context.enter() calls are used within one function
    /// (to denote some separation), this can be used to exit such a compartment
    /// with an ok track.
    pub fn err_section<C, I, Y>(&self, rest: I, code: C, err: &nom::Err<ParserError<C, I, Y>>)
    where
        C: Code,
        I: FindTracker<C>,
        Y: Copy + Debug,
    {
        rest.exit_err(code, err);
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
        parsed: Self,
        value: O,
    ) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>> {
        self.extra
            .0
            .exit_ok(&clear_span(&self), &clear_span(&parsed));
        Ok((self, value))
    }

    fn err<O, Y>(
        &self,
        err: nom::Err<ParserError<C, Self, Y>>,
    ) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>>
    where
        Y: Copy + Debug,
    {
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

    fn exit_err<Y>(&self, code: C, err: &nom::Err<ParserError<C, Self, Y>>)
    where
        Y: Copy + Debug,
    {
        self.extra
            .0
            .exit_err(&clear_span(self), code, err.to_string());
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
        _parsed: Self,
        value: O,
    ) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>> {
        Ok((self, value))
    }

    fn err<O, Y>(
        &self,
        err: nom::Err<ParserError<C, Self, Y>>,
    ) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>>
    where
        Y: Copy + Debug,
    {
        Err(err)
    }

    fn enter(&self, _func: C) {}

    fn debug(&self, _debug: String) {}

    fn info(&self, _info: &'static str) {}

    fn warn(&self, _warn: &'static str) {}

    fn exit_ok(&self, _parsed: PlainSpan<'s, T>) {}

    fn exit_err<Y>(&self, _func: C, _err: &nom::Err<ParserError<C, Self, Y>>)
    where
        Y: Copy + Debug,
    {
    }
}

impl<'s, C> FindTracker<C> for &'s str
where
    C: Code,
{
    fn ok<O, Y>(
        self,
        _parsed: Self,
        value: O,
    ) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>> {
        Ok((self, value))
    }

    fn err<O, Y>(
        &self,
        err: nom::Err<ParserError<C, Self, Y>>,
    ) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>>
    where
        Y: Copy + Debug,
    {
        Err(err)
    }

    fn enter(&self, _func: C) {}

    fn debug(&self, _debug: String) {}

    fn info(&self, _info: &'static str) {}

    fn warn(&self, _warn: &'static str) {}

    fn exit_ok(&self, _input: Self) {}

    fn exit_err<Y>(&self, _func: C, _err: &nom::Err<ParserError<C, Self, Y>>)
    where
        Y: Copy + Debug,
    {
    }
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

    fn err<O, Y>(
        &self,
        err: nom::Err<ParserError<C, Self, Y>>,
    ) -> Result<(Self, O), nom::Err<ParserError<C, Self, Y>>>
    where
        Y: Copy + Debug,
    {
        Err(err)
    }

    fn enter(&self, _func: C) {}

    fn debug(&self, _debug: String) {}

    fn info(&self, _info: &'static str) {}

    fn warn(&self, _warn: &'static str) {}

    fn exit_ok(&self, _input: Self) {}

    fn exit_err<Y>(&self, _func: C, _err: &nom::Err<ParserError<C, Self, Y>>)
    where
        Y: Copy + Debug,
    {
    }
}
