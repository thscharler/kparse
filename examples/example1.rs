#![allow(dead_code)]

use crate::ICode::*;
use kparse::prelude::*;
use kparse::spans::SpanExt;
use kparse::test::{track_parse, Trace};
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{char as nchar, digit1};
use nom::combinator::{consumed, opt, recognize};
use nom::sequence::{terminated, tuple};
use nom::{AsChar, InputTake, InputTakeAtPosition};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ICode {
    ICNomError,

    ICTerminalA,
    ICTerminalB,
    ICTerminalC,
    ICTerminalD,
    ICNonTerminal1,
    ICNonTerminal2,
    ICNonTerminal3,
    ICInteger,
    ICNummer,
}

impl Code for ICode {
    const NOM_ERROR: Self = ICNomError;
}

impl Display for ICode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ICNomError => "NomError",
            ICTerminalA => "TerminalA",
            ICInteger => "Int",
            ICTerminalB => "TerminalB",
            ICNonTerminal1 => "NonTerminal1",
            ICNonTerminal2 => "NonTerminal2",
            ICNonTerminal3 => "NonTerminal3",
            ICTerminalC => "TerminalC",
            ICTerminalD => "TerminalD",
            ICNummer => "Nummer",
        };
        write!(f, "{}", name)
    }
}

pub type Span<'s> = kparse::Span<'s, &'s str, ICode>;
pub type IParserResult<'s, O> = ParserResult<'s, O, &'s str, ICode, ()>;
pub type INomResult<'s> = ParserNomResult<'s, &'s str, ICode, ()>;
pub type IParserError<'s> = ParserError<'s, &'s str, ICode, ()>;

#[derive(Debug)]
pub struct TerminalA<'s> {
    pub term: String,
    pub span: Span<'s>,
}

#[derive(Debug)]
pub struct TerminalB<'s> {
    pub term: String,
    pub span: Span<'s>,
}

#[derive(Debug)]
pub struct TerminalC<'s> {
    pub term: u32,
    pub span: Span<'s>,
}

#[derive(Debug)]
pub struct TerminalD<'s> {
    pub term: INummer<'s>,
    pub span: Span<'s>,
}

#[derive(Debug)]
pub struct NonTerminal1<'s> {
    pub a: TerminalA<'s>,
    pub b: TerminalB<'s>,
    pub span: Span<'s>,
}

#[derive(Debug)]
pub struct NonTerminal2<'s> {
    pub a: Option<TerminalA<'s>>,
    pub b: TerminalB<'s>,
    pub c: TerminalC<'s>,
    pub span: Span<'s>,
}

#[derive(Debug)]
pub struct INummer<'s> {
    pub nummer: u32,
    pub span: Span<'s>,
}

