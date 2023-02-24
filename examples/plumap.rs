//!
//! Parser for a PLU transformation.
//!
//! Defines rules for what changes to the numbering system occurred over time,
//! thus allowing evaluation of historical data.
//!

use chrono::NaiveDate;
use kparse::test::{str_parse, CheckDump};
#[cfg(debug_assertions)]
use kparse::tracker::TrackSpan;
use kparse::{Code, ParserError, ParserResult, TokenizerResult};
pub use parser::*;
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};

fn main() {
    // call into test framework
    str_parse(&mut None, "1 -> 2\n", parse_plumap)
        .ok_any()
        .q(CheckDump);
}

/// Parser Codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PLUCode {
    PLUNomError,

    PLUMap,
    PLUMapping,
    PLUComment,

    PLUNumber,
    PLUFactor,
    PLUDate,

    PLUDay,
    PLUMapOp,
    PLUMinus,
    PLUMonth,
    PLURangeEnd,
    PLURangeStart,
    PLUStarOp,
    PLUYear,
}

impl Display for PLUCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            PLUCode::PLUNomError => "NomError",

            PLUCode::PLUMap => "map",
            PLUCode::PLUMapping => "mapping",
            PLUCode::PLUComment => "comment",

            PLUCode::PLUNumber => "number",
            PLUCode::PLUFactor => "factor",
            PLUCode::PLUDate => "date",

            PLUCode::PLUDay => "day",
            PLUCode::PLUMapOp => "->",
            PLUCode::PLUMinus => "-",
            PLUCode::PLUMonth => "month",
            PLUCode::PLURangeEnd => "<=",
            PLUCode::PLURangeStart => ">=",
            PLUCode::PLUStarOp => "*",
            PLUCode::PLUYear => "year",
        };
        write!(f, "{}", name)?;
        Ok(())
    }
}

impl PLUCode {
    pub fn description(&self) -> &'static str {
        match self {
            PLUCode::PLUNomError => "NOM",

            PLUCode::PLUMap => "PLU rules",
            PLUCode::PLUMapping => "rule",
            PLUCode::PLUComment => "comment",

            PLUCode::PLUNumber => "PLU",
            PLUCode::PLUFactor => "factor",
            PLUCode::PLUDate => "date",

            PLUCode::PLUDay => "dd",
            PLUCode::PLUMapOp => "->",
            PLUCode::PLUMinus => "-",
            PLUCode::PLUMonth => "mm",
            PLUCode::PLURangeEnd => "<=",
            PLUCode::PLURangeStart => ">=",
            PLUCode::PLUStarOp => "*",
            PLUCode::PLUYear => "yyyy",
        }
    }
}

// Define as our error code.
impl Code for PLUCode {
    const NOM_ERROR: Self = Self::PLUNomError;
}

// type aliases to avoid typing PLUCode all the time.
#[cfg(debug_assertions)]
pub type PSpan<'s> = TrackSpan<'s, PLUCode, &'s str>;
#[cfg(not(debug_assertions))]
pub type PSpan<'s> = &'s str;
pub type PLUParserResult<'s, O> = ParserResult<PLUCode, PSpan<'s>, O>;
pub type PLUTokenizerResult<'s, O> = TokenizerResult<PLUCode, PSpan<'s>, O>;
pub type PLUNomResult<'s> = ParserResult<PLUCode, PSpan<'s>, PSpan<'s>>;
pub type PLUParserError<'s> = ParserError<PLUCode, PSpan<'s>>;

/// complete
#[derive(Debug)]
pub struct PPluMap<'s> {
    pub maps: Vec<PMap<'s>>,
}

/// single rule.
#[derive(Debug)]
pub struct PMap<'s> {
    pub nummer: PNumber<'s>,
    pub faktor: Option<PFactor<'s>>,
    pub to_nummer: PNumber<'s>,
    pub from_datum: Option<PDate<'s>>,
    pub to_datum: Option<PDate<'s>>,
    pub span: PSpan<'s>,
}

/// Nummer.
#[derive(Debug)]
pub struct PNumber<'s> {
    pub nummer: u32,
    pub span: PSpan<'s>,
}

