//!
//! Provides [Context] to access the tracker.
//!

use crate::tracker::{DynTracker, FindTracker};
use crate::{Code, ParserError};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
use std::error::Error;
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};

/// Provides access to the tracker functions for various input types.
pub struct Context;

type DynSpan<'s, C, T> = LocatedSpan<T, DynTracker<'s, C, T>>;

impl<'s, T, C> FindTracker<C, DynSpan<'s, C, T>> for Context
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
        &self,
        remainder: DynSpan<'s, C, T>,
        parsed: DynSpan<'s, C, T>,
        value: O,
    ) -> Result<(DynSpan<'s, C, T>, O), nom::Err<ParserError<C, DynSpan<'s, C, T>, Y>>>
    where
        Y: Copy,
    {
        Context.exit_ok(remainder, parsed);
        Ok((remainder, value))
    }

    fn err<O, E, Y>(
        &self,
        err: E,
    ) -> Result<(DynSpan<'s, C, T>, O), nom::Err<ParserError<C, DynSpan<'s, C, T>, Y>>>
    where
        E: Into<nom::Err<ParserError<C, DynSpan<'s, C, T>, Y>>>,
        Y: Copy + Debug,
    {
        let err: nom::Err<ParserError<C, DynSpan<'s, C, T>, Y>> = err.into();
        match &err {
            nom::Err::Incomplete(_) => {}
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                Context.exit_err(e.span, e.code, &e);
            }
        }
        Err(err)
    }

    fn enter(&self, func: C, span: DynSpan<'s, C, T>) {
        span.extra.0.enter(func, &clear_span(&span))
    }

    fn debug(&self, span: DynSpan<'s, C, T>, debug: String) {
        span.extra.0.debug(&clear_span(&span), debug)
    }

    fn info(&self, span: DynSpan<'s, C, T>, info: &'static str) {
        span.extra.0.info(&clear_span(&span), info)
    }

    fn warn(&self, span: DynSpan<'s, C, T>, warn: &'static str) {
        span.extra.0.warn(&clear_span(&span), warn)
    }

    fn exit_ok(&self, span: DynSpan<'s, C, T>, parsed: DynSpan<'s, C, T>) {
        span.extra
            .0
            .exit_ok(&clear_span(&span), &clear_span(&parsed))
    }

    fn exit_err(&self, span: DynSpan<'s, C, T>, code: C, err: &dyn Error) {
        span.extra.0.exit_err(&clear_span(&span), code, err)
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

impl<'s, T, C> FindTracker<C, PlainSpan<'s, T>> for Context
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
        &self,
        remainder: PlainSpan<'s, T>,
        _parsed: PlainSpan<'s, T>,
        value: O,
    ) -> Result<(PlainSpan<'s, T>, O), nom::Err<ParserError<C, PlainSpan<'s, T>, Y>>>
    where
        Y: Copy,
    {
        Ok((remainder, value))
    }

    fn err<O, E, Y>(
        &self,
        err: E,
    ) -> Result<(PlainSpan<'s, T>, O), nom::Err<ParserError<C, PlainSpan<'s, T>, Y>>>
    where
        E: Into<nom::Err<ParserError<C, PlainSpan<'s, T>, Y>>>,
        Y: Copy,
    {
        Err(err.into())
    }

    fn enter(&self, _func: C, _span: PlainSpan<'s, T>) {}

    fn debug(&self, _span: PlainSpan<'s, T>, _debug: String) {}

    fn info(&self, _span: PlainSpan<'s, T>, _info: &'static str) {}

    fn warn(&self, _span: PlainSpan<'s, T>, _warn: &'static str) {}

    fn exit_ok(&self, _span: PlainSpan<'s, T>, _parsed: PlainSpan<'s, T>) {}

    fn exit_err(&self, _span: PlainSpan<'s, T>, _code: C, _err: &dyn Error) {}
}

impl<'s, C> FindTracker<C, &'s str> for Context
where
    C: Code,
{
    fn ok<O, Y>(
        &self,
        remainder: &'s str,
        _parsed: &'s str,
        value: O,
    ) -> Result<(&'s str, O), nom::Err<ParserError<C, &'s str, Y>>>
    where
        Y: Copy,
        C: Code,
    {
        Ok((remainder, value))
    }

    fn err<O, E, Y>(&self, err: E) -> Result<(&'s str, O), nom::Err<ParserError<C, &'s str, Y>>>
    where
        E: Into<nom::Err<ParserError<C, &'s str, Y>>>,
        Y: Copy,
    {
        Err(err.into())
    }

    fn enter(&self, _func: C, _span: &'s str) {}

    fn debug(&self, _span: &'s str, _debug: String) {}

    fn info(&self, _span: &'s str, _info: &'static str) {}

    fn warn(&self, _span: &'s str, _warn: &'static str) {}

    fn exit_ok(&self, _span: &'s str, _parsed: &'s str) {}

    fn exit_err(&self, _span: &'s str, _code: C, _err: &dyn Error) {}
}

impl<'s, C> FindTracker<C, &'s [u8]> for Context
where
    C: Code,
{
    fn ok<O, Y>(
        &self,
        remainder: &'s [u8],
        _parsed: &'s [u8],
        value: O,
    ) -> Result<(&'s [u8], O), nom::Err<ParserError<C, &'s [u8], Y>>>
    where
        Y: Copy,
        C: Code,
    {
        Ok((remainder, value))
    }

    fn err<O, E, Y>(&self, err: E) -> Result<(&'s [u8], O), nom::Err<ParserError<C, &'s [u8], Y>>>
    where
        E: Into<nom::Err<ParserError<C, &'s [u8], Y>>>,
        Y: Copy,
    {
        Err(err.into())
    }

    fn enter(&self, _func: C, _span: &'s [u8]) {}

    fn debug(&self, _span: &'s [u8], _debug: String) {}

    fn info(&self, _span: &'s [u8], _info: &'static str) {}

    fn warn(&self, _span: &'s [u8], _warn: &'static str) {}

    fn exit_ok(&self, _span: &'s [u8], _parsed: &'s [u8]) {}

    fn exit_err(&self, _span: &'s [u8], _code: C, _err: &dyn Error) {}
}
