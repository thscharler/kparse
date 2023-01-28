use crate::{
    Code, Context, ContextTrait, DynTracker, ParserError, ResultWithSpan, TrackParserError,
    WithCode, WithSpan,
};
use nom::error::ParseError;
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
use std::error::Error;
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};

//
// std::num::ParseIntError
//

// from the std::wilds
impl<C, I, Y> WithSpan<C, I, ParserError<C, I, Y>> for std::num::ParseIntError
where
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn with_span(self, code: C, span: I) -> nom::Err<ParserError<C, I, Y>> {
        nom::Err::Failure(ParserError::new(code, span))
    }
}

//
// std::num::ParseFloatError
//

// from the std::wilds
impl<C, I, Y> WithSpan<C, I, ParserError<C, I, Y>> for std::num::ParseFloatError
where
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn with_span(self, code: C, span: I) -> nom::Err<ParserError<C, I, Y>> {
        nom::Err::Failure(ParserError::new(code, span))
    }
}

// ***********************************************************************
// LAYER 1 - useful conversions
// ***********************************************************************

//
// ParserError
//
impl<C, I, Y> From<ParserError<C, I, Y>> for nom::Err<ParserError<C, I, Y>>
where
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn from(e: ParserError<C, I, Y>) -> Self {
        nom::Err::Error(e)
    }
}

impl<C, I, Y> WithCode<C, ParserError<C, I, Y>> for ParserError<C, I, Y>
where
    I: AsBytes + Copy,
    C: Code,
    Y: Copy,
{
    fn with_code(self, code: C) -> ParserError<C, I, Y> {
        ParserError::with_code(self, code)
    }
}

//
// nom::error::Error
//

// take everything from nom::error::Error
impl<C, I, Y> From<nom::error::Error<I>> for ParserError<C, I, Y>
where
    I: AsBytes + Copy,
    C: Code,
    Y: Copy,
{
    fn from(e: nom::error::Error<I>) -> Self {
        ParserError::from_error_kind(e.input, e.code)
    }
}

// ***********************************************************************
// LAYER 2 - wrapped in a nom::Err
// ***********************************************************************

//
// nom::Err::<E>
//

// for ease of use in case of a nom::Err wrapped something.
//
// 1. just to call with_code on an existing ParserError.
// 2. to convert whatever to a ParserError and give it a code.
impl<C, I, Y, E> WithCode<C, nom::Err<ParserError<C, I, Y>>> for nom::Err<E>
where
    E: Into<ParserError<C, I, Y>>,
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn with_code(self, code: C) -> nom::Err<ParserError<C, I, Y>> {
        match self {
            nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                let p_err: ParserError<C, I, Y> = e.into();
                let p_err = p_err.with_code(code);
                nom::Err::Error(p_err)
            }
        }
    }
}

// ***********************************************************************
// LAYER 3 - wrapped in a Result
// ***********************************************************************

//
// Result
//

// Any result that wraps an error type that can be converted via with_span is fine.
impl<C, I, Y, O, E> ResultWithSpan<C, I, Result<O, nom::Err<ParserError<C, I, Y>>>> for Result<O, E>
where
    E: WithSpan<C, I, ParserError<C, I, Y>>,
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn with_span(self, code: C, span: I) -> Result<O, nom::Err<ParserError<C, I, Y>>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.with_span(code, span)),
        }
    }
}

// everything needs a new code sometimes ... continued ...
//
// 1. this is a ParserResult with a nom::Err with a ParserError.
// 2. this is a Result with a whatever which has a WithCode<ParserError>
impl<C, I, Y, O, E> WithCode<C, Result<(I, O), nom::Err<ParserError<C, I, Y>>>>
    for Result<(I, O), E>
where
    E: WithCode<C, nom::Err<ParserError<C, I, Y>>>,
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn with_code(self, code: C) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let p_err: nom::Err<ParserError<C, I, Y>> = e.with_code(code);
                Err(p_err)
            }
        }
    }
}

// ***********************************************************************
// LAYER 4 - Convert & Track
// ***********************************************************************

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
