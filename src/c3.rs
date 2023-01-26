use crate::{Code, DynContext, ParserError};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
use std::error::Error;
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};

pub struct C3;

pub trait CCC<C, I>
where
    C: Code,
    I: Copy,
{
    fn ok<O, Y>(
        &self,
        remainder: I,
        parsed: I,
        value: O,
    ) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>>
    where
        Y: Copy;

    fn err<O, Y, E>(&self, err: E) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>>
    where
        E: Into<nom::Err<ParserError<C, I, Y>>>,
        Y: Copy,
        C: Code;

    fn enter(&self, func: C, span: I);

    fn exit_ok(&self, span: I, parsed: I);

    fn exit_err(&self, span: I, code: C, err: &dyn Error);
}

impl<'s, C> CCC<C, &'s str> for C3
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

    fn err<O, Y, E>(&self, err: E) -> Result<(&'s str, O), nom::Err<ParserError<C, &'s str, Y>>>
    where
        E: Into<nom::Err<ParserError<C, &'s str, Y>>>,
        Y: Copy,
    {
        Err(err.into())
    }

    fn enter(&self, _func: C, _span: &'s str) {
        //
    }

    fn exit_ok(&self, _span: &'s str, _parsed: &'s str) {
        //
    }

    fn exit_err(&self, _span: &'s str, _code: C, _err: &dyn Error) {
        //
    }
}

impl<'s, C> CCC<C, &'s [u8]> for C3
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

    fn err<O, Y, E>(&self, err: E) -> Result<(&'s [u8], O), nom::Err<ParserError<C, &'s [u8], Y>>>
    where
        E: Into<nom::Err<ParserError<C, &'s [u8], Y>>>,
        Y: Copy,
    {
        Err(err.into())
    }

    fn enter(&self, _func: C, _span: &'s [u8]) {
        //
    }

    fn exit_ok(&self, _span: &'s [u8], _parsed: &'s [u8]) {
        //
    }

    fn exit_err(&self, _span: &'s [u8], _code: C, _err: &dyn Error) {
        //
    }
}

impl<'s, T, C> CCC<C, LocatedSpan<T, DynContext<'s, T, C>>> for C3
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
        remainder: LocatedSpan<T, DynContext<'s, T, C>>,
        parsed: LocatedSpan<T, DynContext<'s, T, C>>,
        value: O,
    ) -> Result<
        (LocatedSpan<T, DynContext<'s, T, C>>, O),
        nom::Err<ParserError<C, LocatedSpan<T, DynContext<'s, T, C>>, Y>>,
    >
    where
        Y: Copy,
    {
        C3.exit_ok(remainder, parsed);
        Ok((remainder, value))
    }

    fn err<O, Y, E>(
        &self,
        err: E,
    ) -> Result<
        (LocatedSpan<T, DynContext<'s, T, C>>, O),
        nom::Err<ParserError<C, LocatedSpan<T, DynContext<'s, T, C>>, Y>>,
    >
    where
        E: Into<nom::Err<ParserError<C, LocatedSpan<T, DynContext<'s, T, C>>, Y>>>,
        Y: Copy,
    {
        let err: nom::Err<ParserError<C, LocatedSpan<T, DynContext<'s, T, C>>, Y>> = err.into();
        match &err {
            nom::Err::Incomplete(_) => {}
            nom::Err::Error(e) => C3.exit_err(e.span, e.code, &e),
            nom::Err::Failure(e) => C3.exit_err(e.span, e.code, &e),
        }
        Err(err)
    }

    fn enter(&self, func: C, span: LocatedSpan<T, DynContext<'s, T, C>>) {
        if let Some(ctx) = span.extra.0 {
            ctx.enter(func, &clear_span(span))
        }
    }

    fn exit_ok(
        &self,
        span: LocatedSpan<T, DynContext<'s, T, C>>,
        parsed: LocatedSpan<T, DynContext<'s, T, C>>,
    ) {
        if let Some(ctx) = span.extra.0 {
            ctx.exit_ok(&clear_span(span), &clear_span(parsed))
        }
    }

    fn exit_err(&self, span: LocatedSpan<T, DynContext<'s, T, C>>, code: C, err: &dyn Error) {
        if let Some(ctx) = span.extra.0 {
            ctx.exit_err(&clear_span(span), code, err)
        }
    }
}

fn clear_span<T, C>(span: LocatedSpan<T, DynContext<'_, T, C>>) -> LocatedSpan<T, ()>
where
    T: AsBytes + Copy,
    C: Code,
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

impl<T, C> CCC<C, LocatedSpan<T, ()>> for C3
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
        remainder: LocatedSpan<T, ()>,
        _parsed: LocatedSpan<T, ()>,
        value: O,
    ) -> Result<(LocatedSpan<T, ()>, O), nom::Err<ParserError<C, LocatedSpan<T, ()>, Y>>>
    where
        Y: Copy,
    {
        Ok((remainder, value))
    }

    fn err<O, Y, E>(
        &self,
        err: E,
    ) -> Result<(LocatedSpan<T, ()>, O), nom::Err<ParserError<C, LocatedSpan<T, ()>, Y>>>
    where
        E: Into<nom::Err<ParserError<C, LocatedSpan<T, ()>, Y>>>,
        Y: Copy,
    {
        Err(err.into())
    }

    fn enter(&self, _func: C, _span: LocatedSpan<T, ()>) {
        //
    }

    fn exit_ok(&self, _span: LocatedSpan<T, ()>, _parsed: LocatedSpan<T, ()>) {
        //
    }

    fn exit_err(&self, _span: LocatedSpan<T, ()>, _code: C, _err: &dyn Error) {
        //
    }
}
