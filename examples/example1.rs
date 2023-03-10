#![allow(dead_code)]

use crate::ExCode::*;
use kparse::combinators::track;
use kparse::prelude::*;
use kparse::token_error::TokenizerError;
use kparse::{ParserError, ParserResult, TokenizerResult};
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::consumed;
use nom::multi::many0;
use nom::sequence::{terminated, tuple};
use nom::{AsChar, InputTakeAtPosition, Parser};
use std::env;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExCode {
    ExNomError,

    ExTagA,
    ExTagB,
    ExNumber,

    ExAthenB,
    ExAoptB,
    ExAstarB,
    ExABstar,
    ExAorB,
    ExABNum,
}

impl Display for ExCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ExNomError => "nom",
                ExTagA => "a",
                ExTagB => "b",
                ExNumber => "number",
                ExAthenB => "A B",
                ExAoptB => "A? B",
                ExAstarB => "A* B",
                ExABstar => "(A | B)*",
                ExAorB => "A | B",
                ExABNum => "A B Number",
            }
        )
    }
}

impl Code for ExCode {
    const NOM_ERROR: Self = Self::ExNomError;
}

define_span!(ExSpan = ExCode, str);
pub type ExParserResult<'s, O> = ParserResult<ExCode, ExSpan<'s>, O>;
pub type ExTokenizerResult<'s, O> = TokenizerResult<ExCode, ExSpan<'s>, O>;
pub type ExParserError<'s> = ParserError<ExCode, ExSpan<'s>>;
pub type ExTokenizerError<'s> = TokenizerError<ExCode, ExSpan<'s>>;

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
    tag("a").with_code(ExTagA).parse(i)
}

fn nom_parse_b(i: ExSpan<'_>) -> ExTokenizerResult<'_, ExSpan<'_>> {
    tag("b").with_code(ExTagB).parse(i)
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
    consumed(terminated(digit1, nom_ws).parse_from_str::<_, u32>(ExNumber)) //
        .err_into()
        .parse(i)
}

fn token_number(i: ExSpan<'_>) -> ExParserResult<'_, AstNumber<'_>> {
    nom_number
        .map(|(span, number)| AstNumber { number, span })
        .parse(i)
}

fn parse_a(input: ExSpan<'_>) -> ExParserResult<'_, AstA> {
    Track.enter(ExTagA, input);
    let (rest, tok) = nom_parse_a.parse(input).err_into().track()?;
    Track.ok(rest, tok, AstA { span: tok })
}

fn parse_b(input: ExSpan<'_>) -> ExParserResult<'_, AstB> {
    track(
        ExTagB, //
        nom_parse_b.map(|span| AstB { span }),
    )
    .err_into()
    .parse(input)
}

// := a b
fn parse_ab(input: ExSpan<'_>) -> ExParserResult<'_, AstAthenB> {
    Track.enter(ExAthenB, input);

    let rest = input;

    let (rest, a) = parse_a(rest).track()?;
    let (rest, b) = parse_b(rest).track()?;

    let span = input.span_union(&a.span, &b.span);

    Track.ok(rest, span, AstAthenB { a, b })
}

// := a b
fn parse_ab_v2(input: ExSpan<'_>) -> ExParserResult<'_, AstAthenB> {
    Track.enter(ExAthenB, input);
    let (rest, (span, (a, b))) = consumed(tuple((parse_a, parse_b)))(input).track()?;
    Track.ok(rest, span, AstAthenB { a, b })
}

// := a? b
fn parse_a_opt_b(input: ExSpan<'_>) -> ExParserResult<'_, AstAoptB> {
    track(
        ExAoptB,
        tuple((parse_a.opt(), parse_b)) //
            .map(|(a, b)| AstAoptB { a, b }),
    )
    .err_into()
    .parse(input)
}

// := a* b
fn parse_a_star_b(input: ExSpan<'_>) -> ExParserResult<'_, AstAstarB> {
    track(
        ExAstarB,
        tuple((many0(parse_a), parse_b)) //
            .map(|(a, b)| AstAstarB { a, b }),
    )
    .err_into()
    .parse(input)
}

// := ( a | b )*
fn parse_a_b_star(input: ExSpan<'_>) -> ExParserResult<'_, AstABstar> {
    Track.enter(ExABstar, input);

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
            return Track.err(err);
        }
        if rest2.is_empty() {
            break;
        }

        loop_rest = rest2;
    }

    Track.ok(loop_rest, input, res)
}

fn parse_a_or_b(input: ExSpan<'_>) -> ExParserResult<'_, AstAorB> {
    track(
        ExAorB,
        parse_a
            .or_else(parse_b) //
            .map(|(a, b)| AstAorB { a, b }),
    )
    .err_into()
    .parse(input)
}

fn parse_a_b_num(input: ExSpan<'_>) -> ExParserResult<'_, AstABNum> {
    track(
        ExABNum,
        tuple((
            //
            parse_a.opt(),
            parse_b.opt(),
            token_number,
        ))
        .map(|(a, b, num)| AstABNum { a, b, num }),
    )
    .err_into()
    .parse(input)
}

fn main() {
    for txt in env::args() {
        let trk = Track.new_tracker::<ExCode, _>();
        let span = trk.span(txt.as_str());

        match parse_a_b_star(span) {
            Ok((_rest, val)) => {
                dbg!(val);
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                println!("{:?}", trk.results());
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
