//!
//! Provides some extra parser combinators.
//!

use crate::tracker::Tracking;
use crate::{Code, ParserError, WithCode};
use nom::{AsBytes, InputIter, InputLength, InputTake, Parser};
use std::fmt::Debug;

/// Tracked execution of a parser.
pub fn track<PA, C, I, O>(
    code: C,
    mut parser: PA,
) -> impl FnMut(I) -> Result<(I, O), nom::Err<ParserError<C, I>>>
where
    PA: Parser<I, O, ParserError<C, I>>,
    C: Code,
    I: Copy + Debug + Tracking<C>,
    I: InputTake + InputLength + InputIter + AsBytes,
{
    move |i| -> Result<(I, O), nom::Err<ParserError<C, I>>> {
        i.track_enter(code);
        match parser.parse(i) {
            Ok((r, v)) => r.ok(i, v),
            Err(nom::Err::Incomplete(e)) => i.err(C::NOM_ERROR, nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => i.err(e.code, nom::Err::Error(e)),
            Err(nom::Err::Failure(e)) => i.err(e.code, nom::Err::Failure(e)),
        }
    }
}

/// Takes a parser and converts the error via the WithCode trait.
pub fn error_code<PA, C, I, O, E>(
    mut parser: PA,
    code: C,
) -> impl FnMut(I) -> Result<(I, O), nom::Err<E>>
where
    PA: Parser<I, O, E>,
    E: WithCode<C, E>,
    C: Code,
    I: AsBytes + Copy,
{
    move |i| -> Result<(I, O), nom::Err<E>> {
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
pub fn transform<PA, TRFn, I, O1, O2, E>(
    mut parser: PA,
    transform: TRFn,
) -> impl FnMut(I) -> Result<(I, O2), nom::Err<E>>
where
    PA: Parser<I, O1, E>,
    TRFn: Fn(O1) -> Result<O2, nom::Err<E>>,
    I: AsBytes + Copy,
{
    move |i| -> Result<(I, O2), nom::Err<E>> {
        parser
            .parse(i)
            .and_then(|(rest, tok)| Ok((rest, transform(tok)?)))
    }
}

/// Runs a condition on the input and only executes the parser on success.
pub fn conditional<CFn, PFn, C, I, O, Y>(
    cond_fn: CFn,
    mut parse_fn: PFn,
) -> impl FnMut(I) -> Result<(I, Option<O>), nom::Err<ParserError<C, I, Y>>>
where
    CFn: Fn(I) -> bool,
    PFn: Parser<I, O, ParserError<C, I, Y>>,
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    move |i| -> Result<(I, Option<O>), nom::Err<ParserError<C, I, Y>>> {
        if cond_fn(i) {
            parse_fn.parse(i).map(|(r, v)| (r, Some(v)))
        } else {
            Ok((i, None))
        }
    }
}
