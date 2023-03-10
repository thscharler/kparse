//!
//! Parser für PLU Transformationen.
//!
//! Damit können alte Daten in das aktuelle Artikelschema übersetzt werden.
//!

use chrono::NaiveDate;
use kparse::prelude::*;
use kparse::test::{str_parse, CheckDump};
use kparse::{ParserError, ParserResult};
pub use parser::*;
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};

#[test]
fn test_plumap() {
    str_parse(
        &mut None,
        "1 -> 2\n
# comment",
        parse_plumap,
    )
    .ok_any()
    .q(CheckDump);
}

/// Parser Codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PLUCode {
    PLUNomError,
    PLUNomFailure,
    PLUIncomplete,

    PLUMap,
    PLUKommentar,
    PLUMapping,

    PLUNummer,
    PLUFaktor,
    PLUDatum,

    PLUStarOp,
    PLUMapOp,
    PLURangeStart,
    PLURangeEnd,
    PLUDay,
    PLUMonth,
    PLUYear,
    PLUMinus,
}

impl Display for PLUCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            PLUCode::PLUNomError => "NomError",
            PLUCode::PLUNomFailure => "NomFailure",
            PLUCode::PLUIncomplete => "ParseIncomplete",
            PLUCode::PLUKommentar => "Kommentar",
            PLUCode::PLUNummer => "Nummer",
            PLUCode::PLUFaktor => "Faktor",
            PLUCode::PLUDatum => "Datum",
            PLUCode::PLUStarOp => "*",
            PLUCode::PLUMapOp => "->",
            PLUCode::PLURangeStart => ">=",
            PLUCode::PLURangeEnd => "<=",
            PLUCode::PLUMap => "Map",
            PLUCode::PLUMapping => "Mapping",
            PLUCode::PLUDay => "Tag",
            PLUCode::PLUMonth => "Monat",
            PLUCode::PLUYear => "Jahr",
            PLUCode::PLUMinus => "-",
        };
        write!(f, "{}", name)?;
        Ok(())
    }
}

impl PLUCode {
    pub fn description(&self) -> &'static str {
        match self {
            PLUCode::PLUNomError => "NOM",
            PLUCode::PLUNomFailure => "NOM",
            PLUCode::PLUIncomplete => "Unvollständig",
            PLUCode::PLUMap => "PLU Regeln",
            PLUCode::PLUKommentar => "Kommentar",
            PLUCode::PLUMapping => "Regel",
            PLUCode::PLUNummer => "PLU",
            PLUCode::PLUFaktor => "Faktor",
            PLUCode::PLUDatum => "Datum",
            PLUCode::PLUStarOp => "*",
            PLUCode::PLUMapOp => "->",
            PLUCode::PLURangeStart => ">=",
            PLUCode::PLURangeEnd => "<=",
            PLUCode::PLUDay => "dd",
            PLUCode::PLUMonth => "mm",
            PLUCode::PLUYear => "yyyy",
            PLUCode::PLUMinus => "-",
        }
    }
}

impl Code for PLUCode {
    const NOM_ERROR: Self = Self::PLUNomError;
}

define_span!(PSpan = PLUCode, str);
pub type PLUParserResult<'s, O> = ParserResult<PLUCode, PSpan<'s>, O>;
pub type PLUNomResult<'s> = ParserResult<PLUCode, PSpan<'s>, PSpan<'s>>;
pub type PLUParserError<'s> = ParserError<PLUCode, PSpan<'s>>;

/// Gesamte Map.
#[derive(Debug)]
pub struct PPluMap<'s> {
    pub maps: Vec<PMap<'s>>,
}

/// Kommentar.
#[derive(Debug)]
pub struct PKommentar<'s> {
    pub span: PSpan<'s>,
}

/// Einzelne Regel.
#[derive(Debug)]
pub struct PMap<'s> {
    pub nummer: PNummer<'s>,
    pub faktor: Option<PFaktor<'s>>,
    pub to_nummer: PNummer<'s>,
    pub from_datum: Option<PDatum<'s>>,
    pub to_datum: Option<PDatum<'s>>,
    pub span: PSpan<'s>,
}

/// Nummer.
#[derive(Debug)]
pub struct PNummer<'s> {
    pub nummer: u32,
    pub span: PSpan<'s>,
}

/// Faktor.
#[derive(Debug)]
pub struct PFaktor<'s> {
    pub faktor: Decimal,
    pub span: PSpan<'s>,
}

/// Datum.
#[derive(Debug)]
pub struct PDatum<'s> {
    pub datum: NaiveDate,
    pub span: PSpan<'s>,
}

mod debug {
    use crate::{PLUCode, PLUParserError, PSpan};
    use kparse::prelude::*;
    use kparse::provider::TrackedDataVec;
    use kparse::Track;
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
        let txt = Track::source_str(txt.fragment());
        let text1 = txt.get_lines_around(err.span, 3);

