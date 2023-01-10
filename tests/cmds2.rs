use chrono::{Local, NaiveDate};
use glob::PatternError;
use iparse::error::ParserError;
use iparse::span::span_union;
use iparse::{
    Code, ConfParser, IntoParserError, IntoParserResultAddCode, IntoParserResultAddSpan, Parser,
    ParserNomResult, ParserResult, Span, Tracer, TrackParseResult,
};
use nom::bytes::complete::{tag, take_till, take_till1, take_while1};
use nom::character::complete::{char as nchar, digit1};
use nom::combinator::recognize;
use nom::sequence::terminated;
use nom::InputTake;
use nom::{AsChar, InputTakeAtPosition};
use std::fmt::{Debug, Display, Formatter};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::{fs, io};
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
    const NOM_FAILURE: Self = CNomFailure;
    const PARSE_INCOMPLETE: Self = CParseIncomplete;
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

impl CCode {
    pub fn is_sample_token(self) -> bool {
        match self {
            CDatum => true,
            CDateDay => true,
            CDotDay => true,
            CDateMonth => true,
            CDotMonth => true,
            CDateYear => true,
            // CInteger => "",
            CNummer => true,
            CFileName => true,
            // CWhitespace => "required whitespace",
            _ => false,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            CNomError => "",
            CNomFailure => "",
            CParseIncomplete => "",
            CIgnore => "",

            CWhitespace => "",
            CCommand => "Befehl.",
            CCreate => "create",
            CDebug => "debug",
            CDiff => "diff",
            CEtik => "etik",
            CExport => "export",
            CGet => "get",
            CHelp => "help",
            CImport => "import",
            CNew => "new",
            CPrint => "print",
            CReport => "report",
            CSend => "send",
            CSet => "set",
            CTest => "test",

            CArtikel => "artikel",
            CBm => "bm",
            CBs => "bs",
            CElcom => "elcom",
            CFile => "file",
            CFink => "fink",
            CGastro => "gastro",
            CKassa => "kassa",
            CKunde => "kunde",
            CKundeArtikel => "kundeartikel",
            CKw => "kw",
            CLf => "lf",
            CMitarbeiter => "mitarbeiter",
            CMonth => "month",
            COff => "off",
            COn => "on",
            CPlan => "plan",
            CRe => "re",
            CReDatum => "re-datum",
            CRind => "rind",
            CStatistik => "statistik",
            CWeek => "week",
            CWix => "wix",
            CWixId => "wix-id",
            CYear => "year",
            CZeiterfassung => "zeiterfassung",

            CDatum => "",
            CDateDay => "",
            CDotDay => "",
            CDateMonth => "",
            CDotMonth => "",
            CDateYear => "",
            CInteger => "",
            CNummer => "",
            CString => "",
            CFileName => "",
        }
    }

    pub fn token(self) -> &'static str {
        match self {
            CNomError => "",
            CNomFailure => "",
            CParseIncomplete => "",
            CIgnore => "",

            CCommand => "",
            CCreate => "create",
            CDebug => "debug",
            CDiff => "diff",
            CEtik => "etik",
            CExport => "export",
            CGet => "get",
            CHelp => "help",
            CImport => "import",
            CNew => "new",
            CPrint => "print",
            CReport => "report",
            CSend => "send",
            CSet => "set",
            CTest => "test",

            CArtikel => "artikel",
            CBm => "bm",
            CBs => "bs",
            CElcom => "elcom",
            CFile => "file",
            CFink => "fink",
            CGastro => "gastro",
            CKassa => "kassa",
            CKunde => "kunde",
            CKundeArtikel => "kundeartikel",
            CKw => "kw",
            CLf => "lf",
            CMitarbeiter => "mitarbeiter",
            CMonth => "month",
            COff => "off",
            COn => "on",
            CPlan => "plan",
            CRe => "re",
            CReDatum => "re-datum",
            CRind => "rind",
            CStatistik => "statistik",
            CWeek => "week",
            CWix => "wix",
            CWixId => "wix-id",
            CYear => "year",
            CZeiterfassung => "zeiterfassung",

            CDatum => " x: ungültiges Datum ",
            CDateDay => "dd.mm.jjjj",
            CDotDay => ".mm.jjjj",
            CDateMonth => "mm.jjjj",
            CDotMonth => ".jjjj",
            CDateYear => "jjjj",
            CInteger => " x: Zahl",
            CNummer => "###",
            CWhitespace => "",
            CString => "",
            CFileName => "",
        }
    }
}

