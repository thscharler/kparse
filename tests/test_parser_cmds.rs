use chrono::NaiveDate;
use kparse::prelude::*;
use nom::bytes::complete::{tag, take_till, take_till1, take_while1};
use nom::character::complete::{char as nchar, digit1};
use nom::combinator::recognize;
use nom::error::ParseError;
use nom::sequence::terminated;
use nom::InputTake;
use nom::{AsChar, InputTakeAtPosition};
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;

use kparse::Context;
pub use CCode::*;

pub type Span<'s> = kparse::Span<'s, CCode>;
pub type CParserError<'s> = ParserError<'s, CCode>;
pub type CParserResult<'s, O> = ParserResult<'s, O, CCode, ()>;
pub type CNomResult<'s> = ParserNomResult<'s, CCode, ()>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CCode {
    CNomError,
    CNomFailure,
    CParseIncomplete,
    CIgnore,
    CWhitespace,

    CCommand,
    CCreate,
    CDebug,
    CDiff,
    CEtik,
    CExport,
    CGet,
    CHelp,
    CImport,
    CNew,
    CPrint,
    CReport,
    CSend,
    CSet,
    CTest,

    CArtikel,
    CBm,
    CBs,
    CElcom,
    CFile,
    CFink,
    CGastro,
    CKassa,
    CKunde,
    CKundeArtikel,
    CKw,
    CLf,
    CMitarbeiter,
    CMonth,
    CYear,
    COff,
    COn,
    CPlan,
    CRe,
    CRind,
    CStatistik,
    CWeek,
    CWix,
    CWixId,
    CZeiterfassung,
    CReDatum,

    CDatum,
    CDateDay,
    CDotDay,
    CDateMonth,
    CDotMonth,
    CDateYear,
    CInteger,
    CNummer,
    CString,
    CFileName,
}

impl Code for CCode {
    const NOM_ERROR: Self = CNomError;
}

impl Display for CCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            CNomError => "NomError",
            CNomFailure => "NomFailure",
            CParseIncomplete => "ParseIncomplete",
            CIgnore => "Ignore",

            CCreate => "Create",
            CDebug => "Debug",
            CDiff => "Diff",
            CEtik => "Etik",
            CExport => "Export",
            CGet => "Get",
            CHelp => "Help",
            CImport => "Import",
            CNew => "New",
            CPrint => "Print",
            CReport => "Report",
            CSend => "Send",
            CSet => "Set",
            CTest => "Test",

            CArtikel => "Artikel",
            CBm => "Bm",
            CBs => "Bs",
            CCommand => "Command",
            CElcom => "Elcom",
            CFile => "File",
            CFink => "Fink",
            CGastro => "Gastro",
            CKassa => "Kassa",
            CKunde => "Kunde",
            CKundeArtikel => "KundeArtikel",
            CKw => "Kw",
            CLf => "Lf",
            CMitarbeiter => "Mitarbeiter",
            CMonth => "Month",
            COff => "Off",
            COn => "On",
            CPlan => "Plan",
            CRe => "Re",
            CReDatum => "ReDatum",
            CRind => "Rind",
            CStatistik => "Statistik",
            CWeek => "Week",
            CWix => "Wix",
            CWixId => "WixId",
            CYear => "Year",
            CZeiterfassung => "Zeiterfassung",

            CDatum => "Datum",
            CDateDay => "Day",
            CDotDay => "Dot",
            CDateMonth => "Month",
            CDotMonth => "Dot",
            CDateYear => "Year",
            CInteger => "Integer",
            CNummer => "Nummer",
            CWhitespace => "Whitespace1",
            CString => "String",
            CFileName => "FileName",
        };
        write!(f, "{}", name)
    }
}

// AST -------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum BCommand {
    Create(Create),
    Print(Print),
    Import(Import),
    Export(Export),
    New(New),
    Diff(Diff),
    Etik(Etik),
    Report(Report),
    SendMail(SendMail),
    Set(Set),
    Get(Get),
    Help(Help),
    Debug(Debugging),
    Test(Test),
    None(),
}

#[derive(Debug, Clone, Copy)]
pub enum Create {
    KundeArt,
    WixId,
}

#[derive(Debug, Clone, Copy)]
pub enum Print {
    KundeArtikel,
    Kunde,
    Artikel,
    Elcom,
    Kw,
    Week,
    Month,
    Year,
    Bs,
    Lf,
    Re,
}

#[derive(Debug, Clone, Copy)]
pub enum Import {
    Elcom,
    Wix,
}

#[derive(Debug, Clone, Copy)]
pub enum Export {
    Elcom,
    Plan,
    Wix,
}

#[derive(Debug, Clone, Copy)]
pub enum New {
    Bs(Option<u32>),
    Lf(Option<u32>),
    Re(Option<u32>),
}

#[derive(Debug, Clone, Copy)]
pub enum Set {
    ReDatum(NaiveDate),
}

#[derive(Debug, Clone, Copy)]
pub enum Get {
    ReDatum,
}

#[derive(Debug, Clone, Copy)]
pub enum Diff {
    Elcom,
}

