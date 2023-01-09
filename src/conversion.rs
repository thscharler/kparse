use crate::{Code, ErrorSpan, Hints, ParserError, ParserResult, Span, WithCode, WithSpan};
use nom;
use nom::error::ParseError;
use std;

//
// std::num::ParseIntError
//

impl<'s, C: Code, X: Copy> WithSpan<'s, C, ParserError<'s, C, X>> for std::num::ParseIntError {
    fn with_span(self, code: C, span: Span<'s>) -> ParserError<'s, C, X> {
        ParserError::new(code, span)
    }
}

//
// std::num::ParseFloatError
//

impl<'s, C: Code, X: Copy> WithSpan<'s, C, ParserError<'s, C, X>> for std::num::ParseFloatError {
    fn with_span(self, code: C, span: Span<'s>) -> ParserError<'s, C, X> {
        ParserError::new(code, span)
    }
}

//
// nom::Needed
//

impl<'s, C: Code, X: Copy> From<nom::Needed> for ParserError<'s, C, X> {
    fn from(v: nom::Needed) -> Self {
        let mut p = ParserError {
            code: C::NOM_INCOMPLETE,
            span: ErrorSpan::Unknown,
            hints: Vec::new(),
        };
        match v {
            nom::Needed::Unknown => {}
            nom::Needed::Size(s) => p.hints.push(Hints::Needed(s)),
        };
        p
    }
}

//
// nom::error::Error
//

impl<'s, C: Code, X: Copy> From<nom::error::Error<Span<'s>>> for ParserError<'s, C, X> {
    fn from(e: nom::error::Error<Span<'s>>) -> Self {
        ParserError::from_error_kind(e.input, e.code)
    }
}

//
// Result
//

impl<'s, C: Code, X: Copy, O, E> WithSpan<'s, C, ParserResult<'s, C, X, O>> for Result<O, E>
where
    E: WithSpan<'s, C, ParserError<'s, C, X>>,
{
    fn with_span(self, code: C, span: Span<'s>) -> ParserResult<'s, C, X, O> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.with_span(code, span)),
        }
    }
}

impl<'s, C: Code, X: Copy, O, E> WithCode<'s, C, ParserResult<'s, C, X, O>> for Result<O, E>
where
    E: WithCode<'s, C, ParserError<'s, C, X>>,
{
    fn with_code(self, code: C) -> ParserResult<'s, C, X, O> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.with_code(code)),
        }
    }
}

//
// nom::Err::<E>
//

impl<'s, C: Code, X: Copy, E> From<nom::Err<E>> for ParserError<'s, C, X>
where
    E: Into<ParserError<'s, C, X>>,
{
    fn from(e: nom::Err<E>) -> Self {
        match e {
            nom::Err::Incomplete(e) => e.into(),
            nom::Err::Error(e) => e.into(),
            nom::Err::Failure(e) => {
                let mut p = e.into();
                if p.code == C::NOM_ERROR {
                    p.code = C::NOM_FAILURE;
                }
                p
            }
        }
    }
}

impl<'s, C: Code, X: Copy, E> WithCode<'s, C, ParserError<'s, C, X>> for nom::Err<E>
where
    E: Into<ParserError<'s, C, X>>,
{
    fn with_code(self, code: C) -> ParserError<'s, C, X> {
        let pe: ParserError<'s, C, X> = match self {
            nom::Err::Incomplete(e) => e.into(),
            nom::Err::Error(e) => e.into(),
            nom::Err::Failure(e) => e.into(),
        };
        pe.into_code(code)
    }
}

//
// ParserError
//

impl<'s, C: Code, X: Copy> WithCode<'s, C, ParserError<'s, C, X>> for ParserError<'s, C, X> {
    fn with_code(self, code: C) -> ParserError<'s, C, X> {
        self.into_code(code)
    }
}