/// Faktor.
#[derive(Debug)]
pub struct PFactor<'s> {
    pub faktor: Decimal,
    pub span: PSpan<'s>,
}

/// Datum.
#[derive(Debug)]
pub struct PDate<'s> {
    pub datum: NaiveDate,
    pub span: PSpan<'s>,
}

mod debug {
    use crate::{PLUCode, PLUParserError, PSpan};
    #[cfg(debug_assertions)]
    use kparse::spans::SpanLines;
    #[cfg(not(debug_assertions))]
    use kparse::spans::SpanStr;
    use kparse::tracker::Tracks;
    use std::ffi::OsStr;
    use std::path::Path;

    /// Fehler Diagnose.
    #[allow(dead_code)]
    pub fn dump_diagnostics(
        src: &Path,
        txt: PSpan<'_>,
        err: &PLUParserError<'_>,
        msg: &str,
        is_err: bool,
    ) {
        #[cfg(debug_assertions)]
        let txt = SpanLines::new(txt);
        #[cfg(not(debug_assertions))]
        let txt = SpanStr::new(txt);

        let text1 = txt.get_lines_around(&err.span, 3);

        println!();
        if !msg.is_empty() {
            println!(
                "{}: {:?}: {}",
                if is_err { "ERROR" } else { "Warning" },
                src.file_name().unwrap_or_else(|| OsStr::new("")),
                msg
            );
        } else {
            println!(
                "{}: {:?}: {}",
                if is_err { "ERROR" } else { "Warning" },
                src.file_name().unwrap_or_else(|| OsStr::new("")),
                err.code
            );
        }

        let expect = err.iter_expected().collect::<Vec<_>>();

        for t in text1.iter().copied() {
            let t_line = txt.line(t);
            let s_line = txt.line(err.span);
            let s_column = txt.utf8_column(err.span);

            if t_line == s_line {
                println!("*{:04} {}", t_line, t);
            } else {
                println!(" {:04}  {}", t_line, t);
            }

            if expect.is_empty() {
                if t_line == s_line {
                    println!("      {}^", " ".repeat(s_column - 1));
                    if !msg.is_empty() {
                        println!("expected: {}", msg);
                    } else {
                        println!("expected: {}", err.code);
                    }
                }
            }

            for exp in &expect {
                let e_line = txt.line(exp.span);
                let e_column = txt.utf8_column(exp.span);
                if t_line == e_line {
                    println!("      {}^", " ".repeat(e_column - 1));
                    println!("expected: {}", exp.code);
                }
            }
        }

        for sugg in err.iter_suggested() {
            println!("hint: {}", sugg.code);
        }

        if let Some(n) = err.nom() {
            let n_line = txt.line(n.span);
            let n_column = txt.utf8_column(n.span);
            println!(
                "parser details: {:?} {}:{}:\"{}\"",
                n.kind,
                n_line,
                n_column,
                n.span.escape_debug().take(40).collect::<String>()
            );
        }
    }

    /// dump the parser trace
    #[allow(dead_code)]
    pub fn dump_trace(tracks: &Tracks<PLUCode, &'_ str>) {
        println!("{:?}", tracks);
    }
}

mod parser {
    use crate::nom_parser::{
        lah_comment, lah_number, nom_comment, nom_empty, nom_is_nl, nom_map_op, nom_range_end,
        nom_range_start, nom_star_op, nom_ws_nl,
    };
    use crate::token::{token_date, token_factor, token_number};
    use crate::PLUCode::*;
    use crate::{PLUParserError, PLUParserResult, PLUTokenizerResult, PMap, PPluMap, PSpan};
    use kparse::combinators::track;
    use kparse::prelude::*;
    use kparse::Context;
    use nom::combinator::{consumed, opt};
    use nom::sequence::{preceded, tuple};
    use nom::Parser;

