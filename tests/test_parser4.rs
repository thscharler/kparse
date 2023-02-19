use crate::planung4::ast::{APMenge, APName};
use crate::planung4::diagnostics::ReportDiagnostics;
use crate::planung4::nom_tokens::nom_metadata;
use crate::planung4::parser::*;
use crate::planung4::tokens::{
    token_datum, token_menge, token_name, token_name_kurz, token_nummer,
};
use crate::planung4::APCode::*;
use kparse::test::{span_parse, CheckDump};
use std::fs::read_to_string;
use std::time::Instant;

const R: ReportDiagnostics = ReportDiagnostics;
// const R: CheckDump = CheckDump;
const RT: CheckDump = CheckDump;

#[test]
pub fn timing() {
    let s = read_to_string("tests/2022_Anbauplan.txt").unwrap();
    let now = Instant::now();
    let cnt = 100;
    for _i in 0..cnt {
        span_parse(&mut None, s.as_str(), parse)
            .ok_any()
            .rest("")
            .q(CheckDump);
    }
    let duration = now.elapsed();
    println!("{:?}", duration / cnt);
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
        .err_any()
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
        .err(APCBracketOpen)
        .q(R);
    span_parse(&mut None, "( K ) ", parse_einheit)
        .ok_any()
        .rest("")
        .q(R);
    span_parse(&mut None, "K ) ", parse_einheit)
        .err_any()
        .err(APCBracketOpen)
        .q(R);
    span_parse(&mut None, "( ) ", parse_einheit)
        .err(APCName)
        .q(R);
    span_parse(&mut None, "( K ", parse_einheit)
        .err_any()
        .err(APCBracketClose)
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
    .err_any()
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

    span_parse(&mut None, "25 Treviso, 15 Castel Franco, ", parse_sorten)
        .err_any()
        .err(APCMenge)
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
        *name.span == val
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
    span_parse(&mut None, " 1234 ", token_nummer)
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

mod planung4 {
    pub use diagnostics::{
        dump_diagnostics as dump_diagnostics_v4, dump_diagnostics_info as dump_diagnostics_info_v4,
        dump_trace as dump_trace_v4,
    };
    use kparse::token_error::TokenizerError;
    #[cfg(debug_assertions)]
    use kparse::tracker::TrackSpan;
    use kparse::tracker::{TrackParserResult, TrackTokenizerResult};
    use kparse::{Code, ParserError};
    #[cfg(not(debug_assertions))]
    use nom_locate::LocatedSpan;
    use std::fmt::{Display, Formatter};

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
            };
            write!(f, "{}", name)
        }
    }

    #[cfg(debug_assertions)]
    pub type APSpan<'s> = TrackSpan<'s, APCode, &'s str>;
    #[cfg(not(debug_assertions))]
    pub type APSpan<'s> = LocatedSpan<&'s str, ()>;
    pub type APParserError<'s> = ParserError<APCode, APSpan<'s>, ()>;
    pub type APTokenizerError<'s> = TokenizerError<APCode, APSpan<'s>>;
    pub type APParserResult<'s, O> = TrackParserResult<APCode, APSpan<'s>, O, ()>;
    pub type APTokenizerResult<'s, O> = TrackTokenizerResult<APCode, APSpan<'s>, O>;

    pub mod diagnostics {
        use crate::planung4::{APCode, APParserError, APSpan};
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
        use crate::planung4::APCode::*;
        use crate::planung4::APSpan;
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
        use crate::planung4::ast::{
            APAktion, APAnbauPlan, APBsNr, APEinheit, APKdNr, APKommentar, APKultur, APKunde,
            APMarkt, APMonat, APPflanzort, APPlan, APSorte, APSorten, APStichtag, APTag, APWoche,
            APWochen, PlanValues,
        };
        use crate::planung4::ast::{APLieferant, APNotiz};
        use crate::planung4::nom_tokens::*;
        use crate::planung4::tokens::{
            token_datum, token_menge, token_name, token_name_kurz, token_nummer,
        };
        use crate::planung4::APCode::*;
        use crate::planung4::{nom_tokens, APParserResult, APSpan};
        use kparse::prelude::*;
        use kparse::{Context, ParserError};
        use nom::combinator::opt;
        use nom::sequence::tuple;

        // impl<'s, T> IntoParserResultAddSpan<'s, APCode, T> for Result<T, ParseIntError> {
        //     fn into_with_span(self, span: Span<'s>) -> ParserResult<'s, APCode, T> {
        //         match self {
        //             Ok(v) => Ok(v),
        //             Err(_) => Err(ParserError::new(APCInteger, span)),
        //         }
        //     }
        // }

        pub fn parse(rest: APSpan<'_>) -> APParserResult<'_, APAnbauPlan<'_>> {
            Context.enter(APCAnbauplan, rest);

            let mut loop_rest = rest;
            loop {
                let rest2 = loop_rest;

                let rest2 = match nom_metadata(rest2) {
                    Ok((rest2, _meta)) => rest2,
                    Err(_) => break,
                };

                loop_rest = nom_ws_nl(rest2);
            }
            let rest = nom_ws_nl(loop_rest);

            let (rest, plan) = parse_plan(rest).track()?;
            let rest = nom_ws_nl(rest);

            let (rest, kdnr) = opt(parse_kdnr)(rest).track()?;
            let rest = nom_ws_nl(rest);

            let mut data = Vec::new();
            let mut loop_rest = rest;
            loop {
                let rest2 = loop_rest;

                let rest2 = if lah_stichtag(rest2) {
                    let (rest3, val) = parse_stichtag(rest2).track()?;
                    data.push(PlanValues::Stichtag(val));
                    rest3
                } else if lah_bsnr(rest2) {
                    let (rest3, val) = parse_bsnr(rest2).track()?;
                    data.push(PlanValues::BsNr(val));
                    rest3
                } else if lah_monat(rest2) {
                    let (rest3, val) = parse_monat(rest2).track()?;
                    data.push(PlanValues::Monat(val));
                    rest3
                } else if lah_woche(rest2) {
                    let (rest3, val) = parse_woche(rest2).track()?;
                    data.push(PlanValues::Woche(val));
                    rest3
                } else if lah_tag(rest2) {
                    let (rest3, val) = parse_tag(rest2).track()?;
                    data.push(PlanValues::Tag(val));
                    rest3
                } else if lah_notiz(rest2) {
                    let (rest3, val) = parse_notiz(rest2).track()?;
                    data.push(PlanValues::Notiz(val));
                    rest3
                } else if lah_kommentar(rest2) {
                    let (rest3, val) = parse_kommentar(rest2).track()?;
                    data.push(PlanValues::Kommentar(val));
                    rest3
                } else if lah_kunde(rest2) {
                    let (rest3, val) = parse_kunde(rest2).track()?;
                    data.push(PlanValues::Kunde(val));
                    rest3
                } else if lah_lieferant(rest2) {
                    let (rest3, val) = parse_lieferant(rest2).track()?;
                    data.push(PlanValues::Lieferant(val));
                    rest3
                } else if lah_markt(rest2) {
                    let (rest3, val) = parse_markt(rest2).track()?;
                    data.push(PlanValues::Markt(val));
                    rest3
                } else if lah_aktion(rest2) {
                    let (rest3, val) = parse_aktion(rest2).track()?;
                    data.push(PlanValues::Aktion(val));
                    rest3
                } else if lah_pflanzort(rest2) {
                    let (rest3, val) = parse_pflanzort(rest2).track()?;
                    data.push(PlanValues::Pflanzort(val));
                    rest3
                } else if rest2.len() == 0 {
                    // eof shouldn't try to parse as kultur
                    rest2
                } else {
                    let (rest3, val) = parse_kultur(rest2).track()?;
                    data.push(PlanValues::Kultur(val));
                    rest3
                };

                if loop_rest == rest2 {
                    break;
                }

                // skip empty lines and whitespace
                loop_rest = nom_ws_nl(rest2);
            }
            let rest = loop_rest;

            Context.ok(rest, rest, APAnbauPlan { plan, kdnr, data })
        }

        fn lah_plan(span: APSpan<'_>) -> bool {
            nom_tokens::lah_plan(span)
        }

        pub fn parse_plan(input: APSpan<'_>) -> APParserResult<'_, APPlan<'_>> {
            Context.enter(APCPlan, input);

            let (rest, h0) = nom_header(input).track_as(APCHeader).into_parser_error()?;
            let (rest, _) = nom_tag_plan(rest).track_as(APCPlan).into_parser_error()?;
            let (rest, plan) = token_name(rest).track()?;
            let (rest, h1) = nom_header(rest).track_as(APCHeader).into_parser_error()?;

            let span = input.span_union(&h0, &h1);
            Context.ok(rest, span, APPlan { name: plan, span })
        }

        fn lah_kdnr(span: APSpan<'_>) -> bool {
            nom_tokens::lah_kdnr(span)
        }

        pub fn parse_kdnr(input: APSpan<'_>) -> APParserResult<'_, APKdNr<'_>> {
            Context.enter(APCKdNr, input);

            let (rest, _) = opt(nom_star_star)(input).track().into_parser_error()?;
            let (rest, _) = opt(nom_slash_slash)(rest).track().into_parser_error()?;

            let (rest, tag) = nom_tag_kdnr(rest).track().into_parser_error()?;
            let (rest, kdnr) = token_nummer(rest).track()?;

            let (rest, _) = opt(nom_star_star)(rest).track().into_parser_error()?;
            let (rest, _) = opt(nom_slash_slash)(rest).track().into_parser_error()?;

            let span = input.span_union(&tag, &kdnr.span);
            Context.ok(rest, span, APKdNr { kdnr, span })
        }

        fn lah_stichtag(span: APSpan<'_>) -> bool {
            nom_tokens::lah_stichtag(span)
        }

        pub fn parse_stichtag(input: APSpan<'_>) -> APParserResult<'_, APStichtag<'_>> {
            Context.enter(APCStichtag, input);

            let (rest, h0) = nom_header(input).track().into_parser_error()?;
            let (rest, _) = nom_tag_stichtag(rest).track().into_parser_error()?;
            let (rest, stichtag) = token_datum(rest).track()?;

            let (rest, _brop) = opt(nom_brop)(rest).track().into_parser_error()?;
            let (rest, _kw) = opt(nom_kw)(rest).track().into_parser_error()?;
            let (rest, _brcl) = opt(nom_brcl)(rest).track().into_parser_error()?;

            let (rest, h1) = nom_header(rest).track().into_parser_error()?;

            let span = input.span_union(&h0, &h1);
            Context.ok(rest, span, APStichtag { stichtag, span })
        }

        fn lah_bsnr(span: APSpan<'_>) -> bool {
            nom_tokens::lah_bsnr(span)
        }

        pub fn parse_bsnr(input: APSpan<'_>) -> APParserResult<'_, APBsNr<'_>> {
            Context.enter(APCBsNr, input);

            let (rest, _) = opt(nom_slash_slash)(input).track().into_parser_error()?;
            let (rest, _) = opt(nom_star_star)(rest).track().into_parser_error()?;

            let (rest, tag) = nom_tag_bsnr(rest).track().into_parser_error()?;
            let (rest, bsnr) = token_nummer(rest).track()?;

            let (rest, _) = opt(nom_star_star)(rest).track().into_parser_error()?;
            let (rest, _) = opt(nom_slash_slash)(rest).track().into_parser_error()?;

            let span = input.span_union(&tag, &bsnr.span);
            Context.ok(rest, span, APBsNr { bsnr, span })
        }

        fn lah_monat(span: APSpan<'_>) -> bool {
            nom_tokens::lah_monat(span)
        }

        pub fn parse_monat(input: APSpan<'_>) -> APParserResult<'_, APMonat<'_>> {
            Context.enter(APCMonat, input);

            let (rest, h0) = nom_header(input).track().into_parser_error()?;
            let (rest, _) = nom_tag_monat(rest).track().into_parser_error()?;
            let (rest, monat) = token_name(rest).track()?;
            let (rest, h1) = nom_header(rest).track().into_parser_error()?;

            let span = input.span_union(&h0, &h1);
            Context.ok(rest, span, APMonat { monat, span })
        }

        fn lah_woche(span: APSpan<'_>) -> bool {
            nom_tokens::lah_woche(span)
        }

        pub fn parse_woche(input: APSpan<'_>) -> APParserResult<'_, APWoche<'_>> {
            Context.enter(APCWoche, input);

            let (rest, h0) = nom_header(input).track().into_parser_error()?;

            let (rest, _) = nom_tag_woche(rest).track().into_parser_error()?;
            let (rest, datum) = token_datum(rest).track()?;

            let (rest, _brop) = opt(nom_brop)(rest).track().into_parser_error()?;
            let (rest, _kw) = opt(nom_kw)(rest).track().into_parser_error()?;
            let (rest, _brcl) = opt(nom_brcl)(rest).track().into_parser_error()?;

            let (rest, h1) = nom_header(rest).track().into_parser_error()?;

            let span = input.span_union(&h0, &h1);
            Context.ok(rest, span, APWoche { datum, span })
        }

        fn lah_tag(span: APSpan<'_>) -> bool {
            nom_tokens::lah_tag(span)
        }

        pub fn parse_tag(input: APSpan<'_>) -> APParserResult<'_, APTag<'_>> {
            Context.enter(APCTag, input);

            let (rest, h0) = nom_header(input).track().into_parser_error()?;
            let (rest, _) = nom_tag_tag(rest).track().into_parser_error()?;
            let (rest, tage) = token_nummer(rest).track()?;
            let (rest, h1) = nom_header(rest).track().into_parser_error()?;

            let span = input.span_union(&h0, &h1);
            Context.ok(rest, span, APTag { tage, span })
        }

        fn lah_aktion(span: APSpan<'_>) -> bool {
            nom_tokens::lah_aktion(span)
        }

        pub fn parse_aktion(input: APSpan<'_>) -> APParserResult<'_, APAktion<'_>> {
            Context.enter(APCAktion, input);

            let (rest, tag) = nom_tag_aktion(input).track().into_parser_error()?;
            let (rest, aktion) = nom_aktion_aktion(rest)
                .track_as(APCAktionTyp)
                .into_parser_error()?;

            let span = input.span_union(&tag, &aktion);
            Context.ok(rest, span, APAktion { aktion, span })
        }

        fn lah_pflanzort(span: APSpan<'_>) -> bool {
            nom_tokens::lah_pflanzort(span)
        }

        pub fn parse_pflanzort(input: APSpan<'_>) -> APParserResult<'_, APPflanzort<'_>> {
            Context.enter(APCPflanzort, input);

            let (rest, tag) = nom_tag_pflanzort(input).track().into_parser_error()?;

            let (rest, ort) = token_name_kurz(rest).track()?;

            let (rest, kultur) = if !lah_brop(rest) && !lah_pluswochen(rest) && !lah_wochen(rest) {
                opt(token_name_kurz)(rest).track()?
            } else {
                (rest, None)
            };

            let (rest, brop) = opt(nom_brop)(rest).track().into_parser_error()?;
            let (rest, start) = opt(parse_wochen)(rest).track()?;
            let (rest, dauer) = opt(parse_pluswochen)(rest).track()?;
            let (rest, brcl) = opt(nom_brcl)(rest).track().into_parser_error()?;

            let span = if let Some(brcl) = brcl {
                input.span_union(&tag, &brcl)
            } else if let Some(dauer) = &dauer {
                input.span_union(&tag, &dauer.span)
            } else if let Some(start) = &start {
                input.span_union(&tag, &start.span)
            } else if let Some(brop) = brop {
                input.span_union(&tag, &brop)
            } else if let Some(kultur) = &kultur {
                input.span_union(&tag, &kultur.span)
            } else {
                input.span_union(&tag, &ort.span)
            };

            Context.ok(
                rest,
                span,
                APPflanzort {
                    ort,
                    kultur,
                    start,
                    dauer,
                    span,
                },
            )
        }

        fn lah_wochen(i: APSpan<'_>) -> bool {
            tuple((nom_number, nom_tag_w))(i).is_ok()
        }

        pub fn parse_wochen(input: APSpan<'_>) -> APParserResult<'_, APWochen<'_>> {
            Context.enter(APCWochen, input);

            let (rest, wochen) = token_nummer(input).track()?;
            let (rest, w) = nom_tag_w(rest).track().into_parser_error()?;

            let span = input.span_union(&wochen.span, &w);
            Context.ok(rest, span, APWochen { wochen, span })
        }

        fn lah_pluswochen(rest: APSpan<'_>) -> bool {
            lah_plus(rest)
        }

        pub fn parse_pluswochen(input: APSpan<'_>) -> APParserResult<'_, APWochen<'_>> {
            Context.enter(APCPlusWochen, input);

            let (rest, _) = nom_plus(input).track().into_parser_error()?;
            let (rest, wochen) = token_nummer(rest).track()?;
            let (rest, w) = nom_tag_w(rest).track().into_parser_error()?;

            let span = input.span_union(&wochen.span, &w);
            Context.ok(rest, span, APWochen { wochen, span })
        }

        fn lah_kunde(span: APSpan<'_>) -> bool {
            nom_tokens::lah_kunde(span)
        }

        pub fn parse_kunde(input: APSpan<'_>) -> APParserResult<'_, APKunde<'_>> {
            Context.enter(APCKunde, input);

            let (rest, _) = opt(nom_star_star)(input).track().into_parser_error()?;
            let (rest, tag) = nom_tag_kunde(rest).track().into_parser_error()?;
            let (rest, name) = token_name(rest).track()?;
            let (rest, _) = opt(nom_star_star)(rest).track().into_parser_error()?;

            let span = input.span_union(&tag, &name.span);

            Context.ok(rest, span, APKunde { name, span })
        }

        fn lah_lieferant(span: APSpan<'_>) -> bool {
            nom_tokens::lah_lieferant(span)
        }

        pub fn parse_lieferant(input: APSpan<'_>) -> APParserResult<'_, APLieferant<'_>> {
            Context.enter(APCLieferant, input);

            let (rest, _) = opt(nom_star_star)(input).track().into_parser_error()?;
            let (rest, tag) = nom_tag_lieferant(rest).track().into_parser_error()?;
            let (rest, name) = token_name(rest).track()?;
            let (rest, _) = opt(nom_star_star)(rest).track().into_parser_error()?;

            let span = input.span_union(&tag, &name.span);

            Context.ok(rest, span, APLieferant { name, span })
        }

        fn lah_markt(span: APSpan<'_>) -> bool {
            nom_tokens::lah_markt(span)
        }

        pub fn parse_markt(input: APSpan<'_>) -> APParserResult<'_, APMarkt<'_>> {
            Context.enter(APCMarkt, input);

            let (rest, _) = opt(nom_star_star)(input).track().into_parser_error()?;
            let (rest, tag) = nom_tag_markt(rest).track().into_parser_error()?;
            let (rest, name) = token_name(rest).track()?;
            let (rest, _) = opt(nom_star_star)(rest).track().into_parser_error()?;

            let span = input.span_union(&tag, &name.span);

            Context.ok(rest, span, APMarkt { name, span })
        }

        pub fn parse_kultur(input: APSpan<'_>) -> APParserResult<'_, APKultur<'_>> {
            Context.enter(APCKultur, input);

            let (rest, kultur) = token_name(input).track()?;

            let (rest, einheit) = opt(parse_einheit)(rest).track()?;

            let (rest, sorten) = match opt(nom_colon)(rest) {
                Ok((rest, Some(_colon))) => {
                    //
                    parse_sorten(rest).track()?
                }
                Ok((rest, None)) => {
                    // if we don't have a colon, we're done here.
                    (
                        rest,
                        APSorten {
                            sorten: Vec::new(),
                            kommentar: None,
                            notiz: None,
                            span: nom_empty(rest),
                        },
                    )
                }
                Err(e) => return Context.err(e.into_parser_error()),
            };

            // must be at line end now, and can eat some whitespace
            let rest = if !nom_is_nl(rest) {
                return Context.err(ParserError::new(APCSorten, rest));
            } else {
                nom_ws_nl(rest)
            };

            let span = input.span_union(&kultur.span, &sorten.span);

            Context.ok(
                rest,
                span,
                APKultur {
                    kultur,
                    einheit,
                    sorten,
                    span,
                },
            )
        }

        pub fn parse_einheit(input: APSpan<'_>) -> APParserResult<'_, APEinheit<'_>> {
            Context.enter(APCEinheit, input);

            let (rest, brop) = nom_brop(input)
                .track_as(APCBracketOpen)
                .into_parser_error()?;
            let (rest, name) = token_name(rest).track()?;
            let (rest, brcl) = nom_brcl(rest)
                .track_as(APCBracketClose)
                .into_parser_error()?;

            let span = input.span_union(&brop, &brcl);

            Context.ok(
                rest,
                span,
                APEinheit {
                    einheit: name,
                    span,
                },
            )
        }

        pub fn parse_sorten(input: APSpan<'_>) -> APParserResult<'_, APSorten<'_>> {
            Context.enter(APCSorten, input);

            let mut sorten = Vec::new();

            let mut rest_loop = input;
            loop {
                let rest1 = rest_loop;

                let (rest1, sorte) = parse_sorte(rest1).track()?;
                sorten.push(sorte);

                let rest1 = match nom_comma(rest1) {
                    Ok((rest2, _comma)) => {
                        // at the line end?
                        if !nom_is_nl(rest2) {
                            rest2
                        } else {
                            // is at newline. consume and check next...
                            let rest2 = nom_ws_nl(rest2);

                            // next must be a number, otherwise the continue fails.
                            if lah_number(rest2) {
                                rest2
                            } else {
                                return Context.err(ParserError::new(APCSortenContinue, rest1));
                            }
                        }
                    }
                    Err(_e) => {
                        // no comma, maybe at the line end?
                        if nom_is_nl(rest1) || nom_is_comment_or_notiz(rest1) {
                            // don't eat the new line. that'_ the job of the caller.
                            rest_loop = rest1;
                            break;
                        } else {
                            // continue and fail later
                            rest1
                        }
                    }
                };

                rest_loop = nom_ws2(rest1);
            }
            let rest = rest_loop;

            let (rest, notiz) = opt(parse_notiz)(rest).track()?;
            let (rest, kommentar) = opt(parse_kommentar)(rest).track()?;

            let first = sorten.first();
            let last = sorten.last();

            let span = if let Some(first) = first {
                if let Some(last) = last {
                    input.span_union(&first.span, &last.span)
                } else {
                    unreachable!()
                }
            } else {
                nom_empty(rest)
            };

            Context.ok(
                rest,
                span,
                APSorten {
                    sorten,
                    kommentar,
                    notiz,
                    span,
                },
            )
        }

        pub fn parse_sorte(input: APSpan<'_>) -> APParserResult<'_, APSorte<'_>> {
            Context.enter(APCSorte, input);

            let (rest, menge) = token_menge(input).track()?;
            let (rest, name) = token_name(rest).track()?;

            let span = input.span_union(&menge.span, &name.span);

            Context.ok(rest, span, APSorte { menge, name, span })
        }

        fn lah_kommentar(span: APSpan<'_>) -> bool {
            nom_tokens::lah_kommentar(span)
        }

        pub fn parse_kommentar(rest: APSpan<'_>) -> APParserResult<'_, APKommentar<'_>> {
            Context.enter(APCKommentar, rest);

            let (rest, kommentar_tag) = nom_kommentar_tag(rest).track().into_parser_error()?;
            let (rest, kommentar) = nom_kommentar(rest).track().into_parser_error()?;

            Context.ok(
                rest,
                kommentar,
                APKommentar {
                    tag: kommentar_tag,
                    span: kommentar,
                },
            )
        }

        fn lah_notiz(span: APSpan<'_>) -> bool {
            nom_tokens::lah_notiz(span)
        }

        pub fn parse_notiz(rest: APSpan<'_>) -> APParserResult<'_, APNotiz<'_>> {
            Context.enter(APCNotiz, rest);

            let (rest, notiz_tag) = nom_notiz_tag(rest).track().into_parser_error()?;
            let (rest, notiz) = nom_notiz(rest).track().into_parser_error()?;

            Context.ok(
                rest,
                notiz,
                APNotiz {
                    tag: notiz_tag,
                    span: notiz,
                },
            )
        }
    }

    pub mod tokens {
        use crate::planung4::ast::{APDatum, APMenge, APName, APNummer};
        use crate::planung4::nom_tokens::{nom_dot, nom_name, nom_name_kurz, nom_number};
        use crate::planung4::APCode::*;
        use crate::planung4::{APParserResult, APSpan, APTokenizerError};
        use chrono::NaiveDate;
        use kparse::combinators::transform;
        use kparse::prelude::*;
        use kparse::{Code, ParserError};
        use nom::combinator::recognize;
        use nom::sequence::tuple;

        pub fn token_name(rest: APSpan<'_>) -> APParserResult<'_, APName<'_>> {
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
                Err(e) => Err(e.into_parser_error().with_code(APCName)),
            }
        }

        pub fn token_name_kurz(rest: APSpan<'_>) -> APParserResult<'_, APName<'_>> {
            match nom_name_kurz(rest) {
                Ok((rest, tok)) => Ok((rest, APName { span: tok })),
                Err(e) => Err(e.into_parser_error().with_code(APCNameKurz)),
            }
        }

        fn cnv_err<C, I, O, E>(
            r: Result<O, E>,
            code: C,
            span: I,
        ) -> Result<O, nom::Err<ParserError<C, I, ()>>>
        where
            C: Code,
            I: Copy,
        {
            match r {
                Ok(v) => Ok(v),
                Err(_) => Err(nom::Err::Failure(ParserError::new(code, span))),
            }
        }

        pub fn token_nummer(rest: APSpan<'_>) -> APParserResult<'_, APNummer<'_>> {
            match nom_number(rest) {
                Ok((rest, tok)) => Ok((
                    rest,
                    APNummer {
                        nummer: cnv_err(tok.parse::<u32>(), APCNummer, tok)?,
                        span: tok,
                    },
                )),
                Err(e) => Err(e.into_parser_error().with_code(APCNummer)),
            }
        }

        pub fn token_menge(rest: APSpan<'_>) -> APParserResult<'_, APMenge<'_>> {
            match nom_number(rest) {
                Ok((rest, tok)) => Ok((
                    rest,
                    APMenge {
                        menge: cnv_err(tok.parse::<i32>(), APCMenge, rest)?,
                        span: tok,
                    },
                )),
                Err(e) => Err(e.into_parser_error().with_code(APCMenge)),
            }
        }

        #[allow(dead_code)]
        pub fn token_datum2(rest: APSpan) -> APParserResult<'_, APDatum<'_>> {
            let k = transform(
                recognize(tuple((
                    nom_number, nom_dot, nom_number, nom_dot, nom_number,
                ))),
                |v: APSpan<'_>| match NaiveDate::parse_from_str(*v, "%d.%m.%Y") {
                    Ok(vv) => Ok(APDatum { datum: vv, span: v }),
                    Err(_) => Err(nom::Err::Failure(APTokenizerError::new(APCDatum, v))),
                },
            )(rest)
            .into_parser_error();

            k
        }

        pub fn token_datum(input: APSpan<'_>) -> APParserResult<'_, APDatum<'_>> {
            let (rest, day) = nom_number(input).with_code(APCDay).into_parser_error()?;
            let (rest, _) = nom_dot(rest).with_code(APCDot).into_parser_error()?;
            let (rest, month) = nom_number(rest).with_code(APCMonth).into_parser_error()?;
            let (rest, _) = nom_dot(rest).with_code(APCDot).into_parser_error()?;
            let (rest, year) = nom_number(rest).with_code(APCYear).into_parser_error()?;

            let iday = cnv_err((*day).parse::<u32>(), APCDay, day)?;
            let imonth = cnv_err((*month).parse::<u32>(), APCMonth, month)?;
            let iyear = cnv_err((*year).parse::<i32>(), APCYear, year)?;

            let span = input.span_union(&day, &year);
            let datum = NaiveDate::from_ymd_opt(iyear, imonth, iday);

            if let Some(datum) = datum {
                Ok((rest, APDatum { datum, span }))
            } else {
                Err(nom::Err::Error(ParserError::new(APCDatum, span)))
            }
        }
    }

    pub mod nom_tokens {
        use crate::planung4::{APSpan, APTokenizerResult};
        use nom::branch::alt;
        use nom::bytes::complete::{tag, tag_no_case, take_till, take_till1, take_while1};
        use nom::character::complete::{char as nchar, one_of};
        use nom::character::complete::{digit1, not_line_ending};
        use nom::combinator::{opt, recognize};
        use nom::multi::many_m_n;
        use nom::sequence::{preceded, terminated, tuple};
        use nom::{AsChar, InputTake, InputTakeAtPosition};

        pub fn lah_plan(i: APSpan<'_>) -> bool {
            tuple((nom_header, tag_no_case("plan")))(i).is_ok()
        }

        pub fn nom_tag_plan(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag_no_case("plan")), nom_ws)(i)
        }

        pub fn nom_metadata(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            recognize(tuple((
                take_till1(|c: char| c == ':' || c == '\n' || c == '\r'),
                nom_colon,
                not_line_ending,
            )))(i)
        }

        pub fn lah_kdnr(i: APSpan<'_>) -> bool {
            tuple((opt(nom_slash_slash), tag_no_case("kdnr")))(i).is_ok()
        }

        pub fn nom_tag_kdnr(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag_no_case("kdnr")), nom_ws)(i)
        }

        pub fn lah_stichtag(i: APSpan<'_>) -> bool {
            tuple((nom_header, tag_no_case("stichtag")))(i).is_ok()
        }

        pub fn nom_tag_stichtag(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag_no_case("stichtag")), nom_ws)(i)
        }

        pub fn lah_bsnr(i: APSpan<'_>) -> bool {
            tuple((opt(nom_slash_slash), tag_no_case("bsnr")))(i).is_ok()
        }

        pub fn nom_tag_bsnr(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag_no_case("bsnr")), nom_ws)(i)
        }

        pub fn lah_monat(i: APSpan<'_>) -> bool {
            tuple((nom_header, tag_no_case("monat")))(i).is_ok()
        }

        pub fn nom_tag_monat(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag_no_case("monat")), nom_ws)(i)
        }

        pub fn lah_woche(i: APSpan<'_>) -> bool {
            tuple((nom_header, tag_no_case("woche")))(i).is_ok()
        }

        pub fn nom_tag_woche(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag_no_case("woche")), nom_ws)(i)
        }

        pub fn lah_tag(i: APSpan<'_>) -> bool {
            tuple((nom_header, tag_no_case("tag")))(i).is_ok()
        }

        pub fn nom_tag_tag(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag_no_case("tag")), nom_ws)(i)
        }

        pub fn lah_aktion(i: APSpan<'_>) -> bool {
            tag::<_, _, nom::error::Error<APSpan<'_>>>("=>")(i).is_ok()
        }

        pub fn nom_tag_aktion(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag("=>")), nom_ws)(i)
        }

        pub fn nom_aktion_aktion(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(
                recognize(alt((
                    tag("Überwintern"),
                    tag("Direktsaat"),
                    tag("Pflanzen"),
                ))),
                nom_ws,
            )(i)
        }

        pub fn lah_pflanzort(i: APSpan<'_>) -> bool {
            alt((
                recognize(nchar::<_, nom::error::Error<APSpan<'_>>>('@')),
                tag_no_case("parzelle"),
            ))(i)
            .is_ok()
        }

        pub fn nom_tag_pflanzort(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(
                recognize(alt((recognize(nchar('@')), tag_no_case("parzelle")))),
                nom_ws,
            )(i)
        }

        pub fn nom_tag_w(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(one_of("wW")), nom_ws)(i)
        }

        pub fn lah_kunde(i: APSpan<'_>) -> bool {
            tuple((opt(nom_star_star), tag_no_case("kunde")))(i).is_ok()
        }

        pub fn lah_lieferant(i: APSpan<'_>) -> bool {
            tuple((opt(nom_star_star), tag_no_case("lieferant")))(i).is_ok()
        }

        pub fn nom_tag_kunde(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag_no_case("kunde")), nom_ws)(i)
        }

        pub fn nom_tag_lieferant(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag_no_case("lieferant")), nom_ws)(i)
        }

        pub fn lah_markt(i: APSpan<'_>) -> bool {
            tuple((opt(nom_star_star), tag_no_case("markt")))(i).is_ok()
        }

        pub fn nom_tag_markt(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag_no_case("markt")), nom_ws)(i)
        }

        pub fn lah_kommentar(i: APSpan<'_>) -> bool {
            nchar::<_, nom::error::Error<APSpan<'_>>>('#')(i).is_ok()
        }

        pub fn nom_kommentar_tag(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag("#")), nom_ws)(i)
        }

        pub fn nom_kommentar(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(take_till(|c: char| c == '\n'), nom_ws)(i)
        }

        pub fn lah_notiz(i: APSpan<'_>) -> bool {
            tag_no_case::<_, _, nom::error::Error<APSpan<'_>>>("##")(i).is_ok()
        }

        pub fn nom_notiz_tag(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tag("##")), nom_ws)(i)
        }

        pub fn nom_notiz(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(take_till(|c: char| c == '\n'), nom_ws)(i)
        }

        pub fn nom_name(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(
                recognize(take_while1(|c: char| {
                    c.is_alphanumeric() || c == ' ' || "\'+-²/_.".contains(c)
                })),
                nom_ws,
            )(i)
        }

        pub fn nom_name_kurz(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(
                recognize(take_while1(|c: char| {
                    c.is_alphanumeric() || "\'+-²/_.".contains(c)
                })),
                nom_ws,
            )(i)
        }

        pub fn nom_kw(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(preceded(tag_no_case("KW"), digit1), nom_ws)(i)
        }

        pub fn lah_number(i: APSpan<'_>) -> bool {
            digit1::<_, nom::error::Error<APSpan<'_>>>(i).is_ok()
        }

        pub fn nom_number(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(digit1, nom_ws)(i)
        }

        pub fn nom_dot(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(nchar('.')), nom_ws)(i)
        }

        pub fn nom_comma(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(nchar(',')), nom_ws)(i)
        }

        pub fn lah_plus(i: APSpan<'_>) -> bool {
            nchar::<_, nom::error::Error<APSpan<'_>>>('+')(i).is_ok()
        }

        pub fn nom_plus(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(nchar('+')), nom_ws)(i)
        }

        pub fn nom_colon(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(nchar(':')), nom_ws)(i)
        }

        pub fn nom_star_star(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tuple((nchar('*'), nchar('*')))), nom_ws)(i)
        }

        pub fn nom_slash_slash(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(tuple((nchar('/'), nchar('/')))), nom_ws)(i)
        }

        pub fn nom_header(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(many_m_n(0, 6, nchar('='))), nom_ws)(i)
        }

        pub fn lah_brop(i: APSpan<'_>) -> bool {
            nchar::<_, nom::error::Error<APSpan<'_>>>('(')(i).is_ok()
        }

        pub fn nom_brop(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(nchar('(')), nom_ws)(i)
        }

        pub fn nom_brcl(i: APSpan<'_>) -> APTokenizerResult<'_, APSpan<'_>> {
            terminated(recognize(nchar(')')), nom_ws)(i)
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

        pub fn nom_ws_nl(i: APSpan<'_>) -> APSpan<'_> {
            match i.split_at_position_complete::<_, nom::error::Error<APSpan<'_>>>(|item| {
                let c = item.as_char();
                !(c == ' ' || c == '\t' || c == '\n' || c == '\r')
            }) {
                Ok((rest, _)) => rest,
                Err(_) => i,
            }
        }

        pub fn nom_is_nl(i: APSpan<'_>) -> bool {
            terminated(
                recognize(take_while1(|c: char| c == '\n' || c == '\r')),
                nom_ws,
            )(i)
            .is_ok()
        }

        pub fn nom_is_comment_or_notiz(i: APSpan<'_>) -> bool {
            terminated(recognize(take_while1(|c: char| c == '#')), nom_ws)(i).is_ok()
        }

        pub fn nom_empty(i: APSpan<'_>) -> APSpan<'_> {
            i.take(0)
        }
    }
}
