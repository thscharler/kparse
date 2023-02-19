//!
//! Parser for a PLU transformation.
//!
//! Defines rules for what changes to the numbering system occurred over time,
//! thus allowing evaluation of historical data.
//!

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};

use kparse::test::{track_parse, CheckDump};
use kparse::tracker::TrackSpan;
use kparse::{Code, ParserError, ParserResult};
pub use parser::*;

fn main() {
    // call into test framework
    track_parse(&mut None, "1 -> 2\n", parse_plumap)
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
pub type PSpan<'s> = TrackSpan<'s, PLUCode, &'s str>;
pub type PLUParserResult<'s, O> = ParserResult<PLUCode, PSpan<'s>, O>;
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
    use kparse::spans::SpanLines;
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
        let txt = SpanLines::new(txt);
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

        let expect = err.expected_grouped_by_line();

        for t in &text1 {
            if t.location_line() == err.span.location_line() {
                println!("*{:04} {}", t.location_line(), t);
            } else {
                println!(" {:04}  {}", t.location_line(), t);
            }

            if expect.is_empty() {
                if t.location_line() == err.span.location_line() {
                    println!("      {}^", " ".repeat(err.span.get_utf8_column() - 1));
                    if !msg.is_empty() {
                        println!("expected: {}", msg);
                    } else {
                        println!("expected: {}", err.code);
                    }
                }
            }

            for (line, exp) in &expect {
                if t.location_line() == *line {
                    for exp in exp {
                        println!("      {}^", " ".repeat(exp.span.get_utf8_column() - 1));
                        println!("expected: {}", exp.code);
                    }
                }
            }
        }

        for (_line, sugg) in err.suggested_grouped_by_line() {
            for sug in sugg {
                println!("hint: {}", sug.code);
            }
        }

        if let Some(n) = err.nom() {
            println!(
                "parser details: {:?} {}:{}:\"{}\"",
                n.kind,
                n.span.location_line(),
                n.span.get_utf8_column(),
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
    use crate::{PLUParserError, PLUParserResult, PMap, PPluMap, PSpan};
    use kparse::combinators::error_code;
    use kparse::prelude::*;
    use kparse::Context;
    use nom::combinator::opt;
    use nom::sequence::preceded;
    use nom::Parser;

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

    /// main parser
    pub fn parse_plumap(input: PSpan<'_>) -> PLUParserResult<'_, PPluMap<'_>> {
        Context.enter(PLUMap, input);

        let mut r = Vec::new();

        let mut loop_rest = input;
        loop {
            let rest2 = loop_rest;

            let rest2 = if lah_comment(rest2) {
                let (rest3, _) = parse_kommentar(rest2).track()?;
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

        let (rest, nummer) = token_number(input).track()?;
        let (rest, faktor) = opt(preceded(nom_star_op, token_factor))(rest).track()?;

        let (rest, to_nummer) =
            preceded(error_code(nom_map_op, PLUMapOp), token_number)(rest).track()?;

        let (rest, from_datum) = opt(preceded(
            error_code(nom_range_start, PLURangeStart),
            token_date,
        ))(rest)
        .track()?;
        let (rest, to_datum) =
            opt(preceded(error_code(nom_range_end, PLURangeEnd), token_date))(rest).track()?;

        if !nom_is_nl(rest) {
            return Context.err(PLUParserError::new(PLUMapping, rest));
        }

        let span = input.span_union(&nummer.span, &nom_empty(rest));

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
    fn parse_kommentar(rest: PSpan<'_>) -> PLUParserResult<'_, ()> {
        Context.enter(PLUComment, rest);
        let (rest, kommentar) = nom_comment(rest).track()?;
        Context.ok(rest, kommentar, ())
    }
}

mod token {
    use crate::nom_parser::{nom_float, nom_minus, nom_number};
    use crate::PLUCode::*;
    use crate::{PDate, PFactor, PLUParserError, PLUParserResult, PNumber, PSpan};
    use kparse::prelude::*;

    /// factor
    pub fn token_factor(rest: PSpan<'_>) -> PLUParserResult<'_, PFactor<'_>> {
        match nom_float(rest) {
            Ok((rest, (tok, val))) => Ok((
                rest,
                PFactor {
                    faktor: val,
                    span: tok,
                },
            )),
            Err(e) => Err(e.with_code(PLUFactor)),
        }
    }

    /// token for the plu
    pub fn token_number(rest: PSpan<'_>) -> PLUParserResult<'_, PNumber<'_>> {
        match nom_number(rest) {
            Ok((rest, (tok, val))) => Ok((
                rest,
                PNumber {
                    nummer: val,
                    span: tok,
                },
            )),
            Err(e) => Err(e.with_code(PLUNumber)),
        }
    }

    /// token for a date
    pub fn token_date(input: PSpan<'_>) -> PLUParserResult<'_, PDate<'_>> {
        let (rest, (year_span, year)) = nom_number(input).with_code(PLUYear)?;
        let (rest, _) = nom_minus(rest).with_code(PLUMinus)?;
        let (rest, (_month_span, month)) = nom_number(rest).with_code(PLUMonth)?;
        let (rest, _) = nom_minus(rest).with_code(PLUMinus)?;
        let (rest, (day_span, day)) = nom_number(rest).with_code(PLUDay)?;

        let span = input.span_union(&year_span, &day_span);
        let datum = chrono::NaiveDate::from_ymd_opt(year as i32, month, day);

        if let Some(datum) = datum {
            Ok((rest, PDate { datum, span }))
        } else {
            Err(nom::Err::Error(PLUParserError::new(PLUDate, span)))
        }
    }
}

mod nom_parser {
    use crate::PLUCode::{PLUFactor, PLUNumber};
    use crate::{PLUNomResult, PLUParserError, PLUParserResult, PSpan};
    use kparse::combinators::transform;
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_till, take_while1};
    use nom::character::complete::{char as nchar, digit1, one_of};
    use nom::combinator::{consumed, opt, recognize};
    use nom::multi::many1;
    use nom::sequence::{terminated, tuple};
    use nom::InputTake;
    use nom::{AsChar, InputTakeAtPosition};
    use rust_decimal::Decimal;

    pub fn lah_comment(i: PSpan<'_>) -> bool {
        nchar::<_, nom::error::Error<PSpan<'_>>>('#')(i).is_ok()
    }

    pub fn nom_comment(i: PSpan<'_>) -> PLUParserResult<'_, PSpan<'_>> {
        recognize(tuple((
            terminated(tag("#"), nom_ws),
            terminated(take_till(|c: char| c == '\n'), nom_ws),
        )))(i)
    }

    /// numeric value.
    pub fn nom_float(input: PSpan<'_>) -> PLUParserResult<'_, (PSpan<'_>, Decimal)> {
        consumed(transform(
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
            ),
            |v| match (*v).parse::<Decimal>() {
                Ok(vv) => Ok(vv),
                Err(_) => Err(nom::Err::Failure(PLUParserError::new(PLUFactor, v))),
            },
        ))(input)
    }

    /// sequence of digits.
    pub fn decimal(input: PSpan<'_>) -> PLUNomResult<'_> {
        recognize(many1(one_of("0123456789")))(input)
    }

    pub fn lah_number(i: PSpan<'_>) -> bool {
        digit1::<_, nom::error::Error<PSpan<'_>>>(i).is_ok()
    }

    pub fn nom_number(i: PSpan<'_>) -> PLUParserResult<'_, (PSpan<'_>, u32)> {
        consumed(transform(terminated(digit1, nom_ws), |v| {
            match (*v).parse::<u32>() {
                Ok(vv) => Ok(vv),
                Err(_) => Err(nom::Err::Failure(PLUParserError::new(PLUNumber, v))),
            }
        }))(i)
    }

    pub fn nom_star_op(i: PSpan<'_>) -> PLUNomResult<'_> {
        terminated(recognize(nchar('*')), nom_ws)(i)
    }

    pub fn nom_map_op(i: PSpan<'_>) -> PLUNomResult<'_> {
        terminated(tag("->"), nom_ws)(i)
    }

    pub fn nom_range_start(i: PSpan<'_>) -> PLUNomResult<'_> {
        terminated(tag(">="), nom_ws)(i)
    }

    pub fn nom_range_end(i: PSpan<'_>) -> PLUNomResult<'_> {
        terminated(tag("<="), nom_ws)(i)
    }

    pub fn nom_minus(i: PSpan<'_>) -> PLUNomResult<'_> {
        terminated(recognize(nchar('-')), nom_ws)(i)
    }

    // regular whitespace
    pub fn nom_ws(i: PSpan<'_>) -> PLUNomResult<'_> {
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
