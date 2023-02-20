#![allow(dead_code)]

use kparse::combinators::{map_res, track, with_code};
use kparse::examples::{
    ExABNum, ExABstar, ExAoptB, ExAorB, ExAstarB, ExAthenB, ExNumber, ExParserError,
    ExParserResult, ExSpan, ExTagA, ExTagB, ExTokenizerError, ExTokenizerResult,
};
use kparse::prelude::*;
#[cfg(debug_assertions)]
use kparse::tracker::{StdTracker, TrackSpan};
use kparse::Context;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::{consumed, opt};
use nom::multi::many0;
use nom::sequence::{terminated, tuple};
use nom::{AsChar, InputTakeAtPosition, Parser};
use std::env;

#[derive(Debug)]
struct AstA<'s> {
    pub span: ExSpan<'s>,
}

#[derive(Debug)]
struct AstB<'s> {
    pub span: ExSpan<'s>,
}

#[derive(Debug)]
struct AstAthenB<'s> {
    pub a: AstA<'s>,
    pub b: AstB<'s>,
}

#[derive(Debug)]
struct AstAoptB<'s> {
    pub a: Option<AstA<'s>>,
    pub b: AstB<'s>,
}

#[derive(Debug)]
struct AstAstarB<'s> {
    pub a: Vec<AstA<'s>>,
    pub b: AstB<'s>,
}

#[derive(Debug)]
struct AstABstar<'s> {
    pub a: Vec<AstA<'s>>,
    pub b: Vec<AstB<'s>>,
}

#[derive(Debug)]
struct AstAorB<'s> {
    pub a: Option<AstA<'s>>,
    pub b: Option<AstB<'s>>,
}

#[derive(Debug)]
struct AstABNum<'s> {
    pub a: Option<AstA<'s>>,
    pub b: Option<AstB<'s>>,
    pub num: AstNumber<'s>,
}

#[derive(Debug)]
struct AstNumber<'s> {
    pub number: u32,
    pub span: ExSpan<'s>,
}

fn nom_parse_a(i: ExSpan<'_>) -> ExTokenizerResult<'_, ExSpan<'_>> {
    with_code(tag("a"), ExTagA)(i)
}

fn nom_parse_b(i: ExSpan<'_>) -> ExTokenizerResult<'_, ExSpan<'_>> {
    with_code(tag("b"), ExTagB)(i)
}

fn nom_digits(i: ExSpan<'_>) -> ExTokenizerResult<'_, ExSpan<'_>> {
    digit1(i)
}

fn nom_ws(i: ExSpan<'_>) -> ExTokenizerResult<'_, ExSpan<'_>> {
    i.split_at_position_complete(|item| {
        let c = item.as_char();
        !(c == ' ' || c == '\t')
    })
}

