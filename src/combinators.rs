//!
//! Provides some extra parser combinators.
//!

use crate::tracker::Tracking;
use crate::{Code, KParseError, MapRes};
use nom::{AsBytes, InputIter, InputLength, InputTake, Parser};
use std::fmt::Debug;

/// Tracked execution of a parser.
///
/// ```rust
/// use nom::bytes::complete::tag;
/// use kparse::combinators::{err_into, with_code, track, map_res};
/// use kparse::examples::{ExParserResult, ExSpan, ExTagB, ExTokenizerResult};
/// use kparse::KParseError;
///
/// fn parse_b(input: ExSpan<'_>) -> ExParserResult<'_, AstB> {
///     err_into(track(ExTagB,
///         map_res(nom_parse_b, |span| Ok(AstB { span }))
///     ))(input)
/// }
///
/// fn nom_parse_b(i: ExSpan<'_>) -> ExTokenizerResult<'_, ExSpan<'_>> {
///     with_code(tag("b"), ExTagB)(i)
/// }
///
/// struct AstB<'s> {
///     pub span: ExSpan<'s>,
/// }
///
/// ```
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
    nom::Err<E>: KParseError<C, I>,
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

///
///
#[inline(always)]
pub fn err_into<PA, I, O, E1, E2>(mut parser: PA) -> impl FnMut(I) -> Result<(I, O), nom::Err<E2>>
where
    PA: Parser<I, O, E1>,
    E2: From<E1>,
{
    move |i| -> Result<(I, O), nom::Err<E2>> {
        match parser.parse(i) {
            Ok((r, o)) => Ok((r, o)),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.into())),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.into())),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}

/// Takes a parser and converts the error.
///
/// ```rust
/// use nom::bytes::complete::tag;
/// use kparse::combinators::with_code;
/// use kparse::examples::{ExSpan, ExTagB, ExTokenizerResult};
///
/// fn nom_parse_b(i: ExSpan<'_>) -> ExTokenizerResult<'_, ExSpan<'_>> {
///     with_code(tag("b"), ExTagB)(i)
/// }
/// ```
#[inline(always)]
pub fn with_code<PA, C, I, O, E>(
    mut parser: PA,
    code: C,
) -> impl FnMut(I) -> Result<(I, O), nom::Err<E>>
where
    PA: Parser<I, O, E>,
    E: KParseError<C, I>,
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
/// ```rust
/// use nom::character::complete::digit1;
/// use nom::combinator::consumed;
/// use nom::{AsChar, InputTakeAtPosition, Parser};
/// use nom::sequence::terminated;
/// use kparse::combinators::map_res;
/// use kparse::examples::ExCode::ExNumber;
/// use kparse::examples::{ExParserError, ExSpan, ExTokenizerError, ExTokenizerResult};
///
/// fn nom_number(i: ExSpan<'_>) -> ExTokenizerResult<'_, (ExSpan<'_>, u32)> {
///     consumed(map_res(terminated(digit1, nom_ws), |v| {
///         match (*v).parse::<u32>() {
///             Ok(vv) => Ok(vv),
///             Err(_) => Err(ExTokenizerError::new(ExNumber, v).failure()),
///         }
///     }))(i)
/// }
///
/// fn nom_ws(i: ExSpan<'_>) -> ExTokenizerResult<'_, ExSpan<'_>> {
///     i.split_at_position_complete(|item| {
///         let c = item.as_char();
///         !(c == ' ' || c == '\t')
///     })
/// }
/// ```
#[inline(always)]
pub fn map_res<PA, TR, I, O1, O2, E>(parser: PA, transform: TR) -> MapRes<PA, O1, TR, O2>
where
    PA: Parser<I, O1, E>,
    TR: Fn(O1) -> Result<O2, nom::Err<E>>,
    O1: Copy,
    I: AsBytes + Copy,
{
    MapRes {
        parser,
        map: transform,
        _phantom: Default::default(),
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
