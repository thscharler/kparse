use crate::{Code, ParserError, ParserResult, Span, WithCode, WithSpan};
use nom;
use nom::error::ParseError;
use std;

//
// std::num::ParseIntError
//

// from the std::wilds
impl<'s, C: Code, X: Copy> WithSpan<'s, C, ParserError<'s, C, X>> for std::num::ParseIntError {
    fn with_span(self, code: C, span: Span<'s, C>) -> ParserError<'s, C, X> {
        ParserError::new(code, span)
    }
}

//
// std::num::ParseFloatError
//

// from the std::wilds
impl<'s, C: Code, X: Copy> WithSpan<'s, C, ParserError<'s, C, X>> for std::num::ParseFloatError {
    fn with_span(self, code: C, span: Span<'s, C>) -> ParserError<'s, C, X> {
        ParserError::new(code, span)
    }
}

//
// nom::error::Error
//

// a fresh simple nom::error::Error
//
// we know how to handle these
impl<'s, C: Code, X: Copy> From<nom::error::Error<Span<'s, C>>> for ParserError<'s, C, X> {
    fn from(e: nom::error::Error<Span<'s, C>>) -> Self {
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
impl<'s, C: Code, X: Copy, E> WithCode<'s, C, nom::Err<ParserError<'s, C, X>>> for nom::Err<E>
where
    E: Into<ParserError<'s, C, X>>,
{
    fn with_code(self, code: C) -> nom::Err<ParserError<'s, C, X>> {
        match self {
            nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            nom::Err::Error(e) => {
                let p_err: ParserError<'s, C, X> = e.into();
                let p_err = p_err.with_code(code);
                nom::Err::Error(p_err)
            }
            nom::Err::Failure(e) => {
                let p_err: ParserError<'s, C, X> = e.into();
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

// coming from elsewhere entering our domain ...
//
// 1. be well behaved and let us give you a span and a code
// 2. and wrap you up in a warm ParserError
impl<'s, C: Code, X: Copy, O, E> WithSpan<'s, C, ParserResult<'s, C, X, O>>
    for Result<(Span<'s, C>, O), E>
where
    E: WithSpan<'s, C, nom::Err<ParserError<'s, C, X>>>,
{
    fn with_span(self, code: C, span: Span<'s, C>) -> ParserResult<'s, C, X, O> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let p_err: nom::Err<ParserError<'s, C, X>> = e.with_span(code, span);
                Err(p_err)
            }
        }
    }
}

// everything needs a new code sometimes ... continued ...
//
// 1. this is a ParserResult with a nom::Err with a ParserError.
// 2. this is a Result with a whatever which has a WithCode<ParserError>
impl<'s, C: Code, X: Copy, O, E> WithCode<'s, C, ParserResult<'s, C, X, O>>
    for Result<(Span<'s, C>, O), E>
where
    E: WithCode<'s, C, nom::Err<ParserError<'s, C, X>>>,
{
    fn with_code(self, code: C) -> ParserResult<'s, C, X, O> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let p_err: nom::Err<ParserError<'s, C, X>> = e.with_code(code);
                Err(p_err)
            }
        }
    }
}
