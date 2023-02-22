use crate::parser4::ast::{APMenge, APName};
use crate::parser4::diagnostics::{dump_diagnostics, dump_trace};
use crate::parser4::nom_tokens::nom_metadata;
use crate::parser4::parser::*;
use crate::parser4::tokens::{token_datum, token_menge, token_name, token_name_kurz, token_nummer};
use crate::parser4::APCode::*;
use kparse::test::{span_parse, CheckDump, CheckTrace};
use kparse::tracker::StdTracker;
use nom_locate::LocatedSpan;
use std::fs::read_to_string;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

// const R: ReportDiagnostics = ReportDiagnostics;
const R: CheckTrace = CheckTrace;
const RT: CheckDump = CheckDump;

#[test]
pub fn timing() {
    let s = include_str!("2022_Anbauplan.txt");

    let now = Instant::now();
    let cnt = 100;
    for _i in 0..cnt {
        #[cfg(debug_assertions)]
        let context = StdTracker::new();
        #[cfg(debug_assertions)]
        let span = context.span(s);
        #[cfg(not(debug_assertions))]
        let span = LocatedSpan::new(s);

        let _r = black_box(parse_anbauplan(span));
    }
    let duration = now.elapsed();
    println!("{:?}", duration / cnt);

    // #[cfg(debug_assertions)]
    // let context = StdTracker::new();
    // #[cfg(debug_assertions)]
    // let span = context.span(s);
    // #[cfg(not(debug_assertions))]
    // let span = LocatedSpan::new(s);
    //
    // let ap = parse_anbauplan(span).unwrap();
    // dbg!(ap);
}

#[test]
pub fn full_plan() {
    let s = include_str!("2022_Anbauplan.txt");

    #[cfg(debug_assertions)]
    let context = StdTracker::new();
    #[cfg(debug_assertions)]
    let span = context.span(s);
    #[cfg(not(debug_assertions))]
    let span = LocatedSpan::new(s);

    match parse_anbauplan(span) {
        Ok((_r, v)) => {
            #[cfg(debug_assertions)]
            dump_trace(&context.results());
            dbg!(v);
        }
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            #[cfg(debug_assertions)]
            dump_trace(&context.results());
            dump_diagnostics(&PathBuf::from("2022_Anbauplan.txt"), span, &e, "", true);
            panic!();
        }
        Err(e) => {
            dbg!(e);
            panic!();
        }
    };
}

#[test]
pub fn som_plan() {
    for _ in 0..100 {
        for f in [
            "tests/data/2023_Anbauplan.txt",
            "tests/data/2023_Externe.txt",
            "tests/data/2023_Gesamt.txt",
            "tests/data/2023_Gleisdorf.txt",
            "tests/data/2023_Kräuter.txt",
            "tests/data/2023_Kunden.txt",
            "tests/data/2023_Lebenshilfe.txt",
            "tests/data/2023_Pflanzenmarkt_1.txt",
            "tests/data/2023_Pflanzenmarkt_2.txt",
        ] {
            println!("parse {}", f);
            let s = read_to_string(f).unwrap();

            #[cfg(debug_assertions)]
            let context = StdTracker::new();
            #[cfg(debug_assertions)]
            let span = context.span(s.as_str());
            #[cfg(not(debug_assertions))]
            let span = LocatedSpan::new(s.as_str());

            match parse_anbauplan(span) {
                Ok((_r, v)) => {
                    // #[cfg(debug_assertions)]
                    // dump_trace(&context.results());
                }
                Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                    #[cfg(debug_assertions)]
                    dump_trace(&context.results());
                    dump_diagnostics(&PathBuf::from("2022_Anbauplan.txt"), span, &e, "", true);
                    panic!();
                }
                Err(e) => {
                    panic!();
                }
            };
        }
    }
}

#[test]
pub fn test_metadata() {
    span_parse(&mut None, "Content-Type: text/x-zim-wiki\n", nom_metadata)
        .ok_any()
        .q(RT);
}

#[test]
pub fn test_pflanzort() {
    span_parse(&mut None, "@ N1/1  Salate 1W + 6W", parse_pflanzort)
        .ok_any()
        .q(R);
    span_parse(&mut None, "@ N1/1  1W + 6W", parse_pflanzort)
        .ok_any()
        .q(R);
}

#[test]
pub fn test_kunde() {
    span_parse(&mut None, "** Kunde Test Kunde **", parse_kunde)
        .ok_any()
        .q(R);
    span_parse(&mut None, " ** Kunde Test Kunde ** ", parse_kunde)
        .err_any()
        .q(R);
    span_parse(&mut None, "Kunde Test Kunde ", parse_kunde)
        .ok_any()
        .rest("")
        .q(R);
}

#[test]
pub fn test_markt() {
    span_parse(&mut None, "** Markt Graz **", parse_markt)
        .ok_any()
        .q(R);
    span_parse(&mut None, " ** Markt Graz ** ", parse_markt)
        .err_any()
        .q(R);
    span_parse(&mut None, "Markt Graz ", parse_markt)
        .ok_any()
        .rest("")
        .q(R);
}

#[test]
pub fn test_kultur() {
    span_parse(&mut None, "Salat: 1 GKH\n", parse_kultur)
        .ok_any()
        .q(R);
    span_parse(&mut None, " Salat : 1 GKH \n", parse_kultur)
        .ok_any()
        .q(R);
    span_parse(&mut None, "Salat : \n", parse_kultur)
        .ok_any()
        .q(R);
    span_parse(&mut None, "Salat  \n", parse_kultur)
        .ok_any()
        .q(R);

    span_parse(&mut None, ": 1 GKH\n", parse_kultur)
        .err(APCName)
        .q(R);

    span_parse(&mut None, " : 1 GKH\n", parse_kultur)
        .ok_any()
        .q(R);
}

#[test]
pub fn test_einheit() {
    span_parse(&mut None, "(K)", parse_einheit).ok_any().q(R);
    span_parse(&mut None, " (K)", parse_einheit)
        .err(APCParenthesesOpen)
        .q(R);
    span_parse(&mut None, "( K ) ", parse_einheit)
        .ok_any()
        .rest("")
        .q(R);
    span_parse(&mut None, "K ) ", parse_einheit)
        .err_any()
        .err(APCParenthesesOpen)
        .q(R);
    span_parse(&mut None, "( ) ", parse_einheit)
        .err(APCName)
        .q(R);
    span_parse(&mut None, "( K ", parse_einheit)
        .err_any()
        .err(APCParenthesesClose)
        .q(R);
}