pub type CParserError<'s> = ParserError<'s, CCode>;
pub type CParserResult<'s, O> = ParserResult<'s, CCode, (Span<'s>, O)>;
pub type CNomResult<'s> = ParserNomResult<'s, CCode>;

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

// Parser ----------------------------------------------------------------

pub struct ParseCmds;

impl<'s> Parser<'s, BCommand, CCode> for ParseCmds {
    fn id() -> CCode {
        CCommand
    }

    fn parse<'t>(
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, BCommand)> {
        trace.enter(Self::id(), rest);

        let mut command = None;
        if PARSE_CREATE.lah(rest) {
            match PARSE_CREATE.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if PARSE_PRINT.lah(rest) {
            match PARSE_PRINT.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if PARSE_IMPORT.lah(rest) {
            match PARSE_IMPORT.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if PARSE_EXPORT.lah(rest) {
            match PARSE_EXPORT.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if ParseNew::lah(rest) {
            match ParseNew::parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if ParseSet::lah(rest) {
            match ParseSet::parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if PARSE_GET.lah(rest) {
            match PARSE_GET.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if PARSE_DIFF.lah(rest) {
            match PARSE_DIFF.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if ParseEtik::lah(rest) {
            match ParseEtik::parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if ParseReport::lah(rest) {
            match ParseReport::parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if PARSE_SENDMAIL.lah(rest) {
            match PARSE_SENDMAIL.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if PARSE_HELP_1.lah(rest) {
            match PARSE_HELP_1.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if PARSE_HELP_2.lah(rest) {
            match PARSE_HELP_2.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if PARSE_TEST.lah(rest) {
            match PARSE_TEST.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }
        if PARSE_DEBUG.lah(rest) {
            match PARSE_DEBUG.parse(trace, rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => trace.stash(e),
            }
        }

        if let Some(command) = command {
            trace.ok(rest, nom_empty(rest), command)
        } else {
            let rest = nom_ws_span(rest);
            if !rest.is_empty() {
                trace.err(ParserError::new(CCommand, rest))
            } else {
                trace.ok(rest, nom_empty(rest), BCommand::None())
            }
        }
    }
}

const PARSE_CREATE: Parse2LayerCommand<Create, 2> = Parse2LayerCommand {
    map_cmd: BCommand::Create,
    layers: Parse2Layers {
        token: "create",
        code: CCreate,
        list: [
            SubCmd {
                token: "kundeartikel",
                code: CKundeArtikel,
                output: Create::KundeArt,
            },
            SubCmd {
                token: "wix-id",
                code: CWixId,
                output: Create::WixId,
            },
        ],
    },
};

const PARSE_PRINT: Parse2LayerCommand<Print, 11> = Parse2LayerCommand {
    map_cmd: BCommand::Print,
    layers: Parse2Layers {
        token: "print",
        code: CPrint,
        list: [
            SubCmd {
                token: "artikel",
                code: CArtikel,
                output: Print::Artikel,
            },
            SubCmd {
                token: "elcom",
                code: CElcom,
                output: Print::Elcom,
            },
            SubCmd {
                token: "kundeartikel",
                code: CKundeArtikel,
                output: Print::KundeArtikel,
            },
            SubCmd {
                token: "kunde",
                code: CKunde,
                output: Print::Kunde,
            },
            SubCmd {
                token: "kw",
                code: CKw,
                output: Print::Kw,
            },
            SubCmd {
                token: "month",
                code: CMonth,
                output: Print::Month,
            },
            SubCmd {
                token: "year",
                code: CYear,
                output: Print::Year,
            },
            SubCmd {
                token: "week",
                code: CWeek,
                output: Print::Week,
            },
            SubCmd {
                token: "bs",
                code: CBs,
                output: Print::Bs,
            },
            SubCmd {
                token: "lf",
                code: CLf,
                output: Print::Lf,
            },
            SubCmd {
                token: "re",
                code: CRe,
                output: Print::Re,
            },
        ],
    },
};

const PARSE_IMPORT: Parse2LayerCommand<Import, 2> = Parse2LayerCommand {
    map_cmd: BCommand::Import,
    layers: Parse2Layers {
        token: "import",
        code: CImport,
        list: [
            SubCmd {
                token: "elcom",
                code: CElcom,
                output: Import::Elcom,
            },
            SubCmd {
                token: "wix",
                code: CWix,
                output: Import::Wix,
            },
        ],
    },
};

const PARSE_EXPORT: Parse2LayerCommand<Export, 3> = Parse2LayerCommand {
    map_cmd: BCommand::Export,
    layers: Parse2Layers {
        token: "export",
        code: CExport,
        list: [
            SubCmd {
                token: "elcom",
                code: CElcom,
                output: Export::Elcom,
            },
            SubCmd {
                token: "plan",
                code: CPlan,
                output: Export::Plan,
            },
            SubCmd {
                token: "wix",
                code: CWix,
                output: Export::Wix,
            },
        ],
    },
};

pub struct ParseSet;

impl<'s> Parser<'s, BCommand, CCode> for ParseSet {
    fn id() -> CCode {
        CSet
    }

    fn lah(span: Span<'s>) -> bool {
        lah_command("set", span)
    }

    fn parse<'t>(
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, BCommand)> {
        trace.enter(Self::id(), rest);

        let (rest, (span_sub, sub_cmd)) = Parse2Layers {
            token: "set",
            code: CSet,
            list: [SubCmd {
                token: "re-datum",
                code: CReDatum,
                output: Set::ReDatum(Local::now().date_naive()),
            }],
        }
        .parse(trace, rest)
        .track(trace)?;

        let (rest, span_value, sub_cmd) = if !rest.is_empty() {
            let (rest, _) = nom_ws1(rest).track(trace)?;

            match sub_cmd {
                Set::ReDatum(_) => match token_datum(rest) {
                    Ok((rest, datum)) => (rest, datum.span, Set::ReDatum(datum.datum)),
                    Err(e) => {
                        trace.expect(CReDatum, rest);
                        return trace.err(e);
                    }
                },
            }
        } else {
            match sub_cmd {
                Set::ReDatum(_) => (rest, nom_empty(rest), sub_cmd),
            }
        };

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return trace.err(ParserError::new(CSet, rest));
        }

        let span = span_union(span_sub, span_value);

        trace.ok(rest, span, BCommand::Set(sub_cmd))
    }
}

pub struct ParseNew;

impl<'s> Parser<'s, BCommand, CCode> for ParseNew {
    fn id() -> CCode {
        CEtik
    }

    fn lah(span: Span<'s>) -> bool {
        lah_command("new", span)
    }

    fn parse<'t>(
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, BCommand)> {
        trace.enter(Self::id(), rest);

        let (rest, (span_sub, sub)) = Parse2Layers {
            token: "new",
            code: CNew,
            list: [
                ("bs", CBs, New::Bs(None)).into(),
                ("lf", CLf, New::Lf(None)).into(),
                ("re", CRe, New::Re(None)).into(),
            ],
        }
        .parse(trace, rest)
        .track(trace)?;

        let (rest, nummer) = if !rest.is_empty() {
            let (rest, _) = nom_ws1(rest).track(trace)?;

            match token_nummer(rest) {
                Ok((rest, nummer)) => (rest, Some(nummer)),
                Err(e) => {
                    trace.expect(CKunde, rest);
                    return trace.err(e);
                }
            }
        } else {
            (rest, None)
        };

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return trace.err(ParserError::new(CNew, rest));
        }

        let span = if let Some(nummer) = nummer {
            span_union(span_sub, nummer.span)
        } else {
            span_sub
        };

        match sub {
            New::Bs(_) => trace.ok(rest, span, BCommand::New(New::Bs(nummer.map(|v| v.nummer)))),
            New::Lf(_) => trace.ok(rest, span, BCommand::New(New::Lf(nummer.map(|v| v.nummer)))),
            New::Re(_) => trace.ok(rest, span, BCommand::New(New::Re(nummer.map(|v| v.nummer)))),
        }
    }
}

pub const PARSE_GET: Parse2LayerCommand<Get, 1> = Parse2LayerCommand {
    layers: Parse2Layers {
        token: "get",
        code: CGet,
        list: [SubCmd {
            token: "re-datum",
            code: CReDatum,
            output: Get::ReDatum,
        }],
    },
    map_cmd: BCommand::Get,
};

pub const PARSE_DIFF: Parse2LayerCommand<Diff, 1> = Parse2LayerCommand {
    layers: Parse2Layers {
        token: "diff",
        code: CDiff,
        list: [SubCmd {
            token: "elcom",
            code: CElcom,
            output: Diff::Elcom,
        }],
    },
    map_cmd: BCommand::Diff,
};

pub struct ParseEtik;

impl<'s> Parser<'s, BCommand, CCode> for ParseEtik {
    fn id() -> CCode {
        CEtik
    }

    fn lah(span: Span<'s>) -> bool {
        lah_command("etik", span)
    }

    fn parse<'t>(
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, BCommand)> {
        trace.enter(Self::id(), rest);

        let (rest, (span_sub, sub)) = Parse2Layers {
            token: "etik",
            code: CEtik,
            list: [
                ("bs", CBs, Etik::EtikBs(None)).into(),
                ("file", CFile, Etik::EtikFile).into(),
            ],
        }
        .parse(trace, rest)
        .track(trace)?;

        let (rest, nummer) = if let Etik::EtikBs(_) = sub {
            if !rest.is_empty() {
                let (rest, _) = nom_ws1(rest).track(trace)?;

                match token_nummer(rest) {
                    Ok((rest, nummer)) => (rest, Some(nummer)),
                    Err(e) => {
                        trace.expect(CBs, rest);
                        return trace.err(e);
                    }
                }
            } else {
                (rest, None)
            }
        } else {
            (rest, None)
        };

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return trace.err(ParserError::new(CEtik, rest));
        }

        let span = if let Some(nummer) = nummer {
            span_union(span_sub, nummer.span)
        } else {
            span_sub
        };

        match sub {
            Etik::EtikBs(_) => trace.ok(
                rest,
                span,
                BCommand::Etik(Etik::EtikBs(nummer.map(|v| v.nummer))),
            ),
            Etik::EtikFile => trace.ok(rest, span, BCommand::Etik(Etik::EtikFile)),
        }
    }
}

pub struct ParseReport;

impl<'s> Parser<'s, BCommand, CCode> for ParseReport {
    fn id() -> CCode {
        CReport
    }

    fn lah(span: Span<'s>) -> bool {
        lah_command("report", span)
    }

    fn parse<'t>(
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, BCommand)> {
        trace.enter(Self::id(), rest);

        let (rest, (span_sub, sub)) = Parse2Layers {
            code: CReport,
            token: "report",
            list: [
                ("mitarbeiter", CMitarbeiter, Report::Mitarbeiter(None)).into(),
                ("zeiterfassung", CZeiterfassung, Report::Zeiterfassung(None)).into(),
                ("rind", CRind, Report::Rind).into(),
                ("fink", CFink, Report::Fink).into(),
                ("bm", CBm, Report::Bm).into(),
                ("gastro", CGastro, Report::Gastro).into(),
                ("statistik", CStatistik, Report::Statistik).into(),
            ],
        }
        .parse(trace, rest)
        .track(trace)?;

        let (rest, span_datum, sub) = if !rest.is_empty() {
            let (rest, _) = nom_ws1(rest).track(trace)?;

            match sub {
                Report::Mitarbeiter(_) => match token_datum(rest) {
                    Ok((rest, datum)) => (rest, datum.span, Report::Mitarbeiter(Some(datum.datum))),
                    Err(e) => return trace.err(e),
                },
                Report::Zeiterfassung(_) => match token_datum(rest) {
                    Ok((rest, datum)) => {
                        (rest, datum.span, Report::Zeiterfassung(Some(datum.datum)))
                    }
                    Err(e) => return trace.err(e),
                },
                _ => return trace.err(ParserError::new(CReport, rest)),
            }
        } else {
            (rest, nom_empty(rest), sub)
        };

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return trace.err(ParserError::new(CReport, rest));
        }

        let span = span_union(span_sub, span_datum);
        trace.ok(rest, span, BCommand::Report(sub))
    }
}

pub const PARSE_SENDMAIL: Parse2LayerCommand<SendMail, 1> = Parse2LayerCommand {
    layers: Parse2Layers {
        token: "send",
        code: CSend,
        list: [SubCmd {
            token: "gastro",
            code: CGastro,
            output: SendMail::Gastro,
        }],
    },
    map_cmd: BCommand::SendMail,
};

const PARSE_HELP_1: Parse1LayerCommand = Parse1LayerCommand {
    cmd: BCommand::Help(Help::Help),
    layers: Parse1Layers {
        token: "help",
        code: CHelp,
    },
};

const PARSE_HELP_2: Parse1LayerCommand = Parse1LayerCommand {
    cmd: BCommand::Help(Help::Help),
    layers: Parse1Layers {
        token: "?",
        code: CHelp,
    },
};

const PARSE_TEST: Parse2LayerCommand<Test, 1> = Parse2LayerCommand {
    map_cmd: BCommand::Test,
    layers: Parse2Layers {
        token: "test",
        code: CTest,
        list: [SubCmd {
            token: "test",
            code: CTest,
            output: Test::Test,
        }],
    },
};

pub const PARSE_DEBUG: Parse2LayerCommand<Debugging, 2> = Parse2LayerCommand {
    layers: Parse2Layers {
        token: "debug",
        code: CDebug,
        list: [
            SubCmd {
                token: "on",
                code: COn,
                output: Debugging::On,
            },
            SubCmd {
                token: "off",
                code: COff,
                output: Debugging::Off,
            },
        ],
    },
    map_cmd: BCommand::Debug,
};

pub struct ParseNummer;

impl<'s> Parser<'s, Nummer<'s>, CCode> for ParseNummer {
    fn id() -> CCode {
        CNummer
    }

    fn parse<'t>(
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, Nummer<'s>)> {
        trace.enter(Self::id(), rest);

        let (rest, nummer) = token_nummer(rest).track(trace)?;
        trace.ok(rest, nummer.span, nummer)
    }
}

pub struct ParseDatum;

impl<'s> Parser<'s, Datum<'s>, CCode> for ParseDatum {
    fn id() -> CCode {
        CDatum
    }

    fn parse<'t>(
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, Datum<'s>)> {
        trace.enter(Self::id(), rest);

        let (rest, datum) = token_datum(rest).track(trace)?;
        trace.ok(rest, datum.span, datum)
    }
}

pub struct ParseFile<'a> {
    pub path: &'a Path,
    pub pattern: &'a str,
}