        println!();
        if !msg.is_empty() {
            println!(
                "{}: {:?}: {}",
                if is_err { "FEHLER" } else { "Achtung" },
                src.file_name().unwrap_or_else(|| OsStr::new("")),
                msg
            );
        } else {
            println!(
                "{}: {:?}: {}",
                if is_err { "FEHLER" } else { "Achtung" },
                src.file_name().unwrap_or_else(|| OsStr::new("")),
                err.code
            );
        }

        let expect = err.iter_expected().collect::<Vec<_>>();

        for t in text1.iter().copied() {
            let t_line = txt.line(t);
            let s_line = txt.line(err.span);
            let s_column = txt.column(err.span);

            if t_line == s_line {
                println!("*{:04} {}", t_line, t);
            } else {
                println!(" {:04}  {}", t_line, t);
            }

            if expect.is_empty() {
                if t_line == s_line {
                    println!("      {}^", " ".repeat(s_column - 1));
                    if !msg.is_empty() {
                        println!("Erwarted war: {}", msg);
                    } else {
                        println!("Erwarted war: {}", err.code);
                    }
                }
            }

            for exp in expect.iter() {
                let e_line = txt.line(exp.span);
                let s_column = txt.column(exp.span);

                if t_line == e_line {
                    println!("      {}^", " ".repeat(s_column - 1));
                    println!("Erwarted war: {}", exp.code);
                }
            }
        }

        for sug in err.iter_suggested() {
            println!("Hinweis: {}", sug.code);
        }
    }

    /// Parser Trace.
    #[allow(dead_code)]
    pub fn dump_trace(tracks: &TrackedDataVec<PLUCode, &'_ str>) {
        println!("{:?}", tracks);
    }
}

mod parser {
    use crate::nom_parser::{
        lah_kommentar, lah_number, nom_empty, nom_is_nl, nom_kommentar, nom_map_op, nom_range_end,
        nom_range_start, nom_star_op, nom_ws_nl,
    };
    use crate::token::{token_datum, token_faktor, token_nummer};
    use crate::PLUCode::*;
    use crate::{PLUParserError, PLUParserResult, PMap, PPluMap, PSpan};
    use kparse::combinators::with_code;
    use kparse::prelude::*;
    use kparse::Track;
    use nom::combinator::opt;
    use nom::sequence::preceded;

    /// Parser.
    pub fn parse_plumap(input: PSpan<'_>) -> PLUParserResult<'_, PPluMap<'_>> {
        Track.enter(PLUMap, input);

        let mut r = Vec::new();

        let mut loop_rest = input;
        loop {
            let rest2 = loop_rest;

            let rest2 = if lah_kommentar(rest2) {
                let (rest3, _) = parse_kommentar(rest2).track()?;
                rest3
            } else if lah_mapping(rest2) {
                let (rest3, map) = parse_mapping(rest2).track()?;
                r.push(map);
                rest3
            } else {
                return Track.err(PLUParserError::new(PLUMapping, rest2));
            };

            loop_rest = nom_ws_nl(rest2);
            if loop_rest.is_empty() {
                break;
            }
        }
        let rest = loop_rest;

        Track.ok(rest, nom_empty(rest), PPluMap { maps: r })
    }

    /// Parser für ein Mapping.
    fn lah_mapping(span: PSpan<'_>) -> bool {
        lah_number(span)
    }