#[test]
pub fn test_sorten() {
    span_parse(
        &mut None,
        "25 Treviso, 15 Castel Franco, 10 Di Luisa\n",
        parse_sorten,
    )
    .ok_any()
    .q(R);

    span_parse(
        &mut None,
        "25 Treviso, 15 Castel Franco, \n    10 Di Luisa",
        parse_sorten,
    )
    .ok_any()
    .q(R);

    span_parse(
        &mut None,
        "25 Treviso, 15 Castel Franco, \n    10 Di Luisa\n",
        parse_sorten,
    )
    .ok_any()
    .q(R);

    span_parse(
        &mut None,
        "25 Treviso, 15 Castel Franco, # Kommentar \n 10 Di Luisa",
        parse_sorten,
    )
    .err_any()
    .err(APCMenge)
    .q(R);

    span_parse(
        &mut None,
        "25 Treviso, 15 Castel Franco, \n 10 Di Luisa # Kommentar \n",
        parse_sorten,
    )
    .ok_any()
    .q(R);

    // at eof it's ok
    span_parse(&mut None, "25 Treviso, 15 Castel Franco, ", parse_sorten)
        .ok_any()
        .q(R);
}

#[test]
pub fn test_sorte() {
    span_parse(&mut None, "25 Treviso", parse_sorte)
        .ok_any()
        .q(R);
    span_parse(&mut None, "25 Treviso ", parse_sorte)
        .ok_any()
        .rest("")
        .q(R);
    span_parse(&mut None, " 25 Treviso, ", parse_sorte)
        .err(APCMenge)
        .q(R);
    span_parse(&mut None, "25 ", parse_sorte).err(APCName).q(R);
    span_parse(&mut None, "25 Rouge huif d'Etampes", parse_sorte)
        .ok_any()
        .q(R);
    span_parse(&mut None, "25 Rouge huif d'Etampes   ", parse_sorte)
        .ok_any()
        .q(R);
}

#[test]
pub fn test_name() {
    fn tok(name: &APName<'_>, val: &str) -> bool {
        *name.span.fragment() == val
    }

    span_parse(&mut None, "ab cd  ", token_name).ok_any().q(RT);
    span_parse(&mut None, " ab cd  ", token_name)
        .ok_any()
        .ok(tok, " ab cd")
        .q(RT);
    span_parse(&mut None, "ab cd  ", token_name)
        .ok_any()
        .rest("")
        .q(RT);
}

#[test]
pub fn test_name_kurz() {
    span_parse(&mut None, "abc", token_name_kurz).ok_any().q(RT);
    span_parse(&mut None, " abc ", token_name_kurz)
        .err(APCNameKurz)
        .q(RT);
    span_parse(&mut None, "abc\'+-²/_.", token_name_kurz)
        .ok_any()
        .q(RT);
    span_parse(&mut None, "abc ", token_name_kurz)
        .ok_any()
        .rest("")
        .q(RT);
}

#[test]
pub fn test_nummer() {
    span_parse(&mut None, "1234", token_nummer).ok_any().q(RT);
    span_parse(&mut None, " 1234 ", token_nummer) //todo: should work without err_into()
        .err(APCNummer)
        .q(RT);
    span_parse(&mut None, "1234 ", token_nummer)
        .ok_any()
        .rest("")
        .q(RT);
    span_parse(&mut None, "X", token_nummer)
        .err(APCNummer)
        .q(RT);
}