#[derive(Debug)]
pub enum ParseFileError<'s> {
    PossibleMatches(Span<'s>, Vec<String>),
    DoesNotExist(Span<'s>),
    Pattern(PatternError),
    IOError(io::Error),
}

impl<'s> From<io::Error> for ParseFileError<'s> {
    fn from(err: io::Error) -> Self {
        ParseFileError::IOError(err)
    }
}

impl<'s> From<PatternError> for ParseFileError<'s> {
    fn from(err: PatternError) -> Self {
        ParseFileError::Pattern(err)
    }
}

impl<'a, 's> ParseFile<'a> {
    pub fn parse(&self, rest: Span<'s>) -> Result<Datei<'s>, ParseFileError<'s>> {
        let pattern = glob::Pattern::new(self.pattern)?;

        // not a valid name according to the pattern. return possible matches.
        if !pattern.matches(rest.as_ref()) {
            let mut matches = Vec::new();
            // find matches via input prefix and pattern.
            for r in fs::read_dir(self.path)? {
                match r {
                    Ok(r) => {
                        let file_name = r.file_name().to_string_lossy().to_string();
                        if pattern.matches(&file_name) && file_name.starts_with(*rest) {
                            matches.push(file_name);
                        }
                    }
                    Err(e) => return Err(ParseFileError::IOError(e)),
                }
            }

            return Err(ParseFileError::PossibleMatches(rest, matches));
        }

        // would be matching but doesn't exist.
        let p = self.path.join(*rest);
        if !p.exists() {
            Err(ParseFileError::DoesNotExist(rest))
        } else {
            Ok(Datei {
                path: p,
                span: rest,
            })
        }
    }
}