    /// main parser
    pub fn parse_plumap(input: PSpan<'_>) -> PLUParserResult<'_, PPluMap<'_>> {
        Context.enter(PLUMap, input);

        let mut r = Vec::new();

        let mut loop_rest = input;
        loop {
            let rest2 = loop_rest;

            let rest2 = if lah_comment(rest2) {
                let (rest3, _) = parse_kommentar.err_into().parse(rest2).track()?;
                rest3
            } else if lah_mapping(rest2) {
                let (rest3, map) = parse_mapping(rest2).track()?;
                r.push(map);
                rest3
            } else {
                return Context.err(PLUParserError::new(PLUMapping, rest2));
            };

            loop_rest = nom_ws_nl(rest2); // eat all whitespace & line breaks
            if loop_rest.is_empty() {
                break;
            }
        }
        let rest = loop_rest;

        Context.ok(rest, nom_empty(rest), PPluMap { maps: r })
    }

    fn lah_mapping(span: PSpan<'_>) -> bool {
        lah_number(span)
    }

    // parse one mapping
    fn parse_mapping(input: PSpan<'_>) -> PLUParserResult<'_, PMap<'_>> {
        Context.enter(PLUMapping, input);

        let (rest, (span, (nummer, faktor, to_nummer, from_datum, to_datum))) = consumed(tuple((
            token_number,
            opt(preceded(nom_star_op, token_factor)),
            preceded(nom_map_op, token_number),
            opt(preceded(nom_range_start, token_date)),
            opt(preceded(nom_range_end, token_date)),
        )))
        .err_into()
        .parse(input)
        .track()?;

        if !nom_is_nl(rest) {
            return Context.err(PLUParserError::new(PLUMapping, rest));
        }

        Context.ok(
            rest,
            span,
            PMap {
                nummer,
                faktor,
                to_nummer,
                from_datum,
                to_datum,
                span,
            },
        )
    }

    // parse and ignore
    fn parse_kommentar(rest: PSpan<'_>) -> PLUTokenizerResult<'_, ()> {
        track(PLUComment, nom_comment) //
            .map(|_| ())
            .parse(rest)
    }
}

mod token {
    use crate::nom_parser::{nom_float, nom_minus, nom_number};
    use crate::PLUCode::*;
    use crate::{PDate, PFactor, PLUTokenizerResult, PNumber, PSpan};
    use kparse::prelude::*;
    use kparse::token_error::TokenizerError;
    use nom::combinator::consumed;
    use nom::sequence::tuple;
    use nom::Parser;

    /// factor
    pub fn token_factor(rest: PSpan<'_>) -> PLUTokenizerResult<'_, PFactor<'_>> {
        nom_float
            .map(|(span, faktor)| PFactor { faktor, span })
            .parse(rest)
    }

    /// token for the plu
    pub fn token_number(rest: PSpan<'_>) -> PLUTokenizerResult<'_, PNumber<'_>> {
        nom_number
            .map(|(span, nummer)| PNumber { nummer, span })
            .parse(rest)
    }

    /// token for a date
    pub fn token_date(input: PSpan<'_>) -> PLUTokenizerResult<'_, PDate<'_>> {
        let (rest, (span, (year, _, month, _, day))) = consumed(tuple((
            nom_number.with_code(PLUYear),
            nom_minus,
            nom_number.with_code(PLUMonth),
            nom_minus,
            nom_number.with_code(PLUDay),
        )))
        .parse(input)?;

        let datum = chrono::NaiveDate::from_ymd_opt(year.1 as i32, month.1, day.1);

        if let Some(datum) = datum {
            Ok((rest, PDate { datum, span }))
        } else {
            Err(nom::Err::Error(TokenizerError::new(PLUDate, span)))
        }
    }
}

mod nom_parser {
    use crate::PLUCode::{
        PLUComment, PLUFactor, PLUMapOp, PLUMinus, PLUNumber, PLURangeEnd, PLURangeStart, PLUStarOp,
    };
    use crate::{PLUTokenizerResult, PSpan};
    use kparse::prelude::*;
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_till, take_while1};
    use nom::character::complete::{char as nchar, digit1, one_of};
    use nom::combinator::{consumed, opt, recognize};
    use nom::multi::many1;
    use nom::sequence::{terminated, tuple};
    use nom::{AsChar, InputTakeAtPosition};
    use nom::{InputTake, Parser};
    use rust_decimal::Decimal;

    pub fn lah_comment(i: PSpan<'_>) -> bool {
        nchar::<_, nom::error::Error<PSpan<'_>>>('#')(i).is_ok()
    }