#[derive(Debug, Clone, Copy)]
pub enum Etik {
    EtikBs(Option<u32>),
    EtikFile,
}

#[derive(Debug, Clone, Copy)]
pub enum Report {
    Mitarbeiter(Option<NaiveDate>),
    Zeiterfassung(Option<NaiveDate>),
    Rind,
    Fink,
    Bm,
    Gastro,
    Statistik,
}

#[derive(Debug, Clone, Copy)]
pub enum SendMail {
    Gastro,
}

#[derive(Debug, Clone, Copy)]
pub enum Help {
    Help,
}

#[derive(Debug, Clone, Copy)]
pub enum Debugging {
    Off,
    On,
}

#[derive(Debug, Clone, Copy)]
pub enum Test {
    Test,
}

#[derive(Debug, Clone, Copy)]
pub struct Datum<'s> {
    pub datum: NaiveDate,
    pub span: Span<'s>,
}

#[derive(Debug, Clone, Copy)]
pub struct Nummer<'s> {
    pub nummer: u32,
    pub span: Span<'s>,
}

#[derive(Debug, Clone)]
pub struct Datei<'s> {
    pub path: PathBuf,
    pub span: Span<'s>,
}

// Generic parsers -------------------------------------------------------

pub struct Parse1LayerCommand {
    cmd: BCommand,
    layers: Parse1Layers,
}

impl Parse1LayerCommand {
    fn id(&self) -> CCode {
        self.layers.code
    }

    fn lah(&self, span: Span<'_>) -> bool {
        lah_command(self.layers.token, span)
    }

    fn parse<'s>(&self, rest: Span<'s>) -> CParserResult<'s, BCommand> {
        Context.enter(&rest, self.id());

        let (rest, sub) = self.layers.parse(rest).track()?;

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return Context.err(ParserError::new(self.id(), rest));
        }

        Context.ok(rest, sub, self.cmd)
    }
}

pub struct Parse1Layers {
    pub token: &'static str,
    pub code: CCode,
}

impl Parse1Layers {
    fn id(&self) -> CCode {
        self.code
    }

    fn parse<'s>(&self, rest: Span<'s>) -> CParserResult<'s, Span<'s>> {
        Context.enter(&rest, self.id());

        let (rest, token) = token_command(self.token, self.code, rest).track()?;

        Context.ok(rest, token, token)
    }
}

pub struct Parse2LayerCommand<O: Copy + Debug, const N: usize> {
    map_cmd: fn(O) -> BCommand,
    layers: Parse2Layers<O, N>,
}

impl<O: Copy + Debug, const N: usize> Parse2LayerCommand<O, N> {
    fn lah(&self, span: Span<'_>) -> bool {
        lah_command(self.layers.token, span)
    }

    fn parse<'s>(&self, rest: Span<'s>) -> CParserResult<'s, BCommand> {
        Context.enter(&rest, self.layers.code);

        let (rest, (span, sub)) = self.layers.parse(rest).track()?;

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return Context.err(ParserError::new(self.layers.code, rest));
        }

        Context.ok(rest, span, (self.map_cmd)(sub))
    }
}

pub struct Parse2Layers<O: Copy, const N: usize> {
    pub token: &'static str,
    pub code: CCode,
    pub list: [SubCmd<O>; N],
}

pub struct SubCmd<O: Copy> {
    pub token: &'static str,
    pub code: CCode,
    pub output: O,
}

impl<O: Copy> From<(&'static str, CCode, O)> for SubCmd<O> {
    fn from(t: (&'static str, CCode, O)) -> Self {
        SubCmd {
            token: t.0,
            code: t.1,
            output: t.2,
        }
    }
}

impl<O: Copy, const N: usize> Parse2Layers<O, N> {
    fn parse<'s>(&self, rest: Span<'s>) -> CParserResult<'s, (Span<'s>, O)> {
        Context.enter(&rest, self.code);

        let (rest, token) = token_command(self.token, self.code, rest).track()?;
        Context.debug(&token, format!("found {}", token));

        let (rest, _) = nom_ws1(rest).track()?;

        let (rest, span_sub, sub) = 'for_else: {
            let mut err: Option<CParserError<'_>> = None;
            for sub in &self.list {
                match token_command(sub.token, sub.code, rest) {
                    Ok((rest, span)) => {
                        // println!("found");
                        break 'for_else (rest, span, sub);
                    }
                    Err(nom::Err::Error(e)) => {
                        if e.code != CIgnore {
                            err = if let Some(err) = err {
                                Some(err.or(e))
                            } else {
                                Some(e)
                            };
                        }
                    }
                    Err(e) => return Context.err(e),
                }
            }
            return match err {
                Some(err) => Context.err(err),
                None => {
                    let mut err = ParserError::new(self.code, rest);
                    for sub in &self.list {
                        err.add_suggest(sub.code, rest);
                    }
                    Context.err(err)
                }
            };
        };

        let span = unsafe { Context.span_union(&token, &span_sub) };
        Context.ok(rest, span, (span_sub, sub.output))
    }
}

