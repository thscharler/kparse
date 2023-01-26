use crate::{
    Code, Context, ParserError, ParserResult, ResultWithSpan, Span, TrackParserError, WithCode,
    WithSpan,
};
use nom::error::ParseError;
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use std::fmt::{Debug, Display};
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

impl<'s, T, C, Y, O, E> TrackParserError<'s, Span<'s, T, C>, C, Y>
    for Result<(Span<'s, T, C>, O), nom::Err<E>>
where
    E: Into<ParserError<C, Span<'s, T, C>, Y>>,
    C: Code,
    Y: Copy,
    T: Copy + Display + Debug,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    type Result = Result<(Span<'s, T, C>, O), nom::Err<ParserError<C, Span<'s, T, C>, Y>>>;

    fn track(self) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<C, Span<'s, T, C>, Y> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<C, Span<'s, T, C>, Y> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    fn track_as(self, code: C) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<C, Span<'s, T, C>, Y> = e.into();
                let p_err = p_err.with_code(code);
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<C, Span<'s, T, C>, Y> = e.into();
                let p_err = p_err.with_code(code);
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    fn track_ok(self, parsed: Span<'s, T, C>) -> Self::Result {
        match self {
            Ok((span, v)) => {
                Context.exit_ok(&parsed, &span);
                Ok((span, v))
            }
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<C, Span<'s, T, C>, Y> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<C, Span<'s, T, C>, Y> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }
}