    pub fn nom_comment(i: PSpan<'_>) -> PLUTokenizerResult<'_, PSpan<'_>> {
        recognize(tuple((
            terminated(tag("#"), nom_ws),
            terminated(take_till(|c: char| c == '\n'), nom_ws),
        )))
        .with_code(PLUComment)
        .parse(i)
    }

    /// numeric value.
    pub fn nom_float(input: PSpan<'_>) -> PLUTokenizerResult<'_, (PSpan<'_>, Decimal)> {
        consumed(
            terminated(
                alt((
                    // Case one: .42
                    recognize(tuple((
                        nchar('.'),
                        decimal,
                        opt(tuple((one_of("eE"), opt(one_of("+-")), decimal))),
                    ))),
                    // Case two: 42e42 and 42.42e42
                    recognize(tuple((
                        decimal,
                        opt(tuple((nchar('.'), opt(decimal)))),
                        one_of("eE"),
                        opt(one_of("+-")),
                        decimal,
                    ))),
                    // Case three: 42 and 42. and 42.42
                    recognize(tuple((
                        decimal, //
                        opt(tuple((
                            nchar('.'), //
                            opt(decimal),
                        ))), //
                    ))),
                )),
                nom_ws,
            )
            .parse_from_str::<_, Decimal>(PLUFactor),
        )(input)
    }

    /// sequence of digits.
    pub fn decimal(input: PSpan<'_>) -> PLUTokenizerResult<'_, PSpan<'_>> {
        recognize(many1(one_of("0123456789")))(input)
    }

    pub fn lah_number(i: PSpan<'_>) -> bool {
        digit1::<_, nom::error::Error<PSpan<'_>>>(i).is_ok()
    }

    pub fn nom_number(i: PSpan<'_>) -> PLUTokenizerResult<'_, (PSpan<'_>, u32)> {
        consumed(terminated(digit1, nom_ws).parse_from_str::<_, u32>(PLUNumber)).parse(i)
    }

    pub fn nom_star_op(i: PSpan<'_>) -> PLUTokenizerResult<'_, PSpan<'_>> {
        terminated(recognize(nchar('*')), nom_ws)
            .with_code(PLUStarOp)
            .parse(i)
    }

    pub fn nom_map_op(i: PSpan<'_>) -> PLUTokenizerResult<'_, PSpan<'_>> {
        terminated(tag("->"), nom_ws) //
            .with_code(PLUMapOp)
            .parse(i)
    }

    pub fn nom_range_start(i: PSpan<'_>) -> PLUTokenizerResult<'_, PSpan<'_>> {
        terminated(tag(">="), nom_ws) //
            .with_code(PLURangeStart)
            .parse(i)
    }

    pub fn nom_range_end(i: PSpan<'_>) -> PLUTokenizerResult<'_, PSpan<'_>> {
        terminated(tag("<="), nom_ws) //
            .with_code(PLURangeEnd)
            .parse(i)
    }

    pub fn nom_minus(i: PSpan<'_>) -> PLUTokenizerResult<'_, PSpan<'_>> {
        terminated(recognize(nchar('-')), nom_ws) //
            .with_code(PLUMinus)
            .parse(i)
    }

    // regular whitespace
    pub fn nom_ws(i: PSpan<'_>) -> PLUTokenizerResult<'_, PSpan<'_>> {
        i.split_at_position_complete(|item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t')
        })
    }

    // whitespace and line breaks.
    pub fn nom_ws_nl(i: PSpan<'_>) -> PSpan<'_> {
        match i.split_at_position_complete::<_, nom::error::Error<PSpan<'_>>>(|item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t' || c == '\n' || c == '\r')
        }) {
            Ok((rest, _)) => rest,
            Err(_) => i,
        }
    }

    // check for completeness
    pub fn nom_is_nl(i: PSpan<'_>) -> bool {
        terminated(
            recognize(take_while1(|c: char| c == '\n' || c == '\r')),
            nom_ws,
        )(i)
        .is_ok()
    }

    // create an empty span.
    pub fn nom_empty(i: PSpan<'_>) -> PSpan<'_> {
        i.take(0)
    }
}