    fn parse_mapping(input: PSpan<'_>) -> PLUParserResult<'_, PMap<'_>> {
        Track.enter(PLUMapping, input);

        let (rest, nummer) = token_nummer(input).track()?;
        let (rest, faktor) = opt(preceded(nom_star_op, token_faktor))(rest).track()?;

        let (rest, to_nummer) =
            preceded(with_code(nom_map_op, PLUMapOp), token_nummer)(rest).track()?;

        let (rest, from_datum) = opt(preceded(
            with_code(nom_range_start, PLURangeStart),
            token_datum,
        ))(rest)
        .track()?;
        let (rest, to_datum) =
            opt(preceded(with_code(nom_range_end, PLURangeEnd), token_datum))(rest).track()?;

        if !nom_is_nl(rest) {
            return Track.err(PLUParserError::new(PLUMapping, rest));
        }

        let span = input.span_union(&nummer.span, &nom_empty(rest));

        Track.ok(
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

    fn parse_kommentar(rest: PSpan<'_>) -> PLUParserResult<'_, ()> {
        Track.enter(PLUKommentar, rest);
        let (rest, kommentar) = nom_kommentar(rest).track()?;
        Track.ok(rest, kommentar, ())
    }
}

mod token {
    use crate::nom_parser::{nom_float, nom_minus, nom_number};
    use crate::PLUCode::*;
    use crate::{PDatum, PFaktor, PLUParserError, PLUParserResult, PNummer, PSpan};
    use kparse::combinators::{map_res, with_code};
    use kparse::prelude::*;
    use kparse::{Code, ParserError};
    use nom::Parser;
    use rust_decimal::Decimal;

    /// Token für den Faktor.
    pub fn token_faktor(rest: PSpan<'_>) -> PLUParserResult<'_, PFaktor<'_>> {
        map_res(with_code(nom_float, PLUFaktor), |tok| {
            match tok.parse::<Decimal>() {
                Ok(v) => Ok(PFaktor {
                    faktor: v,
                    span: tok,
                }),
                Err(_) => Err(nom::Err::Failure(PLUParserError::new(PLUFaktor, tok))),
            }
        })
        .parse(rest)
    }

    /// Token für die Artikelnummer.
    pub fn token_nummer(rest: PSpan<'_>) -> PLUParserResult<'_, PNummer<'_>> {
        map_res(with_code(nom_number, PLUNummer), |tok| {
            match tok.parse::<u32>() {
                Ok(v) => Ok(PNummer {
                    nummer: v,
                    span: tok,
                }),
                Err(_) => Err(nom::Err::Failure(PLUParserError::new(PLUNummer, rest))),
            }
        })
        .parse(rest)
    }

    fn cnv_err<C, I, O, E>(
        r: Result<O, E>,
        code: C,
        span: I,
    ) -> Result<O, nom::Err<ParserError<C, I>>>
    where
        C: Code,
        I: Clone,
    {
        match r {
            Ok(v) => Ok(v),
            Err(_) => Err(nom::Err::Failure(ParserError::new(code, span))),
        }
    }

    /// Token für ein Datum.
    pub fn token_datum(input: PSpan<'_>) -> PLUParserResult<'_, PDatum<'_>> {
        let (rest, year) = nom_number(input).with_code(PLUYear)?;
        let (rest, _) = nom_minus(rest).with_code(PLUMinus)?;
        let (rest, month) = nom_number(rest).with_code(PLUMonth)?;
        let (rest, _) = nom_minus(rest).with_code(PLUMinus)?;
        let (rest, day) = nom_number(rest).with_code(PLUDay)?;

        let iyear: i32 = cnv_err((*year).parse::<i32>(), PLUYear, year)?;
        let imonth: u32 = cnv_err((*month).parse::<u32>(), PLUMonth, month)?;
        let iday: u32 = cnv_err((*day).parse::<u32>(), PLUDay, day)?;

        let span = input.span_union(&year, &day);
        let datum = chrono::NaiveDate::from_ymd_opt(iyear, imonth, iday);

        if let Some(datum) = datum {
            Ok((rest, PDatum { datum, span }))
        } else {
            Err(nom::Err::Error(PLUParserError::new(PLUDatum, span)))
        }
    }
}

mod nom_parser {
    use crate::{PLUNomResult, PLUParserResult, PSpan};
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_till, take_while1};
    use nom::character::complete::{char as nchar, digit1, one_of};
    use nom::combinator::{opt, recognize};
    use nom::multi::many1;
    use nom::sequence::{terminated, tuple};
    use nom::InputTake;
    use nom::{AsChar, InputTakeAtPosition};

    pub fn lah_kommentar(i: PSpan<'_>) -> bool {
        nchar::<_, nom::error::Error<PSpan<'_>>>('#')(i).is_ok()
    }

    pub fn nom_kommentar(i: PSpan<'_>) -> PLUParserResult<'_, PSpan<'_>> {
        recognize(tuple((
            terminated(tag("#"), nom_ws),
            terminated(take_till(|c: char| c == '\n'), nom_ws),
        )))(i)
    }

    /// Numeric value.
    pub fn nom_float(input: PSpan<'_>) -> PLUNomResult<'_> {
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
        )(input)
    }

    /// Sequence of digits.
    pub fn decimal(input: PSpan<'_>) -> PLUNomResult<'_> {
        recognize(many1(one_of("0123456789")))(input)
    }

    pub fn lah_number(i: PSpan<'_>) -> bool {
        digit1::<_, nom::error::Error<PSpan<'_>>>(i).is_ok()
    }

    pub fn nom_number(i: PSpan<'_>) -> PLUNomResult<'_> {
        terminated(digit1, nom_ws)(i)
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

    pub fn nom_ws(i: PSpan<'_>) -> PLUNomResult<'_> {
        i.split_at_position_complete(|item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t')
        })
    }

    pub fn nom_ws_nl(i: PSpan<'_>) -> PSpan<'_> {
        match i.split_at_position_complete::<_, nom::error::Error<PSpan<'_>>>(|item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t' || c == '\n' || c == '\r')
        }) {
            Ok((rest, _)) => rest,
            Err(_) => i,
        }
    }

    pub fn nom_is_nl(i: PSpan<'_>) -> bool {
        terminated(
            recognize(take_while1(|c: char| c == '\n' || c == '\r')),
            nom_ws,
        )(i)
        .is_ok()
    }

    pub fn nom_empty(i: PSpan<'_>) -> PSpan<'_> {
        i.take(0)
    }
}