// Tokens ----------------------------------------------------------------
// impl<'s, T> IntoParserResultAddSpan<'s, CCode, T> for Result<T, ParseIntError> {
//     fn into_with_span(self, span: Span<'s>) -> ParserResult<'s, CCode, T> {
//         match self {
//             Ok(v) => Ok(v),
//             Err(_) => Err(ParserError::new(CInteger, span)),
//         }
//     }
// }

pub fn token_nummer(rest: Span<'_>) -> CParserResult<'_, Nummer<'_>> {
    match nom_number(rest) {
        Ok((rest, tok)) => Ok((
            rest,
            Nummer {
                nummer: tok.parse::<u32>().with_span(CNummer, rest)?,
                span: tok,
            },
        )),
        Err(e) => Err(e.with_code(CNummer)),
    }
}

pub fn token_datum(rest: Span<'_>) -> CParserResult<'_, Datum<'_>> {
    let (rest, day) = nom_number(rest).with_code(CDateDay)?;
    let (rest, _) = nom_dot(rest).with_code(CDotDay)?;
    let (rest, month) = nom_number(rest).with_code(CDateMonth)?;
    let (rest, _) = nom_dot(rest).with_code(CDotMonth)?;
    let (rest, year) = nom_number(rest).with_code(CDateYear)?;

    let iday: u32 = (*day).parse().with_span(CDateDay, day)?;
    let imonth: u32 = (*month).parse().with_span(CDateMonth, month)?;
    let iyear: i32 = (*year).parse().with_span(CDateYear, year)?;

    let span = unsafe { Context.span_union(&day, &year) };
    let datum = NaiveDate::from_ymd_opt(iyear, imonth, iday);

    if let Some(datum) = datum {
        Ok((rest, Datum { datum, span }))
    } else {
        Err(nom::Err::Error(ParserError::new(CDatum, span)))
    }
}

fn lah_command(tok: &'_ str, rest: Span<'_>) -> bool {
    match tag::<_, _, CParserError<'_>>(tok)(rest) {
        Ok(_) => true,
        Err(_) => match nom_last_token(rest) {
            Ok((_, last)) => {
                let last = last.to_lowercase();
                tok.starts_with(&last)
            }
            Err(_) => false,
        },
    }
}

/// Tries to parse the token. If it fails and at least partially matches it adds a Suggest.
fn token_command<'a>(tok: &'_ str, code: CCode, rest: Span<'a>) -> CParserResult<'a, Span<'a>> {
    let (rest, token) = match tag::<_, _, CParserError<'a>>(tok)(rest) {
        Ok((rest, token)) => (rest, token),
        Err(nom::Err::Error(_) | nom::Err::Failure(_)) => {
            //
            match nom_last_token(rest) {
                Ok((rest, last)) => {
                    let err = if tok.starts_with(&last.to_lowercase()) {
                        CParserError::new_suggest(code, last)
                    } else {
                        CParserError::new(CIgnore, rest)
                    };
                    return Err(nom::Err::Error(err));
                }
                Err(_) => return Err(nom::Err::Error(CParserError::new(CIgnore, rest))),
            }
        }
        Err(nom::Err::Incomplete(_)) => unreachable!(),
    };

    Ok((rest, token))
}

/// Returns a token, but only if it ends the line.
pub fn nom_last_token(i: Span<'_>) -> CNomResult<'_> {
    match recognize::<_, _, CParserError<'_>, _>(take_till1(|c: char| c == ' ' || c == '\t'))(i) {
        Ok((rest, tok)) if rest.is_empty() => Ok((rest, tok)),
        _ => Err(nom::Err::Error(ParserError::new(CNomError, i))),
    }
}

pub fn nom_number(i: Span<'_>) -> CNomResult<'_> {
    terminated(digit1, nom_ws)(i)
}

pub fn nom_dot(i: Span<'_>) -> CNomResult<'_> {
    terminated(recognize(nchar('.')), nom_ws)(i)
}

pub fn nom_eol(i: Span<'_>) -> Span<'_> {
    let (rest, _) = take_till::<_, _, nom::error::Error<Span<'_>>>(|_: char| false)(i)
        .expect("strange thing #3");
    rest
}

pub fn nom_empty(i: Span<'_>) -> Span<'_> {
    i.take(0)
}

/// Eat whitespace
pub fn nom_ws(i: Span<'_>) -> CNomResult<'_> {
    i.split_at_position_complete(|item| {
        let c = item.as_char();
        !(c == ' ' || c == '\t')
    })
}

/// Eat whitespace
pub fn nom_ws1(i: Span<'_>) -> CParserResult<'_, Span<'_>> {
    take_while1::<_, _, CParserError<'_>>(|c: char| c == ' ' || c == '\t')(i).with_code(CWhitespace)
}

/// Eat whitespace
pub fn nom_ws_span(i: Span<'_>) -> Span<'_> {
    match i.split_at_position_complete::<_, nom::error::Error<Span<'_>>>(|item| {
        let c = item.as_char();
        !(c == ' ' || c == '\t')
    }) {
        Ok((rest, _)) => rest,
        Err(_) => i,
    }
}
