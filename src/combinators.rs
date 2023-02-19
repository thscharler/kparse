//!
//! Provides some extra parser combinators.
//!

use crate::tracker::Tracking;
use crate::{Code, ParseErrorExt};
use nom::{AsBytes, InputIter, InputLength, InputTake, Parser};
use std::fmt::Debug;

/// Tracked execution of a parser.
#[inline(always)]
pub fn track<PA, C, I, O, E>(
    func: C,
    mut parser: PA,
) -> impl FnMut(I) -> Result<(I, O), nom::Err<E>>
where
    PA: Parser<I, O, E>,
    C: Code,
    I: Copy + Debug + Tracking<C>,
    I: InputTake + InputLength + InputIter + AsBytes,
    nom::Err<E>: ParseErrorExt<C, I>,
{
    move |input| -> Result<(I, O), nom::Err<E>> {
        input.track_enter(func);
        match parser.parse(input) {
            Ok((rest, token)) => {
                rest.track_ok(input);
                rest.track_exit();
                Ok((rest, token))
            }
            Err(err) => match err.parts() {
                None => Err(err),
                Some((code, span, e)) => {
                    span.track_err(code, e);
                    span.track_exit();
                    Err(err)
                }
            },
        }
    }
}

/// Takes a parser and converts the error via the WithCode trait.
#[inline(always)]
pub fn error_code<PA, C, I, O, E>(
    mut parser: PA,
    code: C,
) -> impl FnMut(I) -> Result<(I, O), nom::Err<E>>
where
    PA: Parser<I, O, E>,
    E: ParseErrorExt<C, I>,
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
#[inline(always)]
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
#[inline(always)]
pub fn when<CFn, PFn, C, I, O, E>(
    cond_fn: CFn,
    mut parse_fn: PFn,
) -> impl FnMut(I) -> Result<(I, Option<O>), nom::Err<E>>
where
    CFn: Fn(I) -> bool,
    PFn: Parser<I, O, E>,
    C: Code,
    I: AsBytes + Copy,
{
    move |i| -> Result<(I, Option<O>), nom::Err<E>> {
        if cond_fn(i) {
            parse_fn.parse(i).map(|(r, v)| (r, Some(v)))
        } else {
            Ok((i, None))
        }
    }
}