// Generic parsers -------------------------------------------------------

pub struct Parse1LayerCommand {
    cmd: BCommand,
    layers: Parse1Layers,
}

impl<'s> ConfParser<'s, BCommand, CCode> for Parse1LayerCommand {
    fn id(&self) -> CCode {
        self.layers.code
    }

    fn lah(&self, span: Span<'s>) -> bool {
        lah_command(self.layers.token, span)
    }

    fn parse<'t>(
        &self,
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, BCommand)> {
        trace.enter(self.id(), rest);

        let (rest, sub) = self.layers.parse(trace, rest).track(trace)?;

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return trace.err(ParserError::new(self.id(), rest));
        }

        trace.ok(rest, sub, self.cmd)
    }
}

pub struct Parse1Layers {
    pub token: &'static str,
    pub code: CCode,
}

impl<'s> ConfParser<'s, Span<'s>, CCode> for Parse1Layers {
    fn id(&self) -> CCode {
        self.code
    }

    fn parse<'t>(
        &self,
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, Span<'s>)> {
        trace.enter(self.id(), rest);

        let (rest, token) = token_command(self.token, self.code, rest).track(trace)?;

        trace.ok(rest, token, token)
    }
}

pub struct Parse2LayerCommand<O: Copy + Debug, const N: usize> {
    map_cmd: fn(O) -> BCommand,
    layers: Parse2Layers<O, N>,
}

