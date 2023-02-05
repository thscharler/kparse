//!
//! Provides some extra parser combinators.
//!

use crate::tracker::FindTracker;
use crate::{Code, ParserError, WithSpan};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Parser, Slice};
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};

/// Tracked execution of a parser.
pub fn track<PA, C, I, O>(
    code: C,
    mut parser: PA,
) -> impl FnMut(I) -> Result<(I, O), nom::Err<ParserError<C, I>>>
where
    PA: Parser<I, O, ParserError<C, I>>,
    C: Code,
    I: Copy + Debug + FindTracker<C>,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    move |i| -> Result<(I, O), nom::Err<ParserError<C, I>>> {
        i.enter(code);
        match parser.parse(i) {
            Ok((r, v)) => r.ok(i, v),
            Err(e) => i.err(e),
        }
    }
}

/// Takes a parser and converts the error via the WithCode trait.
pub fn error_code<PA, C, I, O, Y>(
    mut parser: PA,
    code: C,
) -> impl FnMut(I) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>>
where
    PA: Parser<I, O, ParserError<C, I, Y>>,
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    move |i| -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
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
pub fn transform<PA, TRFn, C, I, PAO, TrO, TrE, Y>(
    mut parser: PA,
    transform: TRFn,
    code: C,
) -> impl FnMut(I) -> Result<(I, TrO), nom::Err<ParserError<C, I, Y>>>
where
    PA: Parser<I, PAO, ParserError<C, I, Y>>,
    TRFn: Fn(PAO) -> Result<TrO, TrE>,
    C: Code,
    I: AsBytes + Copy,
    PAO: Copy,
    TrE: WithSpan<C, PAO, ParserError<C, I, Y>>,
    Y: Copy,
{
    move |i| -> Result<(I, TrO), nom::Err<ParserError<C, I, Y>>> {
        parser.parse(i).and_then(|(rest, tok)| {
            transform(tok)
                .map(|v| (rest, v))
                .map_err(|e| e.with_span(code, tok))
        })
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
pub fn transform_p<PA, TRFn, C, I, O1, O2, Y>(
    mut parser: PA,
    transform: TRFn,
) -> impl FnMut(I) -> Result<(I, O2), nom::Err<ParserError<C, I, Y>>>
where
    PA: Parser<I, O1, ParserError<C, I, Y>>,
    TRFn: Fn(O1) -> Result<O2, nom::Err<ParserError<C, I, Y>>>,
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    move |i| -> Result<(I, O2), nom::Err<ParserError<C, I, Y>>> {
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