fn nom_number(i: ExSpan<'_>) -> ExParserResult<'_, (ExSpan<'_>, u32)> {
    Parser::into(consumed(map_res(
        terminated(digit1, nom_ws),
        |v| match (*v).parse::<u32>() {
            Ok(vv) => Ok(vv),
            Err(_) => Err(ExTokenizerError::new(ExNumber, v).failure()),
        },
    )))
    .parse(i)
}

fn token_number(i: ExSpan<'_>) -> ExParserResult<'_, AstNumber<'_>> {
    match nom_number(i) {
        Ok((rest, (tok, val))) => Ok((
            rest,
            AstNumber {
                number: val,
                span: tok,
            },
        )),
        Err(e) => Err(e.with_code(ExNumber)),
    }
}

fn parse_a(input: ExSpan<'_>) -> ExParserResult<'_, AstA> {
    Context.enter(ExTagA, input);
    let (rest, tok) = Parser::into(nom_parse_a).parse(input).track()?;
    Context.ok(rest, tok, AstA { span: tok })
}

fn parse_b(input: ExSpan<'_>) -> ExParserResult<'_, AstB> {
    Parser::into(track(
        ExTagB,
        nom_parse_b.and_then(|span| Ok((span, AstB { span }))),
    ))
    .parse(input)
}

// := a b
fn parse_ab(input: ExSpan<'_>) -> ExParserResult<'_, AstAthenB> {
    Context.enter(ExAthenB, input);

    let rest = input;

    let (rest, a) = parse_a(rest).track()?;
    let (rest, b) = parse_b(rest).track()?;

    let span = input.span_union(&a.span, &b.span);

    Context.ok(rest, span, AstAthenB { a, b })
}

// := a b
fn parse_ab_v2(input: ExSpan<'_>) -> ExParserResult<'_, AstAthenB> {
    Context.enter(ExAthenB, input);
    let (rest, (span, (a, b))) = consumed(tuple((parse_a, parse_b)))(input).track()?;
    Context.ok(rest, span, AstAthenB { a, b })
}

// := a? b
fn parse_a_opt_b(input: ExSpan<'_>) -> ExParserResult<'_, AstAoptB> {
    Context.enter(ExAoptB, input);
    let (rest, (span, val)) = consumed(tuple((opt(parse_a), parse_b)))(input).track()?;
    Context.ok(rest, span, AstAoptB { a: val.0, b: val.1 })
}

// := a* b
fn parse_a_star_b(input: ExSpan<'_>) -> ExParserResult<'_, AstAstarB> {
    Context.enter(ExAstarB, input);
    let (rest, (span, val)) = consumed(tuple((many0(parse_a), parse_b)))(input).track()?;
    Context.ok(rest, span, AstAstarB { a: val.0, b: val.1 })
}

// := ( a | b )*
fn parse_a_b_star(input: ExSpan<'_>) -> ExParserResult<'_, AstABstar> {
    Context.enter(ExABstar, input);

    let mut loop_rest = input;
    let mut res = AstABstar {
        a: vec![],
        b: vec![],
    };
    let mut err = None;

    loop {
        let rest2 = loop_rest;

        let rest2 = match parse_a(rest2) {
            Ok((rest3, a)) => {
                res.a.push(a);
                rest3
            }
            Err(e) => match parse_b(rest2) {
                Ok((rest3, b)) => {
                    res.b.push(b);
                    rest3
                }
                Err(e2) => {
                    err.append(e)?;
                    err.append(e2)?;
                    rest2
                }
            },
        };

        if let Some(err) = err {
            return Context.err(err);
        }
        if rest2.is_empty() {
            break;
        }

        loop_rest = rest2;
    }

    Context.ok(loop_rest, input, res)
}

fn parse_a_or_b(input: ExSpan<'_>) -> ExParserResult<'_, AstAorB> {
    Context.enter(ExAorB, input);

    let rest = input;

    let (rest, a) = opt(parse_a)(rest).track()?;
    let (rest, b) = if a.is_none() {
        opt(parse_b)(input).track()?
    } else {
        (rest, None)
    };

    let span = if let Some(a) = &a {
        a.span
    } else if let Some(b) = &b {
        b.span
    } else {
        return Context.err(ExParserError::new(ExAorB, input));
    };

    Context.ok(rest, span, AstAorB { a, b })
}

fn parse_a_b_num(input: ExSpan<'_>) -> ExParserResult<'_, AstABNum> {
    Context.enter(ExABNum, input);

    let rest = input;

    let (rest, a) = opt(parse_a)(rest).track()?;
    let (rest, b) = opt(parse_b)(rest).track()?;
    let (rest, num) = token_number(rest).track()?;

    let span = if let Some(a) = &a {
        input.span_union(&a.span, &num.span)
    } else if let Some(b) = &b {
        input.span_union(&b.span, &num.span)
    } else {
        num.span
    };

    Context.ok(rest, span, AstABNum { a, b, num })
}

fn main() {
    #[cfg(debug_assertions)]
    for txt in env::args() {
        let trk = StdTracker::new();
        let span = trk.span(txt.as_str());

        match parse_a_b_star(span) {
            Ok((_rest, val)) => {
                dbg!(val);
            }
            Err(e) => {
                println!("{:?}", trk.results());
                println!("{:?}", e);
            }
        }
    }
    #[cfg(not(debug_assertions))]
    for txt in env::args() {
        let span = txt.as_str();

        match parse_a_b_star(span) {
            Ok((_rest, val)) => {
                dbg!(val);
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use kparse::test::{str_parse, Timing};

    #[test]
    fn test_1() {
        str_parse(&mut None, "", parse_ab).err_any().q(Timing(1));
        str_parse(&mut None, "ab", parse_ab).ok_any().q(Timing(1));
        str_parse(&mut None, "aba", parse_ab).rest("a").q(Timing(1));
    }

    #[test]
    fn test_2() {
        str_parse(&mut None, "ab", parse_a_or_b)
            .ok_any()
            .q(Timing(1));
        str_parse(&mut None, "a", parse_a_or_b)
            .ok_any()
            .q(Timing(1));
        str_parse(&mut None, "b", parse_a_or_b)
            .ok_any()
            .q(Timing(1));

        str_parse(&mut None, "", parse_a_opt_b)
            .err_any()
            .q(Timing(1));
        str_parse(&mut None, "b", parse_a_opt_b)
            .ok_any()
            .rest("")
            .q(Timing(1));
        str_parse(&mut None, "ab", parse_a_opt_b)
            .ok_any()
            .rest("")
            .q(Timing(1));
        str_parse(&mut None, "bb", parse_a_opt_b)
            .ok_any()
            .rest("b")
            .q(Timing(1));
        str_parse(&mut None, "aab", parse_a_opt_b)
            .err_any()
            .q(Timing(1));
        str_parse(&mut None, "aab", parse_a_opt_b)
            .err_any()
            .q(Timing(1));

        str_parse(&mut None, "aab", parse_a_star_b)
            .ok_any()
            .q(Timing(1));

        str_parse(&mut None, "aab", parse_a_b_star)
            .ok_any()
            .q(Timing(1));
        str_parse(&mut None, "aabc", parse_a_b_star)
            .err_any()
            .q(Timing(1));
    }
}
