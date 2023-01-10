use chrono::NaiveDate;
use kparse::{Code, ParserError, ParserNomResult, ParserResult};
use nom::bytes::complete::{tag, take_till, take_till1, take_while1};
use nom::character::complete::digit1;
use nom::combinator::recognize;
use nom::sequence::terminated;
use nom::{AsChar, InputTakeAtPosition};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub use CCode::*;

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

pub type Span<'s> = kparse::Span<'s, CCode>;
pub type CParserError<'s> = ParserError<'s, CCode>;
pub type CParserResult<'s, O> = ParserResult<'s, CCode, (), (Span<'s>, O)>;
pub type CNomResult<'s> = ParserNomResult<'s, CCode, ()>;

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
    take_while1::<_, _, CParserError<'_>>(|c: char| c == ' ' || c == '\t')(i)
        .into_with_code(CWhitespace)
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
