use crate::{
    Code, DynTracker, ParserError, ResultWithSpan, TrackParserError, WithCode, WithSpan, C3, CCC,
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
    fn exit_ok(span: &'s str, parsed: &'s str) {
        <C3 as CCC<C, &str>>::exit_ok(&C3, span, parsed);
    }

    fn exit_err(span: &'s str, code: C, err: &dyn Error) {
        <C3 as CCC<C, &str>>::exit_err(&C3, span, code, err);
    }
}

impl<'s, C, Y, O, E> TrackParserError<'s, C, &'s [u8], Y, O, E>
    for Result<(&'s [u8], O), nom::Err<E>>
where
    E: Into<ParserError<C, &'s [u8], Y>>,
    C: Code,
    Y: Copy,
{
    fn exit_ok(span: &'s [u8], parsed: &'s [u8]) {
        <C3 as CCC<C, &[u8]>>::exit_ok(&C3, span, parsed);
    }

    fn exit_err(span: &'s [u8], code: C, err: &dyn Error) {
        <C3 as CCC<C, &[u8]>>::exit_err(&C3, span, code, err);
    }
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
    fn exit_ok(span: LocatedSpan<T, ()>, parsed: LocatedSpan<T, ()>) {
        <C3 as CCC<C, LocatedSpan<T, ()>>>::exit_ok(&C3, span, parsed);
    }

    fn exit_err(span: LocatedSpan<T, ()>, code: C, err: &dyn Error) {
        <C3 as CCC<C, LocatedSpan<T, ()>>>::exit_err(&C3, span, code, err);
    }
}

impl<'s, C, T, Y, O, E> TrackParserError<'s, C, LocatedSpan<T, DynTracker<'s, T, C>>, Y, O, E>
    for Result<(LocatedSpan<T, DynTracker<'s, T, C>>, O), nom::Err<E>>
where
    E: Into<ParserError<C, LocatedSpan<T, DynTracker<'s, T, C>>, Y>>,
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
        span: LocatedSpan<T, DynTracker<'s, T, C>>,
        parsed: LocatedSpan<T, DynTracker<'s, T, C>>,
    ) {
        <C3 as CCC<C, LocatedSpan<T, DynTracker<'s, T, C>>>>::exit_ok(&C3, span, parsed);
    }

    fn exit_err(span: LocatedSpan<T, DynTracker<'s, T, C>>, code: C, err: &dyn Error) {
        <C3 as CCC<C, LocatedSpan<T, DynTracker<'s, T, C>>>>::exit_err(&C3, span, code, err);
    }
}