impl<'s, O: Copy + Debug, const N: usize> ConfParser<'s, BCommand, CCode>
    for Parse2LayerCommand<O, N>
{
    fn id(&self) -> CCode {
        self.layers.code
    }

    fn lah(&self, span: Span<'s>) -> bool {
        lah_command(self.layers.token, span)
    }

    fn parse<'t>(
        &self,
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, BCommand)> {
        trace.enter(self.id(), rest);

        let (rest, (span, sub)) = self.layers.parse(trace, rest).track(trace)?;

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return trace.err(ParserError::new(self.id(), rest));
        }

        trace.ok(rest, span, (self.map_cmd)(sub))
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

impl<'s, O: Copy + Debug, const N: usize> ConfParser<'s, (Span<'s>, O), CCode>
    for Parse2Layers<O, N>
{
    fn id(&self) -> CCode {
        self.code
    }

    fn parse<'t>(
        &self,
        trace: &'t mut impl Tracer<'s, CCode>,
        rest: Span<'s>,
    ) -> ParserResult<'s, CCode, (Span<'s>, (Span<'s>, O))> {
        trace.enter(self.id(), rest);

        let (rest, token) = token_command(self.token, self.code, rest).track(trace)?;
        trace.debug(format!("found {}", token));

        let (rest, _) = nom_ws1(rest).track(trace)?;

        let (rest, span_sub, sub) = 'for_else: {
            let mut err = None;
            for sub in &self.list {
                match token_command(sub.token, sub.code, rest) {
                    Ok((rest, span)) => {
                        // println!("found");
                        break 'for_else (rest, span, sub);
                    }
                    Err(e) => {
                        if e.code != CIgnore {
                            if let Some(err) = err {
                                trace.stash(err);
                            }
                            err = Some(e);
                        }
                    }
                }
            }
            return match err {
                Some(err) => trace.err(err),
                None => {
                    for sub in &self.list {
                        trace.suggest(sub.code, rest);
                    }
                    trace.err(ParserError::new(self.code, rest))
                }
            };
        };

        let span = span_union(token, span_sub);
        trace.ok(rest, span, (span_sub, sub.output))
    }
}

