//!
//! Provides some extra parser combinators.
//!

use crate::{Code, WithCode, WithSpan};
use nom::Parser;

/// Takes a parser and converts the error via the WithCode trait.
pub fn error_code<PA, C, I, O, E0, E1>(
    mut parser: PA,
    code: C,
) -> impl FnMut(I) -> Result<(I, O), nom::Err<E1>>
where
    C: Code,
    PA: Parser<I, O, E0>,
    E0: WithCode<C, E1>,
{
    move |i| -> Result<(I, O), nom::Err<E1>> {
        match parser.parse(i) {
            Ok((r, v)) => Ok((r, v)),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_code(code))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Error(e.with_code(code))),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}

/// Takes a parser and a transformation of the parser result.
/// Maps any error to the given error code.
///
/// ```rust ignore
/// use nom::combinator::consumed;
/// use nom::Parser;
/// use kparse::{TrackParserError, transform};
///
/// let (rest, (tok, val)) =
///         consumed(transform(nom_parse_c, |v| (*v).parse::<u32>(), ICInteger))(rest).track()?;
/// ```
pub fn transform<PA, TRFn, C, I, O1, O2, E0, E1, E2>(
    mut parser: PA,
    transform: TRFn,
    code: C,
) -> impl FnMut(I) -> Result<(I, O2), nom::Err<E2>>
where
    C: Code,
    O1: Copy,
    PA: Parser<I, O1, E0>,
    TRFn: Fn(O1) -> Result<O2, E1>,
    E0: WithCode<C, E2>,
    E1: WithSpan<C, O1, E2>,
{
    move |i| -> Result<(I, O2), nom::Err<E2>> {
        let r = parser.parse(i);
        match r {
            Ok((rest, token)) => {
                let o = transform(token);
                match o {
                    Ok(o) => Ok((rest, o)),
                    Err(e) => Err(e.with_span(code, token)),
                }
            }
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_code(code))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.with_code(code))),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}

/// Runs a condition on the input and only executes the parser on succes.
pub fn conditional<I, O, E, CFn, PFn>(
    cond_fn: CFn,
    mut parse_fn: PFn,
) -> impl FnMut(I) -> Result<(I, Option<O>), nom::Err<E>>
where
    I: Copy,
    CFn: Fn(I) -> bool,
    PFn: Parser<I, O, E>,
{
    move |i| -> Result<(I, Option<O>), nom::Err<E>> {
        if cond_fn(i) {
            match parse_fn.parse(i) {
                Ok((r, v)) => Ok((r, Some(v))),
                Err(e) => Err(e),
            }
        } else {
            Ok((i, None))
        }
    }
}