#[test]
pub fn test_menge() {
    span_parse(&mut None, "1234", token_menge).ok_any().q(RT);
    span_parse(&mut None, "1234", token_menge)
        .ok(|v: &APMenge<'_>, w: i32| v.menge == w, 1234i32)
        .q(RT);
    span_parse(&mut None, " 1234 ", token_menge)
        .err(APCMenge)
        .q(RT);
    span_parse(&mut None, "1234 ", token_menge)
        .ok_any()
        .rest("")
        .q(RT);
    span_parse(&mut None, "X", token_menge).err(APCMenge).q(RT);
}

#[test]
pub fn test_date() {
    span_parse(&mut None, "28.2.2023", token_datum)
        .ok_any()
        .q(RT);
    span_parse(&mut None, " 28.2.2023 ", token_datum)
        .err(APCDay)
        .q(RT);
    span_parse(&mut None, "28.2.2023 ", token_datum)
        .ok_any()
        .rest("")
        .q(RT);
    span_parse(&mut None, " 28. 2. 2023 ", token_datum)
        .err(APCDay)
        .q(RT);
    span_parse(&mut None, "X", token_datum).err(APCDay).q(RT);
    span_parse(&mut None, "2.2023", token_datum)
        .err(APCDot)
        .q(RT);
    span_parse(&mut None, "2023", token_datum).err(APCDot).q(RT);
    span_parse(&mut None, "28.2.", token_datum)
        .err(APCYear)
        .q(RT);
    span_parse(&mut None, "28.2", token_datum).err(APCDot).q(RT);
    span_parse(&mut None, "28.", token_datum)
        .err(APCMonth)
        .q(RT);
    span_parse(&mut None, "28", token_datum).err(APCDot).q(RT);
}

pub mod parser4 {
    pub use diagnostics::{
        dump_diagnostics as dump_diagnostics_v4, dump_diagnostics_info as dump_diagnostics_info_v4,
        dump_trace as dump_trace_v4,
    };
    use kparse::{Code, ParserError, ParserResult, TokenizerResult};
    use std::fmt::{Display, Formatter};

    use kparse::token_error::TokenizerError;
    #[cfg(debug_assertions)]
    use kparse::tracker::TrackSpan;
    #[cfg(not(debug_assertions))]
    use nom_locate::LocatedSpan;

    #[allow(clippy::enum_variant_names)]
    #[allow(dead_code)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum APCode {
        APCNomError,
        APCNomFailure,
        APCParseIncomplete,

        APCDatum,
        APCHeader,
        APCInteger,
        APCKommentar,
        APCMenge,
        APCName,
        APCNameKurz,
        APCNewLine,
        APCNotiz,
        APCNummer,
        APCSorte,
        APCSorten,
        APCSortenContinue,
        APCEinheit,
        APCBracketOpen,
        APCBracketClose,
        APCKultur,
        APCMarkt,
        APCKunde,
        APCLieferant,
        APCPflanzort,
        APCWochen,
        APCPlusWochen,
        APCAktion,
        APCAktionTyp,
        APCTag,
        APCWoche,
        APCMonat,
        APCBsNr,
        APCStichtag,
        APCKdNr,
        APCPlan,
        APCMetadata,
        APCAnbauplan,
        APCDay,
        APCMonth,
        APCYear,
        APCDot,
        APCComma,
        APCPlus,
        APCColon,
        APCStarStar,
        APCSlashSlash,
        APCParenthesesOpen,
        APCParenthesesClose,
    }

    impl Code for APCode {
        const NOM_ERROR: Self = Self::APCNomError;
    }

    impl Display for APCode {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let name = match self {
                APCode::APCNomError => "NomError",
                APCode::APCNomFailure => "NomFailure",
                APCode::APCParseIncomplete => "ParseIncomplete",
                APCode::APCBracketClose => "Klammer_geschlossen",
                APCode::APCBracketOpen => "Klammer_offen",
                APCode::APCDatum => "Datum",
                APCode::APCEinheit => "Einheit",
                APCode::APCInteger => "Integer",
                APCode::APCKommentar => "Kommentar",
                APCode::APCKultur => "Kultur",
                APCode::APCKunde => "Kunde",
                APCode::APCLieferant => "Lieferant",
                APCode::APCMarkt => "Markt",
                APCode::APCMenge => "Menge",
                APCode::APCName => "Name",
                APCode::APCNameKurz => "Name_ohne_Leerzeichen",
                APCode::APCNewLine => "Zeilenende",
                APCode::APCNotiz => "Notiz",
                APCode::APCNummer => "Nummer",
                APCode::APCPflanzort => "Parzelle",
                APCode::APCPlusWochen => "Plus_Wochen",
                APCode::APCSorte => "Sorte",
                APCode::APCSorten => "Sorten",
                APCode::APCSortenContinue => "Sorten_Folgezeile",
                APCode::APCWochen => "Wochen",
                APCode::APCAktion => "Aktion",
                APCode::APCAktionTyp => "Überwintern | Direktsaat | Pflanzen",
                APCode::APCTag => "Tag",
                APCode::APCWoche => "Woche",
                APCode::APCMonat => "Monat",
                APCode::APCBsNr => "BsNr",
                APCode::APCStichtag => "Stichtag",
                APCode::APCKdNr => "KdNr",
                APCode::APCPlan => "Plan",
                APCode::APCMetadata => "Metadata",
                APCode::APCAnbauplan => "Anbauplan",
                APCode::APCHeader => "Header",
                APCode::APCDay => "Tag",
                APCode::APCMonth => "Monat",
                APCode::APCYear => "Jahr",
                APCode::APCDot => "._Trenner",
                APCode::APCComma => ",",
                APCode::APCPlus => "+",
                APCode::APCColon => ":",
                APCode::APCStarStar => "**",
                APCode::APCSlashSlash => "//",
                APCode::APCParenthesesOpen => "(",
                APCode::APCParenthesesClose => ")",
            };
            write!(f, "{}", name)
        }
    }

    #[cfg(not(debug_assertions))]
    pub type APSpan<'s> = LocatedSpan<&'s str>;
    #[cfg(debug_assertions)]
    pub type APSpan<'s> = TrackSpan<'s, APCode, &'s str>;
    pub type APParserError<'s> = ParserError<APCode, APSpan<'s>>;
    pub type APTokenizerError<'s> = TokenizerError<APCode, APSpan<'s>>;
    pub type APParserResult<'s, O> = ParserResult<APCode, APSpan<'s>, O>;
    pub type APTokenizerResult<'s, O> = TokenizerResult<APCode, APSpan<'s>, O>;
    pub type APNomResult<'s> = ParserResult<APCode, APSpan<'s>, APSpan<'s>>;

    pub mod diagnostics {
        use crate::parser4::{APCode, APParserError, APSpan, APTokenizerError};
        use kparse::spans::SpanLines;
        use kparse::test::{Report, Test};
        use kparse::tracker::Tracks;
        use nom_locate::LocatedSpan;
        use std::ffi::OsStr;
        use std::fmt::Debug;
        use std::path::{Path, PathBuf};

        /// Write out the Tracer.
        #[allow(dead_code)]
        pub fn dump_trace(tracks: &Tracks<APCode, &'_ str>) {
            println!("{:?}", tracks);
        }

        /// Dumps the full parser trace if any test failed.
        #[derive(Clone, Copy)]
        pub struct ReportDiagnostics;

        impl<'s, P, O> Report<Test<'s, P, APSpan<'s>, O, APTokenizerError<'s>>> for ReportDiagnostics
        where
            O: Debug,
        {
            #[track_caller]
            fn report(&self, test: &Test<'s, P, APSpan<'s>, O, APTokenizerError<'s>>) {
                if test.failed.get() {
                    match &test.result {
                        Ok(_v) => {}
                        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                            dump_diagnostics_tok(&PathBuf::from(""), test.span, e, "", true);
                        }
                        Err(nom::Err::Incomplete(_e)) => {}
                    }
                    panic!("test failed");
                }
            }
        }

        /// Write some diagnostics.
        #[allow(clippy::collapsible_else_if)]
        #[allow(clippy::collapsible_if)]
        pub fn dump_diagnostics_tok(
            src: &Path,
            orig: APSpan<'_>,
            err: &APTokenizerError<'_>,
            msg: &str,
            is_err: bool,
        ) {
            // let txt = SpanLines::new(orig);
            //
            // let text1 = txt.get_lines_around(&err.span, 3);

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
        }

        impl<'s, P, O> Report<Test<'s, P, APSpan<'s>, O, APParserError<'s>>> for ReportDiagnostics
        where
            O: Debug,
        {
            #[track_caller]
            fn report(&self, test: &Test<'s, P, APSpan<'s>, O, APParserError<'s>>) {
                if test.failed.get() {
                    match &test.result {
                        Ok(_v) => {}
                        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                            dump_diagnostics(&PathBuf::from(""), test.span, e, "", true);
                        }
                        Err(nom::Err::Incomplete(_e)) => {}
                    }
                    panic!("test failed");
                }
            }
        }

        /// Write some diagnostics.
        #[allow(clippy::collapsible_else_if)]
        #[allow(clippy::collapsible_if)]
        pub fn dump_diagnostics(
            src: &Path,
            orig: APSpan<'_>,
            err: &APParserError<'_>,
            msg: &str,
            is_err: bool,
        ) {
            let txt = SpanLines::new(orig);

            let text1 = txt.get_lines_around(&err.span, 3);

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
                            println!("Erwarted war: {}", msg);
                        } else {
                            println!("Erwarted war: {}", err.code);
                        }
                    }
                }

                for (line, exp) in &expect {
                    if t.location_line() == *line {
                        for exp in exp {
                            println!("      {}^", " ".repeat(exp.span.get_utf8_column() - 1));
                            println!("Erwarted war: {}", exp.code);
                        }
                    }
                }
            }

            for (_line, sugg) in err.suggested_grouped_by_line() {
                for sug in sugg {
                    println!("Hinweis: {}", sug.code);
                }
            }

            if let Some(n) = err.nom() {
                println!(
                    "Parser-Details: {:?} {}:{}:{:?}",
                    n.kind,
                    n.span.location_line(),
                    n.span.get_utf8_column(),
                    n.span.escape_debug().take(40).collect::<String>()
                );
            }
        }

        /// Write some diagnostics.
        #[allow(dead_code)]
        pub fn dump_diagnostics_info<X: Copy>(
            src: &Path,
            orig: LocatedSpan<&str, X>,
            err: &APParserError<'_>,
            msg: &str,
        ) {
            let txt = SpanLines::new(orig);

            let text1 = txt.get_lines_around(&err.span, 0);

            println!();
            if !msg.is_empty() {
                println!(
                    "Achtung: {:?}: {}",
                    src.file_name().unwrap_or_else(|| OsStr::new("")),
                    msg
                );
            } else {
                println!(
                    "Achtung: {:?}: {}",
                    src.file_name().unwrap_or_else(|| OsStr::new("")),
                    err.code
                );
            }

            for t in &text1 {
                if t.location_line() == err.span.location_line() {
                    println!("*{:04} {}", t.location_line(), t);
                } else {
                    println!(" {:04}  {}", t.location_line(), t);
                }

                if t.location_line() == err.span.location_line() {
                    println!("      {}^", " ".repeat(err.span.get_utf8_column() - 1));
                }
            }
        }
    }

    pub mod ast {
        use crate::parser4::APCode::*;
        use crate::parser4::APSpan;
        use chrono::NaiveDate;
        use std::fmt::{Debug, Formatter};

        #[derive(Clone)]
        pub enum PlanValues<'s> {
            Stichtag(APStichtag<'s>),
            BsNr(APBsNr<'s>),
            Monat(APMonat<'s>),
            Woche(APWoche<'s>),
            Tag(APTag<'s>),

            Pflanzort(APPflanzort<'s>),

            Kunde(APKunde<'s>),
            Markt(APMarkt<'s>),
            Lieferant(APLieferant<'s>),
            Aktion(APAktion<'s>),

            Kultur(APKultur<'s>),
            Kommentar(APKommentar<'s>),
            Notiz(APNotiz<'s>),
        }

        impl<'s> Debug for PlanValues<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self {
                    PlanValues::Stichtag(v) => write!(f, "{:?}", v)?,
                    PlanValues::BsNr(v) => write!(f, "{:?}", v)?,
                    PlanValues::Monat(v) => write!(f, "{:?}", v)?,
                    PlanValues::Woche(v) => write!(f, "{:?}", v)?,
                    PlanValues::Tag(v) => write!(f, "{:?}", v)?,
                    PlanValues::Kommentar(v) => write!(f, "{:?}", v)?,
                    PlanValues::Notiz(v) => write!(f, "{:?}", v)?,
                    PlanValues::Kunde(v) => write!(f, "{:?}", v)?,
                    PlanValues::Lieferant(v) => write!(f, "{:?}", v)?,
                    PlanValues::Markt(v) => write!(f, "{:?}", v)?,
                    PlanValues::Aktion(v) => write!(f, "{:?}", v)?,
                    PlanValues::Pflanzort(v) => write!(f, "{:?}", v)?,
                    PlanValues::Kultur(v) => write!(f, "{:?}", v)?,
                }
                Ok(())
            }
        }

        #[derive(Clone)]
        pub struct APAnbauPlan<'s> {
            pub plan: APPlan<'s>,
            pub kdnr: Option<APKdNr<'s>>,
            pub data: Vec<PlanValues<'s>>,
        }

        impl<'s> Debug for APAnbauPlan<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {}", APCAnbauplan, self.plan.name.span)?;
                if let Some(kdnr) = &self.kdnr {
                    write!(f, "{}", kdnr.kdnr.nummer)?;
                }
                for v in &self.data {
                    writeln!(f, "{:?}", v)?;
                }
                Ok(())
            }
        }

        #[derive(Clone)]
        pub struct APPlan<'s> {
            pub name: APName<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APPlan<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCPlan,
                    self.name.span,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APKdNr<'s> {
            pub kdnr: APNummer<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APKdNr<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCKdNr,
                    self.kdnr.nummer,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APStichtag<'s> {
            pub stichtag: APDatum<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APStichtag<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCStichtag,
                    self.stichtag.datum,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APBsNr<'s> {
            pub bsnr: APNummer<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APBsNr<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCBsNr,
                    self.bsnr.nummer,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APMonat<'s> {
            pub monat: APName<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APMonat<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCMonat,
                    self.monat.span,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APWoche<'s> {
            pub datum: APDatum<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APWoche<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCWoche,
                    self.datum.datum,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APTag<'s> {
            pub tage: APNummer<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APTag<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCTag,
                    self.tage.nummer,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APAktion<'s> {
            pub aktion: APSpan<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APAktion<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCAktion,
                    self.aktion,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APPflanzort<'s> {
            pub ort: APName<'s>,
            pub kultur: Option<APName<'s>>,
            pub start: Option<APWochen<'s>>,
            pub dauer: Option<APWochen<'s>>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APPflanzort<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {}", APCPflanzort, self.ort.span,)?;
                if let Some(kultur) = &self.kultur {
                    write!(f, " {}", kultur.span)?;
                }
                if let Some(start) = &self.start {
                    write!(f, " {}", start.wochen.nummer)?;
                }
                if let Some(dauer) = &self.dauer {
                    write!(f, " +{}", dauer.wochen.nummer)?;
                }
                write!(
                    f,
                    " {}:\"{}\"",
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APWochen<'s> {
            pub wochen: APNummer<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APWochen<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCWochen,
                    self.wochen.nummer,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APKunde<'s> {
            pub name: APName<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APKunde<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCKunde,
                    self.name.span,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APLieferant<'s> {
            pub name: APName<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APLieferant<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCLieferant,
                    self.name.span,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APMarkt<'s> {
            pub name: APName<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APMarkt<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCMarkt,
                    self.name.span,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APKultur<'s> {
            pub kultur: APName<'s>,
            pub einheit: Option<APEinheit<'s>>,
            pub sorten: APSorten<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APKultur<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {}", APCKultur, self.kultur.span)?;
                if let Some(einheit) = &self.einheit {
                    write!(f, " ({})", einheit.span)?;
                }
                write!(
                    f,
                    " {}:\"{}\"",
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )?;
                write!(f, "[")?;
                for (i, s) in self.sorten.sorten.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{:?}", s)?;
                }
                write!(f, "]")?;
                Ok(())
            }
        }

        #[derive(Clone)]
        pub struct APEinheit<'s> {
            pub einheit: APName<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APEinheit<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCEinheit,
                    self.einheit.span,
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APSorten<'s> {
            pub sorten: Vec<APSorte<'s>>,
            pub kommentar: Option<APKommentar<'s>>,
            pub notiz: Option<APNotiz<'s>>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APSorten<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "[")?;
                for (i, s) in self.sorten.iter().enumerate() {
                    if f.alternate() {
                        writeln!(f)?;
                    }
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", s)?;
                }
                write!(f, "] ")?;
                if f.alternate() {
                    writeln!(f)?;
                }
                write!(f, "{:?}", self.kommentar)?;
                Ok(())
            }
        }

        #[derive(Clone)]
        pub struct APSorte<'s> {
            pub menge: APMenge<'s>,
            pub name: APName<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APSorte<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {} {}:\"{}\"",
                    APCSorte,
                    self.menge.menge,
                    self.name.span.escape_default(),
                    self.span.location_offset(),
                    (*self.span).escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APName<'s> {
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APName<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {}:\"{}\"",
                    APCName,
                    self.span.location_offset(),
                    self.span.escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APNummer<'s> {
            pub nummer: u32,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APNummer<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCNummer,
                    self.nummer,
                    self.span.location_offset(),
                    self.span.escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APMenge<'s> {
            pub menge: i32,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APMenge<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCMenge,
                    self.menge,
                    self.span.location_offset(),
                    self.span.escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APDatum<'s> {
            pub datum: NaiveDate,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APDatum<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {} {}:\"{}\"",
                    APCDatum,
                    self.datum,
                    self.span.location_offset(),
                    self.span.escape_default()
                )
            }
        }

        #[derive(Clone)]
        pub struct APKommentar<'s> {
            pub tag: APSpan<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APKommentar<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {}:\"{}\"",
                    APCKommentar,
                    self.span.location_offset(),
                    self.span
                )
            }
        }

        #[derive(Clone)]
        pub struct APNotiz<'s> {
            pub tag: APSpan<'s>,
            pub span: APSpan<'s>,
        }

        impl<'s> Debug for APNotiz<'s> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {}:\"{}\"",
                    APCNotiz,
                    self.span.location_offset(),
                    self.span
                )
            }
        }
    }

    #[allow(clippy::module_inception)]
    #[allow(dead_code)]
    pub mod parser {
        use crate::parser4::ast::*;
        use crate::parser4::nom_tokens::{
            nom_aktion_aktion, nom_colon, nom_comma, nom_empty, nom_header, nom_is_nl,
            nom_kommentar, nom_kommentar_tag, nom_kw, nom_metadata, nom_nl, nom_notiz,
            nom_notiz_tag, nom_number, nom_par_close, nom_par_open, nom_plus, nom_tag_aktion,
            nom_tag_bsnr, nom_tag_kdnr, nom_tag_kunde, nom_tag_lieferant, nom_tag_markt,
            nom_tag_monat, nom_tag_pflanzort, nom_tag_plan, nom_tag_stichtag, nom_tag_tag,
            nom_tag_w, nom_tag_woche, nom_ws, span_ws_nl,
        };
        use crate::parser4::tokens::{
            token_datum, token_menge, token_name, token_name_kurz, token_nummer,
        };
        use crate::parser4::APCode::*;
        use crate::parser4::{nom_tokens, APParserError, APSpan};
        use crate::parser4::{APParserResult, APTokenizerResult};
        use kparse::combinators::{err_into, separated_list_trailing1, track};
        use kparse::prelude::*;
        use kparse::{Context, ParserError};
        use nom::branch::alt;
        use nom::combinator::{consumed, not, opt};
        use nom::multi::separated_list0;
        use nom::sequence::tuple;
        use nom::Parser;

        pub fn parse_anbauplan(rest: APSpan<'_>) -> APParserResult<'_, APAnbauPlan<'_>> {
            Context.enter(APCAnbauplan, rest);

            let mut loop_rest = rest;
            loop {
                let rest2 = loop_rest;

                let rest2 = match nom_metadata(rest2) {
                    Ok((rest2, _meta)) => rest2,
                    Err(_) => break,
                };

                loop_rest = span_ws_nl(rest2);
            }
            let rest = span_ws_nl(loop_rest);

            let (rest, plan) = parse_plan(rest).track()?;
            let rest = span_ws_nl(rest);

            let (rest, kdnr) = opt(parse_kdnr)(rest).track()?;
            let rest = span_ws_nl(rest);

            let mut data = Vec::new();
            let mut loop_rest = rest;
            loop {
                // skip empty lines and whitespace
                loop_rest = span_ws_nl(loop_rest);

                match parse_stichtag(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Stichtag(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCStichtag => {}
                    Err(e) => return Context.err(e),
                }
                match parse_bsnr(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::BsNr(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCBsNr => {}
                    Err(e) => return Context.err(e),
                }
                match parse_monat(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Monat(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCMonat => {}
                    Err(e) => return Context.err(e),
                }
                match parse_woche(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Woche(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCWoche => {}
                    Err(e) => return Context.err(e),
                }
                match parse_tag(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Tag(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCTag => {}
                    Err(e) => return Context.err(e),
                }
                match parse_notiz.err_into::<APParserError<'_>>().parse(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Notiz(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCNotiz => {}
                    Err(e) => return Context.err(e),
                }
                match parse_kommentar
                    .err_into::<APParserError<'_>>()
                    .parse(loop_rest)
                {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Kommentar(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCKommentar => {}
                    Err(e) => return Context.err(e),
                }
                match parse_kunde(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Kunde(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCKunde => {}
                    Err(e) => return Context.err(e),
                }
                match parse_lieferant(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Lieferant(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCLieferant => {}
                    Err(e) => return Context.err(e),
                }
                match parse_markt(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Markt(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCMarkt => {}
                    Err(e) => return Context.err(e),
                }
                match parse_aktion(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Aktion(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCAktion => {}
                    Err(e) => return Context.err(e),
                }
                match parse_pflanzort(loop_rest) {
                    Ok((rest, val)) => {
                        loop_rest = rest;
                        data.push(PlanValues::Pflanzort(val));
                        continue;
                    }
                    Err(nom::Err::Error(e)) if e.code == APCPflanzort => {}
                    Err(e) => return Context.err(e),
                }

                if loop_rest.len() > 0 {
                    match parse_kultur(loop_rest) {
                        Ok((rest, val)) => {
                            loop_rest = rest;
                            data.push(PlanValues::Kultur(val));
                            continue;
                        }
                        Err(e) => return Context.err(e),
                    }
                };

                break;
            }
            let rest = loop_rest;

            Context.ok(rest, rest, APAnbauPlan { plan, kdnr, data })
        }

        pub fn parse_plan(input: APSpan<'_>) -> APParserResult<'_, APPlan<'_>> {
            track(
                APCPlan,
                consumed(tuple((nom_header, nom_tag_plan, token_name, nom_header))),
            )
            .map(|(span, (_, _, name, _))| APPlan { name, span })
            .err_into()
            .parse(input)
        }

        pub fn parse_kdnr(input: APSpan<'_>) -> APParserResult<'_, APKdNr<'_>> {
            track(APCKdNr, consumed(tuple((nom_tag_kdnr, token_nummer))))
                .map(|(span, (_, kdnr))| APKdNr { kdnr, span })
                .err_into()
                .parse(input)
        }

        pub fn parse_stichtag(input: APSpan<'_>) -> APParserResult<'_, APStichtag<'_>> {
            track(
                APCStichtag,
                consumed(tuple((
                    nom_header,
                    nom_tag_stichtag,
                    token_datum,
                    opt(tuple((nom_par_open, nom_kw, nom_par_close))),
                    nom_header,
                ))),
            )
            .map(|(span, (_, _, stichtag, _, _))| APStichtag { stichtag, span })
            .err_into()
            .parse(input)
        }

        pub fn parse_bsnr(input: APSpan<'_>) -> APParserResult<'_, APBsNr<'_>> {
            track(APCBsNr, consumed(tuple((nom_tag_bsnr, token_nummer))))
                .map(|(span, (_, bsnr))| APBsNr { bsnr, span })
                .err_into()
                .parse(input)
        }

        pub fn parse_monat(input: APSpan<'_>) -> APParserResult<'_, APMonat<'_>> {
            track(
                APCMonat,
                consumed(tuple((nom_header, nom_tag_monat, token_name, nom_header))),
            )
            .map(|(span, (_, _, monat, _))| APMonat { monat, span })
            .err_into()
            .parse(input)
        }

        pub fn parse_woche(input: APSpan<'_>) -> APParserResult<'_, APWoche<'_>> {
            track(
                APCWoche,
                consumed(tuple((
                    nom_header,
                    nom_tag_woche,
                    token_datum,
                    opt(nom_par_open),
                    opt(nom_kw),
                    opt(nom_par_close),
                    nom_header,
                ))),
            )
            .map(|(span, (_, _, datum, _, _, _, _))| APWoche { datum, span })
            .err_into()
            .parse(input)
        }

        pub fn parse_tag(input: APSpan<'_>) -> APParserResult<'_, APTag<'_>> {
            track(
                APCTag,
                consumed(tuple((nom_header, nom_tag_tag, token_nummer, nom_header))),
            )
            .map(|(span, (_, _, tage, _))| APTag { tage, span })
            .err_into()
            .parse(input)
        }

        pub fn parse_aktion(input: APSpan<'_>) -> APParserResult<'_, APAktion<'_>> {
            track(
                APCAktion,
                consumed(tuple((nom_tag_aktion, nom_aktion_aktion))),
            )
            .map(|(span, (_, aktion))| APAktion { aktion, span })
            .err_into()
            .parse(input)
        }

        pub fn parse_pflanzort(input: APSpan<'_>) -> APParserResult<'_, APPflanzort<'_>> {
            track(
                APCPflanzort,
                consumed(tuple((
                    nom_tag_pflanzort,
                    token_name_kurz,
                    opt(token_name_kurz),
                    opt(nom_par_open),
                    opt(parse_wochen),
                    opt(parse_pluswochen),
                    opt(nom_par_close),
                ))),
            )
            .map(|(span, (_, ort, kultur, _, start, dauer, _))| APPflanzort {
                ort,
                kultur,
                start,
                dauer,
                span,
            })
            .err_into()
            .parse(input)
        }

        pub fn parse_wochen(input: APSpan<'_>) -> APTokenizerResult<'_, APWochen<'_>> {
            track(
                APCWochen, //
                consumed(tuple((token_nummer, nom_tag_w))),
            )
            .map(|(span, (wochen, _))| APWochen { wochen, span })
            .err_into()
            .parse(input)
        }

        pub fn parse_pluswochen(input: APSpan<'_>) -> APTokenizerResult<'_, APWochen<'_>> {
            track(
                APCPlusWochen,
                consumed(tuple((nom_plus, token_nummer, nom_tag_w))),
            )
            .map(|(span, (_, wochen, _))| APWochen { wochen, span })
            .parse(input)
        }

        pub fn parse_kunde(input: APSpan<'_>) -> APParserResult<'_, APKunde<'_>> {
            track(APCKunde, consumed(tuple((nom_tag_kunde, token_name))))
                .map(|(span, (_, name))| APKunde { name, span })
                .err_into()
                .parse(input)
        }

        pub fn parse_lieferant(input: APSpan<'_>) -> APParserResult<'_, APLieferant<'_>> {
            track(
                APCLieferant,
                consumed(tuple((nom_tag_lieferant, token_name))),
            )
            .map(|(span, (_, name))| APLieferant { name, span })
            .err_into()
            .parse(input)
        }

        pub fn parse_markt(input: APSpan<'_>) -> APParserResult<'_, APMarkt<'_>> {
            track(APCMarkt, consumed(tuple((nom_tag_markt, token_name))))
                .map(|(span, (_, name))| APMarkt { name, span })
                .err_into()
                .parse(input)
        }

        pub fn parse_kultur(input: APSpan<'_>) -> APParserResult<'_, APKultur<'_>> {
            Context.enter(APCKultur, input);

            let (rest, kultur) = consumed(tuple((
                token_name.err_into(),
                opt(parse_einheit),
                consumed(opt(err_into(nom_colon).and(parse_sorten))),
            )))
            .map(|(span, (kultur, einheit, (sorten_span, sorten)))| {
                //
                APKultur {
                    kultur,
                    einheit,
                    sorten: if let Some((_, sorten)) = sorten {
                        sorten
                    } else {
                        APSorten {
                            sorten: Vec::new(),
                            kommentar: None,
                            notiz: None,
                            span: sorten_span,
                        }
                    },
                    span,
                }
            })
            .parse(input)?;

            // must be at line end now, and can eat some whitespace
            let rest = if !nom_is_nl(rest) {
                return Context.err(ParserError::new(APCSorten, rest));
            } else {
                span_ws_nl(rest)
            };

            Context.ok(rest, input, kultur)
        }

        pub fn parse_einheit(input: APSpan<'_>) -> APParserResult<'_, APEinheit<'_>> {
            track(
                APCEinheit,
                consumed(tuple((nom_par_open, token_name, nom_par_close))),
            )
            .map(|(span, (_, einheit, _))| APEinheit { einheit, span })
            .err_into()
            .parse(input)
        }

        pub fn parse_sorten(input: APSpan<'_>) -> APParserResult<'_, APSorten<'_>> {
            track(
                APCSorten,
                tuple((
                    consumed(
                        separated_list0(
                            tuple((nom_nl, nom_ws)),
                            separated_list_trailing1(nom_comma, parse_sorte),
                        )
                        .map(|v| v.into_iter().flatten().collect()),
                    ),
                    opt(parse_notiz.err_into()),
                    opt(parse_kommentar.err_into()),
                    not(tuple((nom_nl, nom_ws, nom_number))).with_code(APCMenge),
                )),
            )
            .map(|((span, sorten), notiz, kommentar, _)| APSorten {
                sorten,
                kommentar,
                notiz,
                span,
            })
            .err_into()
            .parse(input)
        }

        pub fn parse_sorte(input: APSpan<'_>) -> APTokenizerResult<'_, APSorte<'_>> {
            track(APCSorte, consumed(tuple((token_menge, token_name))))
                .map(|(span, (menge, name))| APSorte { menge, name, span })
                .err_into()
                .parse(input)
        }

        pub fn parse_kommentar(rest: APSpan<'_>) -> APTokenizerResult<'_, APKommentar<'_>> {
            track(
                APCKommentar,
                consumed(tuple((nom_kommentar_tag, nom_kommentar))),
            )
            .map(|(span, (_, tag))| APKommentar { tag, span })
            .parse(rest)
        }

        pub fn parse_notiz(rest: APSpan<'_>) -> APTokenizerResult<'_, APNotiz<'_>> {
            track(
                APCNotiz, //
                consumed(tuple((nom_notiz_tag, nom_notiz))),
            )
            .map(|(span, (_, tag))| APNotiz { tag, span })
            .parse(rest)
        }
    }

    pub mod tokens {
        use crate::parser4::ast::{APDatum, APMenge, APName, APNummer};
        use crate::parser4::nom_tokens::{nom_dot, nom_name, nom_name_kurz, nom_number};
        use crate::parser4::APCode::*;
        use crate::parser4::{APSpan, APTokenizerResult};
        use chrono::NaiveDate;
        use kparse::prelude::*;
        use kparse::token_error::TokenizerError;
        use nom::sequence::tuple;
        use nom::Parser;

        pub fn token_name(rest: APSpan<'_>) -> APTokenizerResult<'_, APName<'_>> {
            match nom_name(rest) {
                Ok((rest, tok)) => {
                    // trim trailing whitespace after the fact.
                    let trim = tok.trim_end();

                    // the trimmed span is part of original.
                    // so reusing the rest ought to be fine.
                    let tok = unsafe {
                        APSpan::new_from_raw_offset(
                            tok.location_offset(),
                            tok.location_line(),
                            trim,
                            tok.extra,
                        )
                    };

                    // could rewind the rest too, but since it'_ whitespace
                    // which would be thrown away anyway ...

                    Ok((rest, APName { span: tok }))
                }
                Err(e) => Err(e.with_code(APCName)),
            }
        }

        pub fn token_name_kurz(rest: APSpan<'_>) -> APTokenizerResult<'_, APName<'_>> {
            match nom_name_kurz(rest) {
                Ok((rest, tok)) => Ok((rest, APName { span: tok })),
                Err(e) => Err(e.with_code(APCNameKurz)),
            }
        }

        pub fn token_nummer(rest: APSpan<'_>) -> APTokenizerResult<'_, APNummer<'_>> {
            nom_number
                .parse_from_str(APCNummer)
                .consumed()
                .map(|(span, nummer)| APNummer { nummer, span })
                .parse(rest)
        }

        pub fn token_menge(rest: APSpan<'_>) -> APTokenizerResult<'_, APMenge<'_>> {
            nom_number
                .with_code(APCMenge)
                .parse_from_str(APCMenge)
                .consumed()
                .map(|(span, menge)| APMenge { menge, span })
                .parse(rest)
        }

        pub fn token_datum(input: APSpan<'_>) -> APTokenizerResult<'_, APDatum<'_>> {
            let (rest, (span, (iday, _, imonth, _, iyear))) = tuple((
                nom_number.with_code(APCDay).parse_from_str(APCDay),
                nom_dot,
                nom_number.with_code(APCMonth).parse_from_str(APCMonth),
                nom_dot,
                nom_number.with_code(APCYear).parse_from_str(APCYear),
            ))
            .consumed()
            .parse(input)?;

            let datum = NaiveDate::from_ymd_opt(iyear, imonth, iday);

            if let Some(datum) = datum {
                Ok((rest, APDatum { datum, span }))
            } else {
                Err(TokenizerError::new(APCDatum, span).error())
            }
        }
    }

    pub mod nom_tokens {
        use crate::parser4::APCode::*;
        use crate::parser4::{APSpan, APTokenizerResult};
        use kparse::combinators::pchar;
        use kparse::prelude::*;
        use nom::branch::alt;
        use nom::bytes::complete::{
            tag, tag_no_case, take_till, take_till1, take_while, take_while1,
        };
        use nom::character::complete::one_of;
        use nom::character::complete::{digit1, not_line_ending};
        use nom::combinator::{opt, recognize};
        use nom::sequence::{preceded, terminated, tuple};
        use nom::Parser;
        use nom::{AsChar, InputTake, InputTakeAtPosition};

        pub fn nom_tag_plan(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag_no_case("plan"), nom_ws)
                .with_code(APCPlan)
                .parse(i)
        }

        pub fn nom_metadata(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            recognize(tuple((
                take_till1(|c: char| c == ':' || c == '\n' || c == '\r'),
                nom_colon,
                not_line_ending,
            )))
            .with_code(APCMetadata)
            .parse(i)
        }

        pub fn nom_tag_kdnr(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag_no_case("kdnr"), nom_ws)
                .with_code(APCKdNr)
                .parse(i)
        }

        pub fn nom_tag_stichtag(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag_no_case("stichtag"), nom_ws)
                .with_code(APCStichtag)
                .parse(i)
        }

        pub fn nom_tag_bsnr(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag_no_case("bsnr"), nom_ws)
                .with_code(APCBsNr)
                .parse(i)
        }

        pub fn nom_tag_monat(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag_no_case("monat"), nom_ws)
                .with_code(APCMonat)
                .parse(i)
        }

        pub fn nom_tag_woche(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag_no_case("woche"), nom_ws)
                .with_code(APCWoche)
                .parse(i)
        }

        pub fn nom_tag_tag(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag_no_case("tag"), nom_ws)
                .with_code(APCTag)
                .parse(i)
        }

        pub fn nom_tag_aktion(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag("=>"), nom_ws).with_code(APCAktion).parse(i)
        }

        pub fn nom_aktion_aktion(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(
                alt((tag("Überwintern"), tag("Direktsaat"), tag("Pflanzen"))),
                nom_ws,
            )
            .with_code(APCAktion)
            .parse(i)
        }

        pub fn nom_tag_pflanzort(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(alt((pchar('@'), tag_no_case("parzelle"))), nom_ws)
                .with_code(APCPflanzort)
                .parse(i)
        }

        pub fn nom_tag_w(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(one_of("wW")), nom_ws)
                .with_code(APCWoche)
                .parse(i)
        }

        pub fn nom_tag_kunde(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag_no_case("kunde"), nom_ws)
                .with_code(APCKunde)
                .parse(i)
        }

        pub fn nom_tag_lieferant(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag_no_case("lieferant"), nom_ws)
                .with_code(APCLieferant)
                .parse(i)
        }

        pub fn nom_tag_markt(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag_no_case("markt"), nom_ws)
                .with_code(APCMarkt)
                .parse(i)
        }

        pub fn nom_kommentar_tag(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag("#")), nom_ws)
                .with_code(APCKommentar)
                .parse(i)
        }

        pub fn nom_kommentar(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(take_till(|c: char| c == '\n'), nom_ws)
                .with_code(APCKommentar)
                .parse(i)
        }

        pub fn nom_notiz_tag(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(tag("##"), nom_ws).with_code(APCNotiz).parse(i)
        }

        pub fn nom_notiz(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(take_till(|c: char| c == '\n'), nom_ws)
                .with_code(APCNotiz)
                .parse(i)
        }

        pub fn nom_name(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(
                take_while1(|c: char| c.is_alphanumeric() || c == ' ' || "\'+-²/_.".contains(c)),
                nom_ws,
            )
            .with_code(APCName)
            .parse(i)
        }

        pub fn nom_name_kurz(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(
                take_while1(|c: char| c.is_alphanumeric() || "\'+-²/_.".contains(c)),
                nom_ws,
            )
            .with_code(APCNameKurz)
            .parse(i)
        }

        pub fn nom_kw(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(preceded(tag_no_case("KW"), digit1), nom_ws)
                .with_code(APCWoche)
                .parse(i)
        }

        pub fn nom_number(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(digit1, nom_ws).with_code(APCNummer).parse(i)
        }

        pub fn nom_dot(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(pchar('.'), nom_ws).with_code(APCDot).parse(i)
        }

        pub fn nom_comma(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(pchar(','), nom_ws).with_code(APCComma).parse(i)
        }

        pub fn nom_plus(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(pchar('+'), nom_ws).with_code(APCPlus).parse(i)
        }

        pub fn nom_colon(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(pchar(':'), nom_ws).with_code(APCColon).parse(i)
        }

        // pub fn nom_star_star(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
        //     terminated(tag("**"), nom_ws)
        //         .with_code(APCStarStar)
        //         .parse(i)
        // }
        //
        // pub fn nom_slash_slash(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
        //     terminated(tag("//"), nom_ws)
        //         .with_code(APCSlashSlash)
        //         .parse(i)
        // }

        pub fn nom_header(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(take_while(|c| c == '='), nom_ws)
                .with_code(APCHeader)
                .parse(i)
        }

        pub fn nom_par_open(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(pchar('('), nom_ws)
                .with_code(APCParenthesesOpen)
                .parse(i)
        }

        pub fn nom_par_close(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(pchar(')'), nom_ws)
                .with_code(APCParenthesesClose)
                .parse(i)
        }

        pub fn nom_ws(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            i.split_at_position_complete(|item| {
                let c = item.as_char();
                !(c == ' ' || c == '\t')
            })
        }

        pub fn nom_ws2(i: APSpan<'_>) -> APSpan<'_> {
            match i.split_at_position_complete::<_, nom::error::Error<APSpan<'_>>>(|item| {
                let c = item.as_char();
                !(c == ' ' || c == '\t')
            }) {
                Ok((rest, _)) => rest,
                Err(_) => i,
            }
        }

        pub fn span_ws_nl(i: APSpan<'_>) -> APSpan<'_> {
            match i.split_at_position_complete::<_, nom::error::Error<APSpan<'_>>>(|item| {
                let c = item.as_char();
                !(c == ' ' || c == '\t' || c == '\n' || c == '\r')
            }) {
                Ok((rest, _)) => rest,
                Err(_) => i,
            }
        }

        pub fn nom_is_nl(i: APSpan<'_>) -> bool {
            terminated(take_while1(|c: char| c == '\n' || c == '\r'), nom_ws)(i).is_ok()
        }

        pub fn nom_is_comment_or_notiz(i: APSpan<'_>) -> bool {
            terminated(take_while1(|c: char| c == '#'), nom_ws)(i).is_ok()
        }

        pub fn span_empty(i: APSpan<'_>) -> APSpan<'_> {
            i.take(0)
        }

        pub fn nom_empty(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            Ok((i, i.take(0)))
        }

        pub fn nom_ws_nl(i: APSpan<'_>) -> APTokenizerResult<'_, ()> {
            match i.split_at_position_complete(|item| {
                let c = item.as_char();
                !(c == ' ' || c == '\t' || c == '\n' || c == '\r')
            }) {
                Ok((rest, _)) => Ok((rest, ())),
                Err(e) => Err(e),
            }
        }

        pub fn nom_nl(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            take_while1(|c: char| c == '\n' || c == '\r')(i)
        }
    }
}