// Tokens ----------------------------------------------------------------
impl<'s, T> IntoParserResultAddSpan<'s, CCode, T> for Result<T, ParseIntError> {
    fn into_with_span(self, span: Span<'s>) -> ParserResult<'s, CCode, T> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(ParserError::new(CInteger, span)),
        }
    }
}

pub fn token_nummer(rest: Span<'_>) -> CParserResult<'_, Nummer<'_>> {
    match nom_number(rest) {
        Ok((rest, tok)) => Ok((
            rest,
            Nummer {
                nummer: tok.parse::<u32>().into_with_span(rest)?,
                span: tok,
            },
        )),
        Err(e) => Err(e.into_with_code(CNummer)),
    }
}

pub fn token_datum(rest: Span<'_>) -> CParserResult<'_, Datum<'_>> {
    let (rest, day) = nom_number(rest).into_with_code(CDateDay)?;
    let (rest, _) = nom_dot(rest).into_with_code(CDotDay)?;
    let (rest, month) = nom_number(rest).into_with_code(CDateMonth)?;
    let (rest, _) = nom_dot(rest).into_with_code(CDotMonth)?;
    let (rest, year) = nom_number(rest).into_with_code(CDateYear)?;

    let iday: u32 = (*day).parse().into_with_span(day)?;
    let imonth: u32 = (*month).parse().into_with_span(month)?;
    let iyear: i32 = (*year).parse().into_with_span(year)?;

    let span = span_union(day, year);
    let datum = NaiveDate::from_ymd_opt(iyear, imonth, iday);

    if let Some(datum) = datum {
        Ok((rest, Datum { datum, span }))
    } else {
        Err(ParserError::new(CDatum, span))
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
                    return Err(err);
                }
                Err(_) => return Err(CParserError::new(CIgnore, rest)),
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

#[cfg(test)]
mod tests {
    use crate::cmds2::{token_command, BCommand, CCode, Datum, Nummer, ParseReport, Report};
    use iparse::test::{test_parse, Trace};
    use iparse::{Parser, Span};
    use std::mem::size_of;

    #[test]
    fn test_token() {
        println!(
            "{:?}",
            token_command("report", CCode::CReport, Span::new(""))
        );
        println!(
            "{:?}",
            token_command("report", CCode::CReport, Span::new("rep"))
        );
        println!(
            "{:?}",
            token_command("report", CCode::CReport, Span::new("report"))
        );
        println!(
            "{:?}",
            token_command("report", CCode::CReport, Span::new("report this"))
        );
    }

    #[test]
    fn test_report() {
        const R: Trace = Trace;

        test_parse("rep", ParseReport::parse).okok().q(&R);
        test_parse("report ", ParseReport::parse).okok().q(&R);
        test_parse("report z", ParseReport::parse).okok().q(&R);
        test_parse("report zeiterfassung ", ParseReport::parse)
            .okok()
            .q(&R);
        test_parse("report zeiterfassung 1", ParseReport::parse)
            .okok()
            .q(&R);
        test_parse("report zeiterfassung 1.", ParseReport::parse)
            .okok()
            .q(&R);
    }

    #[test]
    fn test_size() {
        println!("Command {}", size_of::<BCommand>());
        println!("Report {}", size_of::<Report>());
        println!("Datum {}", size_of::<Datum>());
        println!("Nummer {}", size_of::<Nummer>());
        println!("Span {}", size_of::<Span<'_>>());
    }
}
