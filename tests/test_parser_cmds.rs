pub use cmds_parser::*;
use kparse::test::{notrack_parse, CheckDump};
use std::time::Instant;

#[test]
fn test_1() {
    let tests = [
        "#V2",
        "?",
        "print bs",
        "print lf",
        "print bs",
        "print re",
        "?",
        "report mitarbeiter 1.1.2023",
        "report zeiterfassung 1.1.2023",
        "print kw",
        "print mon",
        "print month",
        "print year",
        "new lf 71",
        "new lf 124",
        "new lf 24",
        "new lf 71",
        "new lf 46",
        "new lf",
        "new lf ",
        "load kunde",
        "print kunde ",
        "new lf 54",
        "new lf 77",
        "new lf ",
        "new lf 71",
        "new lf 3",
        "new lf ",
        "new lf 71",
        "new lf 77",
        "print lf",
        "set re-datum 31.12.2022",
        "new re 3",
        "print lf",
        "new re 24",
        "print lf",
        "new re 77",
        "print lf",
        "new re 46",
        "print lf",
        "new re 54",
        "new re 71",
        "print lf",
        "print bs",
        "print re",
        "?",
        "report fink",
        "?",
        "report fink",
        "print artikel",
        "?",
        "print kunde",
        "print el",
        "print elcom",
        "?",
        "diff el",
        "diff elcom",
        "?",
        "print kw",
        "print week",
        "print month",
        "print year ",
        "?",
        "report rind",
        "report bm",
        "report gastro",
        "?",
        "report fink",
        "?",
        "report bm",
        "report gastro",
        "?",
        "print artikel",
        "`??",
        "?",
        "export bm",
        "report bm",
        "new lf",
        "report bm",
        "report gastro",
        "?",
        "send gastro",
        "?",
        "report fink",
        "test 1",
        "?",
        "test 1",
        "?",
        "report statistik",
        "?",
        "reü",
        "report statistik",
        "?",
        "report statistik",
        "    ",
        "report statistik",
        "?",
        "report statistik",
        "?",
        "cls",
        "asdf",
    ];

    let r = CheckDump;

    let now = Instant::now();
    for t in tests {
        for _i in 1..100 {
            notrack_parse(&mut None, t, parse_cmds).q(r);
        }
    }
    let elapsed = now.elapsed();
    println!(
        "{} tests in {}",
        tests.len(),
        humantime::format_duration(elapsed / tests.len() as u32 / 100u32)
    );
}

mod cmds_parser {
    use chrono::{Local, NaiveDate};
    use glob::PatternError;
    use kparse::prelude::*;
    use nom::bytes::complete::{tag, take_till1, take_while1};
    use nom::character::complete::{char as nchar, digit1};
    use nom::combinator::{consumed, recognize};
    use nom::error::ParseError;
    use nom::sequence::{terminated, tuple};
    use nom::{AsChar, InputTake, InputTakeAtPosition};
    use std::fmt::{Debug, Display, Formatter};
    use std::num::ParseIntError;
    use std::path::{Path, PathBuf};
    use std::{fs, io};

    use kparse::spans::SpanExt;
    use kparse::{error_code, transform};
    use CCode::*;

    pub type Span<'s> = kparse::Span<'s, &'s str, CCode>;
    pub type CParserError<'s> = ParserError<'s, &'s str, CCode>;
    pub type CParserResult<'s, O> = ParserResult<'s, O, &'s str, CCode, ()>;
    pub type CNomResult<'s> = ParserNomResult<'s, &'s str, CCode, ()>;

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