pub fn nom_parse_a(i: Span<'_>) -> INomResult<'_> {
    tag("A")(i)
}

pub fn nom_parse_b(i: Span<'_>) -> INomResult<'_> {
    tag("B")(i)
}

pub fn nom_parse_c(i: Span<'_>) -> INomResult<'_> {
    digit1(i)
}

pub fn parse_a(rest: Span<'_>) -> IParserResult<'_, TerminalA> {
    match nom_parse_a(rest) {
        Ok((rest, token)) => Ok((
            rest,
            TerminalA {
                term: token.to_string(),
                span: token,
            },
        )),
        Err(nom::Err::Error(e)) if e.is_error_kind(nom::error::ErrorKind::Tag) => {
            Err(nom::Err::Error(e.with_code(ICTerminalA)))
        }
        Err(e) => Err(e.into()),
    }
}

pub fn nom_star_star(i: Span<'_>) -> INomResult<'_> {
    terminated(recognize(tuple((nchar('*'), nchar('*')))), nom_ws)(i)
}

pub fn nom_tag_kdnr(i: Span<'_>) -> INomResult<'_> {
    terminated(recognize(tag_no_case("kdnr")), nom_ws)(i)
}

pub fn nom_ws(i: Span<'_>) -> INomResult<'_> {
    i.split_at_position_complete(|item| {
        let c = item.as_char();
        !(c == ' ' || c == '\t')
    })
}

pub fn nom_number(i: Span<'_>) -> INomResult<'_> {
    terminated(digit1, nom_ws)(i)
}

pub fn token_nummer(rest: Span<'_>) -> IParserResult<'_, INummer<'_>> {
    match nom_number(rest) {
        Ok((rest, tok)) => Ok((
            rest,
            INummer {
                nummer: tok.parse::<u32>().with_span(ICNummer, rest)?,
                span: tok,
            },
        )),
        Err(e) => Err(e.with_code(ICNummer)),
    }
}

fn parse_terminal_a(rest: Span<'_>) -> IParserResult<'_, TerminalA<'_>> {
    Context.enter(ICTerminalA, &rest);

    let (rest, token) = match parse_a(rest) {
        Ok((rest, token)) => (rest, token),
        Err(e) => return Context.err(e),
    };

    Context.ok(rest, token.span, token)
}

fn parse_terminal_a2(rest: Span<'_>) -> IParserResult<'_, TerminalA<'_>> {
    Context.enter(ICTerminalA, &rest);

    let (rest, token) = parse_a(rest).track()?;

    Context.ok(rest, token.span, token)
}

fn parse_terminal_b(rest: Span<'_>) -> IParserResult<'_, TerminalB<'_>> {
    Context.enter(ICTerminalB, &rest);

    let (rest, token) = nom_parse_b(rest).track()?;

    Context.ok(
        rest,
        token,
        TerminalB {
            term: token.to_string(),
            span: token,
        },
    )
}

pub struct ParseTerminalC;

fn parse_terminal_c(rest: Span<'_>) -> IParserResult<'_, TerminalC<'_>> {
    Context.enter(ICTerminalC, &rest);

    let (rest, (tok, v)) =
        consumed(transform(nom_parse_c, |v| (*v).parse::<u32>(), ICInteger))(rest).track()?;

    Context.ok(rest, tok, TerminalC { term: v, span: tok })
}

fn parse_terminal_d(input: Span<'_>) -> IParserResult<'_, TerminalD<'_>> {
    Context.enter(ICTerminalD, &input);

    let (rest, _) = opt(nom_star_star)(input).track()?;
    let (rest, tag) = nom_tag_kdnr(rest).track()?;
    let (rest, term) = token_nummer(rest).track()?;
    let (rest, _) = opt(nom_star_star)(rest).track()?;

    let span = input.span_union(&tag, &term.span);
    Context.ok(rest, span, TerminalD { term, span })
}

fn parse_non_terminal1(input: Span<'_>) -> IParserResult<'_, NonTerminal1<'_>> {
    Context.enter(ICNonTerminal1, &input);

    let (rest, a) = parse_terminal_a(input).track()?;
    let (rest, b) = parse_terminal_b(rest).track()?;

    let span = input.span_union(&a.span, &b.span);

    Context.ok(rest, span, NonTerminal1 { a, b, span })
}

fn parse_non_terminal1_1(rest: Span<'_>) -> IParserResult<'_, NonTerminal1<'_>> {
    Context.enter(ICNonTerminal1, &rest);

    let (rest, (token, (a, b))) =
        consumed(tuple((parse_terminal_a, parse_terminal_b)))(rest).track()?;

    Context.ok(rest, token, NonTerminal1 { a, b, span: token })
}

fn parse_non_terminal_2(input: Span<'_>) -> IParserResult<'_, NonTerminal2<'_>> {
    Context.enter(ICNonTerminal1, &input);

    let (rest, a) = opt(parse_terminal_a)(input).track()?;
    let (rest, b) = parse_terminal_b(rest).track()?;
    let (rest, c) = parse_terminal_c(rest).track()?;

    let span = if let Some(a) = &a {
        input.span_union(&a.span, &c.span)
    } else {
        c.span
    };

    Context.ok(rest, span, NonTerminal2 { a, b, c, span })
}

fn parse_non_terminal_3(rest: Span<'_>) -> IParserResult<'_, ()> {
    Context.enter(ICNonTerminal3, &rest);

    let mut loop_rest = rest;
    let mut err = None;
    loop {
        let rest2 = loop_rest;
        let (rest2, _a) = opt(parse_terminal_a)(rest2).track()?;
        let (rest2, _b) = match parse_terminal_b(rest2) {
            Ok((rest3, b)) => (rest3, Some(b)),
            Err(e) => {
                err.append(e)?;
                (rest2, None)
            }
        };

        if rest2.is_empty() {
            break;
        }

        // endless loop
        if loop_rest == rest2 {
            return Context.err(ParserError::new(ICNonTerminal3, rest2));
        }

        loop_rest = rest2;
    }

    Context.ok(rest, rest.take(0), ())
}

fn run_parser() {
    let ctx: TrackingContext<'_, &str, ICode, true> = TrackingContext::new("A");
    let span = ctx.span();

    let _r = parse_terminal_a(span);
}

fn run_parser2() {
    let span = NoContext.span("A");

    let _r = parse_terminal_a(span);
}

fn main() {
    run_parser();

    // don't know if tests in examples are a thing. simulate.
    test_terminal_a();
    test_nonterminal2();
}

const R: Trace = Trace;

// #[test]
pub fn test_terminal_a() {
    track_parse(&mut None, "A", parse_terminal_a).okok().q(R);
    track_parse(&mut None, "AA", parse_terminal_a).errerr().q(R);
}

pub fn test_nonterminal2() {
    track_parse(&mut None, "AAA", parse_non_terminal_2)
        .errerr()
        .q(R);
}
