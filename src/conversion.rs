use crate::{Code, ParserError, ParserResult, Span, WithCode, WithSpan};
use nom::error::ParseError;
use nom::AsBytes;

//
// std::num::ParseIntError
//

// todo: this can't be replicated in user code. switch to result and watch go bang.
// from the std::wilds
impl<'s, T: AsBytes + Copy, C: Code, Y: Copy> WithSpan<'s, T, C, nom::Err<ParserError<'s, T, C, Y>>>
    for std::num::ParseIntError
{
    fn with_span(self, code: C, span: Span<'s, T, C>) -> nom::Err<ParserError<'s, T, C, Y>> {
        nom::Err::Failure(ParserError::new(code, span))
    }
}

//
// std::num::ParseFloatError
//

// from the std::wilds
impl<'s, T: AsBytes + Copy, C: Code, Y: Copy> WithSpan<'s, T, C, nom::Err<ParserError<'s, T, C, Y>>>
    for std::num::ParseFloatError
{
    fn with_span(self, code: C, span: Span<'s, T, C>) -> nom::Err<ParserError<'s, T, C, Y>> {
        nom::Err::Failure(ParserError::new(code, span))
    }
}

// ***********************************************************************
// LAYER 1 - useful conversions
// ***********************************************************************

//
// ParserError
//
impl<'s, T, C: Code, Y: Copy> From<ParserError<'s, T, C, Y>>
    for nom::Err<ParserError<'s, T, C, Y>>
{
    fn from(e: ParserError<'s, T, C, Y>) -> Self {
        nom::Err::Error(e)
    }
}

impl<'s, T: AsBytes + Copy + 's, C: Code, Y: Copy> WithCode<C, ParserError<'s, T, C, Y>>
    for ParserError<'s, T, C, Y>
{
    fn with_code(self, code: C) -> ParserError<'s, T, C, Y> {
        ParserError::with_code(self, code)
    }
}

//
// nom::error::Error
//

// take everything from nom::error::Error
impl<'s, T: AsBytes + Copy + 's, C: Code, Y: Copy> From<nom::error::Error<Span<'s, T, C>>>
    for ParserError<'s, T, C, Y>
{
    fn from(e: nom::error::Error<Span<'s, T, C>>) -> Self {
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
impl<'s, T: AsBytes + Copy + 's, C: Code, Y: Copy, E>
    WithCode<C, nom::Err<ParserError<'s, T, C, Y>>> for nom::Err<E>
where
    E: Into<ParserError<'s, T, C, Y>>,
{
    fn with_code(self, code: C) -> nom::Err<ParserError<'s, T, C, Y>> {
        match self {
            nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            nom::Err::Error(e) => {
                let p_err: ParserError<'s, T, C, Y> = e.into();
                let p_err = p_err.with_code(code);
                nom::Err::Error(p_err)
            }
            nom::Err::Failure(e) => {
                let p_err: ParserError<'s, T, C, Y> = e.into();
                let p_err = p_err.with_code(code);
                nom::Err::Failure(p_err)
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
impl<'s, T, C: Code, Y: Copy, O, E>
    WithSpan<'s, T, C, Result<O, nom::Err<ParserError<'s, T, C, Y>>>> for Result<O, E>
where
    E: WithSpan<'s, T, C, nom::Err<ParserError<'s, T, C, Y>>>,
{
    fn with_span(
        self,
        code: C,
        span: Span<'s, T, C>,
    ) -> Result<O, nom::Err<ParserError<'s, T, C, Y>>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let p_err: nom::Err<ParserError<'s, T, C, Y>> = e.with_span(code, span);
                Err(p_err)
            }
        }
    }
}

// everything needs a new code sometimes ... continued ...
//
// 1. this is a ParserResult with a nom::Err with a ParserError.
// 2. this is a Result with a whatever which has a WithCode<ParserError>
impl<'s, T, C: Code, Y: Copy, O, E> WithCode<C, ParserResult<'s, O, T, C, Y>>
    for Result<(Span<'s, T, C>, O), E>
where
    E: WithCode<C, nom::Err<ParserError<'s, T, C, Y>>>,
{
    fn with_code(self, code: C) -> ParserResult<'s, O, T, C, Y> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let p_err: nom::Err<ParserError<'s, T, C, Y>> = e.with_code(code);
                Err(p_err)
            }
        }
    }
}