    pub fn parse_cmds(rest: Span<'_>) -> CParserResult<'_, BCommand> {
        Context.enter(CCommand, &rest);

        let mut command = None;
        let mut err = None;
        if PARSE_CREATE.lah(rest) {
            match PARSE_CREATE.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if PARSE_PRINT.lah(rest) {
            match PARSE_PRINT.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if PARSE_IMPORT.lah(rest) {
            match PARSE_IMPORT.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if PARSE_EXPORT.lah(rest) {
            match PARSE_EXPORT.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if lah_new(rest) {
            match parse_new(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if lah_set(rest) {
            match parse_set(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if PARSE_GET.lah(rest) {
            match PARSE_GET.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if PARSE_DIFF.lah(rest) {
            match PARSE_DIFF.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if lah_etik(rest) {
            match parse_etik(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if lah_report(rest) {
            match parse_report(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if PARSE_SENDMAIL.lah(rest) {
            match PARSE_SENDMAIL.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if PARSE_HELP_1.lah(rest) {
            match PARSE_HELP_1.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if PARSE_HELP_2.lah(rest) {
            match PARSE_HELP_2.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if PARSE_TEST.lah(rest) {
            match PARSE_TEST.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }
        if PARSE_DEBUG.lah(rest) {
            match PARSE_DEBUG.parse(rest) {
                Ok((_, cmd)) => command = Some(cmd),
                Err(e) => err.append(e)?,
            }
        }

        if let Some(command) = command {
            Context.ok(rest, nom_empty(rest), command)
        } else {
            let rest = nom_ws_span(rest);
            if !rest.is_empty() {
                Context.err(ParserError::new(CCommand, rest))
            } else {
                Context.ok(rest, nom_empty(rest), BCommand::None())
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

    fn lah_set(span: Span<'_>) -> bool {
        lah_command("set", span)
    }

    fn parse_set(input: Span<'_>) -> CParserResult<'_, BCommand> {
        Context.enter(CSet, &input);

        let (rest, (span_sub, sub_cmd)) = Parse2Layers {
            token: "set",
            code: CSet,
            list: [SubCmd {
                token: "re-datum",
                code: CReDatum,
                output: Set::ReDatum(Local::now().date_naive()),
            }],
        }
        .parse(input)
        .track()?;

        let (rest, span_value, sub_cmd) = if !rest.is_empty() {
            let (rest, _) = nom_ws1(rest).track()?;

            match sub_cmd {
                Set::ReDatum(_) => match token_datum(rest) {
                    Ok((rest, datum)) => (rest, datum.span, Set::ReDatum(datum.datum)),
                    Err(nom::Err::Error(mut e)) => {
                        e.expect(CReDatum, rest);
                        return Context.err(e);
                    }
                    Err(e) => return Context.err(e),
                },
            }
        } else {
            match sub_cmd {
                Set::ReDatum(_) => (rest, nom_empty(rest), sub_cmd),
            }
        };

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return Context.err(ParserError::new(CSet, rest));
        }

        let span = input.span_union(&span_sub, &span_value);

        Context.ok(rest, span, BCommand::Set(sub_cmd))
    }

    fn lah_new(span: Span<'_>) -> bool {
        lah_command("new", span)
    }

    fn parse_new(input: Span<'_>) -> CParserResult<'_, BCommand> {
        Context.enter(CEtik, &input);

        let (rest, (span_sub, sub)) = Parse2Layers {
            token: "new",
            code: CNew,
            list: [
                ("bs", CBs, New::Bs(None)).into(),
                ("lf", CLf, New::Lf(None)).into(),
                ("re", CRe, New::Re(None)).into(),
            ],
        }
        .parse(input)
        .track()?;

        let (rest, nummer) = if !rest.is_empty() {
            let (rest, _) = nom_ws1(rest).track()?;

            match token_nummer(rest) {
                Ok((rest, nummer)) => (rest, Some(nummer)),
                Err(nom::Err::Error(mut e)) => {
                    e.expect(CKunde, rest);
                    return Context.err(e);
                }
                Err(e) => return Context.err(e),
            }
        } else {
            (rest, None)
        };

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return Context.err(ParserError::new(CNew, rest));
        }

        let span = if let Some(nummer) = nummer {
            input.span_union(&span_sub, &nummer.span)
        } else {
            span_sub
        };

        match sub {
            New::Bs(_) => Context.ok(rest, span, BCommand::New(New::Bs(nummer.map(|v| v.nummer)))),
            New::Lf(_) => Context.ok(rest, span, BCommand::New(New::Lf(nummer.map(|v| v.nummer)))),
            New::Re(_) => Context.ok(rest, span, BCommand::New(New::Re(nummer.map(|v| v.nummer)))),
        }
    }

    const PARSE_GET: Parse2LayerCommand<Get, 1> = Parse2LayerCommand {
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

    const PARSE_DIFF: Parse2LayerCommand<Diff, 1> = Parse2LayerCommand {
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

    fn lah_etik(span: Span<'_>) -> bool {
        lah_command("etik", span)
    }

    fn parse_etik(input: Span<'_>) -> CParserResult<'_, BCommand> {
        Context.enter(CEtik, &input);

        let (rest, (span_sub, sub)) = Parse2Layers {
            token: "etik",
            code: CEtik,
            list: [
                ("bs", CBs, Etik::EtikBs(None)).into(),
                ("file", CFile, Etik::EtikFile).into(),
            ],
        }
        .parse(input)
        .track()?;

        let (rest, nummer) = if let Etik::EtikBs(_) = sub {
            if !rest.is_empty() {
                let (rest, _) = nom_ws1(rest).track()?;

                match token_nummer(rest) {
                    Ok((rest, nummer)) => (rest, Some(nummer)),
                    Err(nom::Err::Error(mut e)) => {
                        e.expect(CBs, rest);
                        return Context.err(e);
                    }
                    Err(e) => return Context.err(e),
                }
            } else {
                (rest, None)
            }
        } else {
            (rest, None)
        };

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return Context.err(ParserError::new(CEtik, rest));
        }

        let span = if let Some(nummer) = nummer {
            input.span_union(&span_sub, &nummer.span)
        } else {
            span_sub
        };

        match sub {
            Etik::EtikBs(_) => Context.ok(
                rest,
                span,
                BCommand::Etik(Etik::EtikBs(nummer.map(|v| v.nummer))),
            ),
            Etik::EtikFile => Context.ok(rest, span, BCommand::Etik(Etik::EtikFile)),
        }
    }

    fn lah_report(span: Span<'_>) -> bool {
        lah_command("report", span)
    }

    fn parse_report(input: Span<'_>) -> CParserResult<'_, BCommand> {
        Context.enter(CReport, &input);

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
        .parse(input)
        .track()?;

        let (rest, span_datum, sub) = if !rest.is_empty() {
            let (rest, _) = nom_ws1(rest).track()?;

            match sub {
                Report::Mitarbeiter(_) => match token_datum(rest) {
                    Ok((rest, datum)) => (rest, datum.span, Report::Mitarbeiter(Some(datum.datum))),
                    Err(e) => return Context.err(e),
                },
                Report::Zeiterfassung(_) => match token_datum(rest) {
                    Ok((rest, datum)) => {
                        (rest, datum.span, Report::Zeiterfassung(Some(datum.datum)))
                    }
                    Err(e) => return Context.err(e),
                },
                _ => return Context.err(ParserError::new(CReport, rest)),
            }
        } else {
            (rest, nom_empty(rest), sub)
        };

        let rest = nom_ws_span(rest);

        if !rest.is_empty() {
            return Context.err(ParserError::new(CReport, rest));
        }

        let span = input.span_union(&span_sub, &span_datum);
        Context.ok(rest, span, BCommand::Report(sub))
    }

    const PARSE_SENDMAIL: Parse2LayerCommand<SendMail, 1> = Parse2LayerCommand {
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

    const PARSE_DEBUG: Parse2LayerCommand<Debugging, 2> = Parse2LayerCommand {
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

    pub fn parse_nummer(rest: Span<'_>) -> CParserResult<'_, Nummer<'_>> {
        Context.enter(CNummer, &rest);

        let (rest, nummer) = token_nummer(rest).track()?;

        Context.ok(rest, nummer.span, nummer)
    }

    pub fn parse_datum(rest: Span<'_>) -> CParserResult<'_, Datum<'_>> {
        Context.enter(CDatum, &rest);

        let (rest, datum) = token_datum(rest).track()?;
        Context.ok(rest, datum.span, datum)
    }

    // ParseFile -------------------------------------------------------------

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

    struct Parse1LayerCommand {
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
            Context.enter(self.id(), &rest);

            let (rest, sub) = self.layers.parse(rest).track()?;

            let rest = nom_ws_span(rest);

            if !rest.is_empty() {
                return Context.err(ParserError::new(self.id(), rest));
            }

            Context.ok(rest, sub, self.cmd)
        }
    }

    struct Parse1Layers {
        pub token: &'static str,
        pub code: CCode,
    }

    impl Parse1Layers {
        fn id(&self) -> CCode {
            self.code
        }

        fn parse<'s>(&self, rest: Span<'s>) -> CParserResult<'s, Span<'s>> {
            Context.enter(self.id(), &rest);

            let (rest, token) = token_command(self.token, self.code, rest).track()?;

            Context.ok(rest, token, token)
        }
    }

    struct Parse2LayerCommand<O: Copy + Debug, const N: usize> {
        map_cmd: fn(O) -> BCommand,
        layers: Parse2Layers<O, N>,
    }

    impl<O: Copy + Debug, const N: usize> Parse2LayerCommand<O, N> {
        fn lah(&self, span: Span<'_>) -> bool {
            lah_command(self.layers.token, span)
        }

        fn parse<'s>(&self, rest: Span<'s>) -> CParserResult<'s, BCommand> {
            Context.enter(self.layers.code, &rest);

            let (rest, (span, sub)) = self.layers.parse(rest).track()?;

            let rest = nom_ws_span(rest);

            if !rest.is_empty() {
                return Context.err(ParserError::new(self.layers.code, rest));
            }

            Context.ok(rest, span, (self.map_cmd)(sub))
        }
    }

    struct Parse2Layers<O: Copy, const N: usize> {
        pub token: &'static str,
        pub code: CCode,
        pub list: [SubCmd<O>; N],
    }

    struct SubCmd<O: Copy> {
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
        fn parse<'s>(&self, input: Span<'s>) -> CParserResult<'s, (Span<'s>, O)> {
            Context.enter(self.code, &input);

            let (rest, token) = token_command(self.token, self.code, input).track()?;
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
                            err.suggest(sub.code, rest);
                        }
                        Context.err(err)
                    }
                };
            };

            let span = input.span_union(&token, &span_sub);
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

    fn token_nummer(rest: Span<'_>) -> CParserResult<'_, Nummer<'_>> {
        transform(
            nom_number,
            |s| -> Result<Nummer<'_>, ParseIntError> {
                Ok(Nummer {
                    nummer: (*s).parse::<u32>()?,
                    span: s,
                })
            },
            CNummer,
        )(rest)

        // match nom_number(rest) {
        //     Ok((rest, tok)) => Ok((
        //         rest,
        //         Nummer {
        //             nummer: tok.parse::<u32>().with_span(CNummer, rest)?,
        //             span: tok,
        //         },
        //     )),
        //     Err(e) => Err(e.with_code(CNummer)),
        // }
    }

    fn token_datum(rest: Span<'_>) -> CParserResult<'_, Datum<'_>> {
        // let (rest, day) = nom_number(rest).with_code(CDateDay)?;
        // let (rest, _) = nom_dot(rest).with_code(CDotDay)?;
        // let (rest, month) = nom_number(rest).with_code(CDateMonth)?;
        // let (rest, _) = nom_dot(rest).with_code(CDotMonth)?;
        // let (rest, year) = nom_number(rest).with_code(CDateYear)?;
        //
        // let iday: u32 = (*day).parse().with_span(CDateDay, day)?;
        // let imonth: u32 = (*month).parse().with_span(CDateMonth, month)?;
        // let iyear: i32 = (*year).parse().with_span(CDateYear, year)?;

        let (rest, (span, (day, _, month, _, year))) = consumed(tuple((
            transform(nom_number, |s| (*s).parse::<u32>(), CDateDay),
            error_code(nom_dot, CDotDay),
            transform(nom_number, |s| (*s).parse::<u32>(), CDateMonth),
            error_code(nom_dot, CDotDay),
            transform(nom_number, |s| (*s).parse::<i32>(), CDateYear),
        )))(rest)?;

        let datum = NaiveDate::from_ymd_opt(year, month, day);

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
    fn nom_last_token(i: Span<'_>) -> CNomResult<'_> {
        match recognize::<_, _, CParserError<'_>, _>(take_till1(|c: char| c == ' ' || c == '\t'))(i)
        {
            Ok((rest, tok)) if rest.is_empty() => Ok((rest, tok)),
            _ => Err(nom::Err::Error(ParserError::new(CNomError, i))),
        }
    }

    fn nom_number(i: Span<'_>) -> CNomResult<'_> {
        terminated(digit1, nom_ws)(i)
    }

    fn nom_dot(i: Span<'_>) -> CNomResult<'_> {
        terminated(recognize(nchar('.')), nom_ws)(i)
    }

    // fn nom_eol(i: Span<'_>) -> Span<'_> {
    //     let (rest, _) = take_till::<_, _, nom::error::Error<Span<'_>>>(|_: char| false)(i)
    //         .expect("strange thing #3");
    //     rest
    // }

    fn nom_empty(i: Span<'_>) -> Span<'_> {
        i.take(0)
    }

    /// Eat whitespace
    fn nom_ws(i: Span<'_>) -> CNomResult<'_> {
        i.split_at_position_complete(|item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t')
        })
    }

    /// Eat whitespace
    fn nom_ws1(i: Span<'_>) -> CParserResult<'_, Span<'_>> {
        take_while1::<_, _, CParserError<'_>>(|c: char| c == ' ' || c == '\t')(i)
            .with_code(CWhitespace)
    }

    /// Eat whitespace
    fn nom_ws_span(i: Span<'_>) -> Span<'_> {
        match i.split_at_position_complete::<_, nom::error::Error<Span<'_>>>(|item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t')
        }) {
            Ok((rest, _)) => rest,
            Err(_) => i,
        }
    }
}
