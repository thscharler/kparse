use std::fmt::{Display, Formatter};

// pub use diagnostics::{
//     dump_diagnostics as dump_diagnostics_v4, dump_diagnostics_info as dump_diagnostics_info_v4,
//     dump_trace as dump_trace_v4,
// };
use kparse::{Code, ParserNomResult, ParserResult};

#[allow(clippy::enum_variant_names)]
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

pub type Span<'s> = kparse::Span<'s, APCode>;
pub type APParserResult<'s, O> = ParserResult<'s, O, APCode, ()>;
pub type APNomResult<'s> = ParserNomResult<'s, APCode, ()>;

// pub mod diagnostics {
//     use crate::APCode;
//     use iparse::error::DebugWidth;
//     use kparse::debug::tracks::{debug_tracks, Tracks};
//     use kparse::prelude::*;
//     use kparse::{Context, DataFrames, ParserError, StrLines, Track};
//     use std::ffi::OsStr;
//     use std::path::Path;
//
//     /// Write out the Tracer.
//     #[allow(dead_code)]
//     pub fn dump_trace(tracks: &Vec<Track<'_, APCode>>) {
//         println!("{:?}", Tracks(tracks));
//     }
//
//     /// Write some diagnostics.
//     #[allow(clippy::collapsible_else_if)]
//     #[allow(clippy::collapsible_if)]
//     pub fn dump_diagnostics(src: &Path, err: &ParserError<'_, APCode>, msg: &str, is_err: bool) {
//         let txt = StrLines::new(*Context.original(&err.span));
//
//         let text1 = Vec::new();
//         for l in txt.backward_from(*err.span).take(3).rev() {
//             text1.push(Span::new_from_raw_offset())
//         }
//
//         let text1 = get_lines_around(err.span, 3);
//
//         println!();
//         if !msg.is_empty() {
//             println!(
//                 "{}: {:?}: {}",
//                 if is_err { "FEHLER" } else { "Achtung" },
//                 src.file_name().unwrap_or_else(|| OsStr::new("")),
//                 msg
//             );
//         } else {
//             println!(
//                 "{}: {:?}: {}",
//                 if is_err { "FEHLER" } else { "Achtung" },
//                 src.file_name().unwrap_or_else(|| OsStr::new("")),
//                 err.code
//             );
//         }
//
//         let expect = err.expect_grouped_by_line();
//
//         for t in &text1 {
//             if t.location_line() == err.span.location_line() {
//                 println!("*{:04} {}", t.location_line(), t);
//             } else {
//                 println!(" {:04}  {}", t.location_line(), t);
//             }
//
//             if expect.is_empty() {
//                 if t.location_line() == err.span.location_line() {
//                     println!("      {}^", " ".repeat(err.span.get_utf8_column() - 1));
//                     if !msg.is_empty() {
//                         println!("Erwarted war: {}", msg);
//                     } else {
//                         println!("Erwarted war: {}", err.code);
//                     }
//                 }
//             }
//
//             for (line, exp) in &expect {
//                 if t.location_line() == *line {
//                     for exp in exp {
//                         println!("      {}^", " ".repeat(exp.span.get_utf8_column() - 1));
//                         println!("Erwarted war: {}", exp.code);
//                     }
//                 }
//             }
//         }
//
//         for (_line, sugg) in err.suggest_grouped_by_line() {
//             for sug in sugg {
//                 println!("Hinweis: {}", sug.code);
//             }
//         }
//
//         for n in err.nom() {
//             println!(
//                 "Parser-Details: {:?} {}:{}:\"{}\"",
//                 n.kind,
//                 n.span.location_line(),
//                 n.span.get_utf8_column(),
//                 restrict_n(60, n.span)
//             );
//         }
//     }
//
//     /// Write some diagnostics.
//     pub fn dump_diagnostics_info(src: &Path, err: &ParserError<'_, APCode>, msg: &str) {
//         let text1 = get_lines_around(err.span, 0);
//
//         println!();
//         if !msg.is_empty() {
//             println!(
//                 "Achtung: {:?}: {}",
//                 src.file_name().unwrap_or_else(|| OsStr::new("")),
//                 msg
//             );
//         } else {
//             println!(
//                 "Achtung: {:?}: {}",
//                 src.file_name().unwrap_or_else(|| OsStr::new("")),
//                 err.code
//             );
//         }
//
//         for t in &text1 {
//             if t.location_line() == err.span.location_line() {
//                 println!("*{:04} {}", t.location_line(), t);
//             } else {
//                 println!(" {:04}  {}", t.location_line(), t);
//             }
//
//             if t.location_line() == err.span.location_line() {
//                 println!("      {}^", " ".repeat(err.span.get_utf8_column() - 1));
//             }
//         }
//     }
// }

pub mod ast {
    use crate::APCode::*;
    use crate::Span;
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub aktion: Span<'s>,
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub span: Span<'s>,
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
        pub tag: Span<'s>,
        pub span: Span<'s>,
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
        pub tag: Span<'s>,
        pub span: Span<'s>,
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
    use crate::ast::{
        APAktion, APAnbauPlan, APBsNr, APEinheit, APKdNr, APKommentar, APKultur, APKunde, APMarkt,
        APMonat, APPflanzort, APPlan, APSorte, APSorten, APStichtag, APTag, APWoche, APWochen,
        PlanValues,
    };
    use crate::ast::{APLieferant, APNotiz};
    use crate::nom_tokens::*;
    use crate::tokens::{token_datum, token_menge, token_name, token_name_kurz, token_nummer};
    use crate::APCode::*;
    use crate::{nom_tokens, APParserResult, Span};
    use kparse::prelude::*;
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

    fn parse(rest: Span<'_>) -> APParserResult<'_, APAnbauPlan<'_>> {
        Context.enter(APCAnbauplan, &rest);

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

            // todo: continue after error ...

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

    fn lah_plan(span: Span<'_>) -> bool {
        nom_tokens::lah_plan(span)
    }

    fn parse_plan(rest: Span<'_>) -> APParserResult<'_, APPlan<'_>> {
        Context.enter(APCPlan, &rest);

        let (rest, h0) = nom_header(rest).track_as(APCHeader)?;
        let (rest, _) = nom_tag_plan(rest).track_as(APCPlan)?;
        let (rest, plan) = token_name(rest).track()?;
        let (rest, h1) = nom_header(rest).track_as(APCHeader)?;

        let span = unsafe { Context.span_union(&h0, &h1) };
        Context.ok(rest, span, APPlan { name: plan, span })
    }

    fn lah_kdnr(span: Span<'_>) -> bool {
        nom_tokens::lah_kdnr(span)
    }

    fn parse_kdnr(rest: Span<'_>) -> APParserResult<'_, APKdNr<'_>> {
        Context.enter(APCKdNr, &rest);

        let (rest, _) = opt(nom_star_star)(rest).track()?;
        let (rest, _) = opt(nom_slash_slash)(rest).track()?;

        let (rest, tag) = nom_tag_kdnr(rest).track()?;
        let (rest, kdnr) = token_nummer(rest).track()?;

        let (rest, _) = opt(nom_star_star)(rest).track()?;
        let (rest, _) = opt(nom_slash_slash)(rest).track()?;

        let span = unsafe { Context.span_union(&tag, &kdnr.span) };
        Context.ok(rest, span, APKdNr { kdnr, span })
    }

    fn lah_stichtag(span: Span<'_>) -> bool {
        nom_tokens::lah_stichtag(span)
    }

    fn parse_stichtag(rest: Span<'_>) -> APParserResult<'_, APStichtag<'_>> {
        Context.enter(APCStichtag, &rest);

        let (rest, h0) = nom_header(rest).track()?;
        let (rest, _) = nom_tag_stichtag(rest).track()?;
        let (rest, stichtag) = token_datum(rest).track()?;

        let (rest, _brop) = opt(nom_brop)(rest).track()?;
        let (rest, _kw) = opt(nom_kw)(rest).track()?;
        let (rest, _brcl) = opt(nom_brcl)(rest).track()?;

        let (rest, h1) = nom_header(rest).track()?;

        let span = unsafe { Context.span_union(&h0, &h1) };
        Context.ok(rest, span, APStichtag { stichtag, span })
    }

    fn lah_bsnr(span: Span<'_>) -> bool {
        nom_tokens::lah_bsnr(span)
    }

    fn parse_bsnr(rest: Span<'_>) -> APParserResult<'_, APBsNr<'_>> {
        Context.enter(APCBsNr, &rest);

        let (rest, _) = opt(nom_slash_slash)(rest).track()?;
        let (rest, _) = opt(nom_star_star)(rest).track()?;

        let (rest, tag) = nom_tag_bsnr(rest).track()?;
        let (rest, bsnr) = token_nummer(rest).track()?;

        let (rest, _) = opt(nom_star_star)(rest).track()?;
        let (rest, _) = opt(nom_slash_slash)(rest).track()?;

        let span = unsafe { Context.span_union(&tag, &bsnr.span) };
        Context.ok(rest, span, APBsNr { bsnr, span })
    }

    fn lah_monat(span: Span<'_>) -> bool {
        nom_tokens::lah_monat(span)
    }

    fn parse_monat(rest: Span<'_>) -> APParserResult<'_, APMonat<'_>> {
        Context.enter(APCMonat, &rest);

        let (rest, h0) = nom_header(rest).track()?;
        let (rest, _) = nom_tag_monat(rest).track()?;
        let (rest, monat) = token_name(rest).track()?;
        let (rest, h1) = nom_header(rest).track()?;

        let span = unsafe { Context.span_union(&h0, &h1) };
        Context.ok(rest, span, APMonat { monat, span })
    }

    fn lah_woche(span: Span<'_>) -> bool {
        nom_tokens::lah_woche(span)
    }

    fn parse_woche(rest: Span<'_>) -> APParserResult<'_, APWoche<'_>> {
        Context.enter(APCWoche, &rest);

        let (rest, h0) = nom_header(rest).track()?;

        let (rest, _) = nom_tag_woche(rest).track()?;
        let (rest, datum) = token_datum(rest).track()?;

        let (rest, _brop) = opt(nom_brop)(rest).track()?;
        let (rest, _kw) = opt(nom_kw)(rest).track()?;
        let (rest, _brcl) = opt(nom_brcl)(rest).track()?;

        let (rest, h1) = nom_header(rest).track()?;

        let span = unsafe { Context.span_union(&h0, &h1) };
        Context.ok(rest, span, APWoche { datum, span })
    }

    fn lah_tag(span: Span<'_>) -> bool {
        nom_tokens::lah_tag(span)
    }

    fn parse_tag(rest: Span<'_>) -> APParserResult<'_, APTag<'_>> {
        Context.enter(APCTag, &rest);

        let (rest, h0) = nom_header(rest).track()?;
        let (rest, _) = nom_tag_tag(rest).track()?;
        let (rest, tage) = token_nummer(rest).track()?;
        let (rest, h1) = nom_header(rest).track()?;

        let span = unsafe { Context.span_union(&h0, &h1) };
        Context.ok(rest, span, APTag { tage, span })
    }

    fn lah_aktion(span: Span<'_>) -> bool {
        nom_tokens::lah_aktion(span)
    }

    fn parse_aktion(rest: Span<'_>) -> APParserResult<'_, APAktion<'_>> {
        Context.enter(APCAktion, &rest);

        let (rest, tag) = nom_tag_aktion(rest).track()?;
        let (rest, aktion) = nom_aktion_aktion(rest).track_as(APCAktionTyp)?;

        let span = unsafe { Context.span_union(&tag, &aktion) };
        Context.ok(rest, span, APAktion { aktion, span })
    }

    fn lah_pflanzort(span: Span<'_>) -> bool {
        nom_tokens::lah_pflanzort(span)
    }

    fn parse_pflanzort(rest: Span<'_>) -> APParserResult<'_, APPflanzort<'_>> {
        Context.enter(APCPflanzort, &rest);

        let (rest, tag) = nom_tag_pflanzort(rest).track()?;

        let (rest, ort) = token_name_kurz(rest).track()?;

        let (rest, kultur) = if !lah_brop(rest) && !lah_pluswochen(rest) && !lah_wochen(rest) {
            opt(token_name_kurz)(rest).track()?
        } else {
            (rest, None)
        };

        let (rest, brop) = opt(nom_brop)(rest).track()?;
        let (rest, start) = opt(parse_wochen)(rest).track()?;
        let (rest, dauer) = opt(parse_pluswochen)(rest).track()?;
        let (rest, brcl) = opt(nom_brcl)(rest).track()?;

        let span = if let Some(brcl) = brcl {
            unsafe { Context.span_union(&tag, &brcl) }
        } else if let Some(dauer) = &dauer {
            unsafe { Context.span_union(&tag, &dauer.span) }
        } else if let Some(start) = &start {
            unsafe { Context.span_union(&tag, &start.span) }
        } else if let Some(brop) = brop {
            unsafe { Context.span_union(&tag, &brop) }
        } else if let Some(kultur) = &kultur {
            unsafe { Context.span_union(&tag, &kultur.span) }
        } else {
            unsafe { Context.span_union(&tag, &ort.span) }
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

    fn lah_wochen(i: Span<'_>) -> bool {
        tuple((nom_number, nom_tag_w))(i).is_ok()
    }

    fn parse_wochen(rest: Span<'_>) -> APParserResult<'_, APWochen<'_>> {
        Context.enter(APCWochen, &rest);

        let (rest, wochen) = token_nummer(rest).track()?;
        let (rest, w) = nom_tag_w(rest).track()?;

        let span = unsafe { Context.span_union(&wochen.span, &w) };
        Context.ok(rest, span, APWochen { wochen, span })
    }

    fn lah_pluswochen(rest: Span<'_>) -> bool {
        lah_plus(rest)
    }

    fn parse_pluswochen(rest: Span<'_>) -> APParserResult<'_, APWochen<'_>> {
        Context.enter(APCPlusWochen, &rest);

        let (rest, _) = nom_plus(rest).track()?;
        let (rest, wochen) = token_nummer(rest).track()?;
        let (rest, w) = nom_tag_w(rest).track()?;

        let span = unsafe { Context.span_union(&wochen.span, &w) };
        Context.ok(rest, span, APWochen { wochen, span })
    }

    fn lah_kunde(span: Span<'_>) -> bool {
        nom_tokens::lah_kunde(span)
    }

    fn parse_kunde(rest: Span<'_>) -> APParserResult<'_, APKunde<'_>> {
        Context.enter(APCKunde, &rest);

        let (rest, _) = opt(nom_star_star)(rest).track()?;
        let (rest, tag) = nom_tag_kunde(rest).track()?;
        let (rest, name) = token_name(rest).track()?;
        let (rest, _) = opt(nom_star_star)(rest).track()?;

        let span = unsafe { Context.span_union(&tag, &name.span) };

        Context.ok(rest, span, APKunde { name, span })
    }

    fn lah_lieferant(span: Span<'_>) -> bool {
        nom_tokens::lah_lieferant(span)
    }

    fn parse_lieferant(rest: Span<'_>) -> APParserResult<'_, APLieferant<'_>> {
        Context.enter(APCLieferant, &rest);

        let (rest, _) = opt(nom_star_star)(rest).track()?;
        let (rest, tag) = nom_tag_lieferant(rest).track()?;
        let (rest, name) = token_name(rest).track()?;
        let (rest, _) = opt(nom_star_star)(rest).track()?;

        let span = unsafe { Context.span_union(&tag, &name.span) };

        Context.ok(rest, span, APLieferant { name, span })
    }

    fn lah_markt(span: Span<'_>) -> bool {
        nom_tokens::lah_markt(span)
    }

    fn parse_markt(rest: Span<'_>) -> APParserResult<'_, APMarkt<'_>> {
        Context.enter(APCMarkt, &rest);

        let (rest, _) = opt(nom_star_star)(rest).track()?;
        let (rest, tag) = nom_tag_markt(rest).track()?;
        let (rest, name) = token_name(rest).track()?;
        let (rest, _) = opt(nom_star_star)(rest).track()?;

        let span = unsafe { Context.span_union(&tag, &name.span) };

        Context.ok(rest, span, APMarkt { name, span })
    }

    fn parse_kultur(rest: Span<'_>) -> APParserResult<'_, APKultur<'_>> {
        Context.enter(APCKultur, &rest);

        let (rest, kultur) = token_name(rest).track()?;

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
            Err(e) => return Context.err(e),
        };

        // must be at line end now, and can eat some whitespace
        let rest = if !nom_is_nl(rest) {
            return Context.err(ParserError::new(APCSorten, rest));
        } else {
            nom_ws_nl(rest)
        };

        let span = unsafe { Context.span_union(&kultur.span, &sorten.span) };

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

    fn parse_einheit(rest: Span<'_>) -> APParserResult<'_, APEinheit<'_>> {
        Context.enter(APCEinheit, &rest);

        let (rest, brop) = nom_brop(rest).track_as(APCBracketOpen)?;
        let (rest, name) = token_name(rest)?;
        let (rest, brcl) = nom_brcl(rest).track_as(APCBracketClose)?;

        let span = unsafe { Context.span_union(&brop, &brcl) };

        Context.ok(
            rest,
            span,
            APEinheit {
                einheit: name,
                span,
            },
        )
    }

    fn parse_sorten(rest: Span<'_>) -> APParserResult<'_, APSorten<'_>> {
        Context.enter(APCSorten, &rest);

        let mut sorten = Vec::new();

        let mut rest_loop = rest;
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
                unsafe { Context.span_union(&first.span, &last.span) }
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

    fn parse_sorte(rest: Span<'_>) -> APParserResult<'_, APSorte<'_>> {
        Context.enter(APCSorte, &rest);

        let (rest, menge) = token_menge(rest).track()?;
        let (rest, name) = token_name(rest).track()?;

        let span = unsafe { Context.span_union(&menge.span, &name.span) };

        Context.ok(rest, span, APSorte { menge, name, span })
    }

    fn lah_kommentar(span: Span<'_>) -> bool {
        nom_tokens::lah_kommentar(span)
    }

    fn parse_kommentar(rest: Span<'_>) -> APParserResult<'_, APKommentar<'_>> {
        Context.enter(APCKommentar, &rest);

        let (rest, kommentar_tag) = nom_kommentar_tag(rest).track()?;
        let (rest, kommentar) = nom_kommentar(rest).track()?;

        Context.ok(
            rest,
            kommentar,
            APKommentar {
                tag: kommentar_tag,
                span: kommentar,
            },
        )
    }

    fn lah_notiz(span: Span<'_>) -> bool {
        nom_tokens::lah_notiz(span)
    }

    fn parse_notiz(rest: Span<'_>) -> APParserResult<'_, APNotiz<'_>> {
        Context.enter(APCNotiz, &rest);

        let (rest, notiz_tag) = nom_notiz_tag(rest).track()?;
        let (rest, notiz) = nom_notiz(rest).track()?;

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
    use crate::ast::{APDatum, APMenge, APName, APNummer};
    use crate::nom_tokens::{nom_dot, nom_name, nom_name_kurz, nom_number};
    use crate::APCode::*;
    use crate::{APParserResult, Span};
    use kparse::prelude::*;

    pub fn token_name(rest: Span<'_>) -> APParserResult<'_, APName<'_>> {
        match nom_name(rest) {
            Ok((rest, tok)) => {
                // trim trailing whitespace after the fact.
                let trim = tok.trim_end();

                // the trimmed span is part of original.
                // so reusing the rest ought to be fine.
                let tok = unsafe {
                    Span::new_from_raw_offset(
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

    pub fn token_name_kurz(rest: Span<'_>) -> APParserResult<'_, APName<'_>> {
        match nom_name_kurz(rest) {
            Ok((rest, tok)) => Ok((rest, APName { span: tok })),
            Err(e) => Err(e.with_code(APCNameKurz)),
        }
    }

    pub fn token_nummer(rest: Span<'_>) -> APParserResult<'_, APNummer<'_>> {
        match nom_number(rest) {
            Ok((rest, tok)) => Ok((
                rest,
                APNummer {
                    nummer: tok.parse::<u32>().with_span(APCNummer, rest)?,
                    span: tok,
                },
            )),
            Err(e) => Err(e.with_code(APCNummer)),
        }
    }

    pub fn token_menge(rest: Span<'_>) -> APParserResult<'_, APMenge<'_>> {
        match nom_number(rest) {
            Ok((rest, tok)) => Ok((
                rest,
                APMenge {
                    menge: tok.parse::<i32>().with_span(APCMenge, rest)?,
                    span: tok,
                },
            )),
            Err(e) => Err(e.with_code(APCMenge)),
        }
    }

    pub fn token_datum(rest: Span<'_>) -> APParserResult<'_, APDatum<'_>> {
        let (rest, day) = nom_number(rest).with_code(APCDay)?;
        let (rest, _) = nom_dot(rest).with_code(APCDot)?;
        let (rest, month) = nom_number(rest).with_code(APCMonth)?;
        let (rest, _) = nom_dot(rest).with_code(APCDot)?;
        let (rest, year) = nom_number(rest).with_code(APCYear)?;

        let iday: u32 = (*day).parse().with_span(APCDay, day)?;
        let imonth: u32 = (*month).parse().with_span(APCMonth, month)?;
        let iyear: i32 = (*year).parse().with_span(APCYear, year)?;

        let span = unsafe { Context.span_union(&day, &year) };
        let datum = chrono::NaiveDate::from_ymd_opt(iyear, imonth, iday);

        if let Some(datum) = datum {
            Ok((rest, APDatum { datum, span }))
        } else {
            Err(nom::Err::Error(ParserError::new(APCDatum, span)))
        }
    }
}

pub mod nom_tokens {
    use crate::{APNomResult, Span};
    use nom::branch::alt;
    use nom::bytes::complete::{tag, tag_no_case, take_till, take_till1, take_while1};
    use nom::character::complete::{char as nchar, one_of};
    use nom::character::complete::{digit1, not_line_ending};
    use nom::combinator::{opt, recognize};
    use nom::multi::many_m_n;
    use nom::sequence::{preceded, terminated, tuple};
    use nom::{AsChar, InputTake, InputTakeAtPosition};

    pub fn lah_plan(i: Span<'_>) -> bool {
        tuple((nom_header, tag_no_case("plan")))(i).is_ok()
    }

    pub fn nom_tag_plan(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag_no_case("plan")), nom_ws)(i)
    }

    pub fn nom_metadata(i: Span<'_>) -> APNomResult<'_> {
        recognize(tuple((
            take_till1(|c: char| c == ':' || c == '\n' || c == '\r'),
            nom_colon,
            not_line_ending,
        )))(i)
    }

    pub fn lah_kdnr(i: Span<'_>) -> bool {
        tuple((opt(nom_slash_slash), tag_no_case("kdnr")))(i).is_ok()
    }

    pub fn nom_tag_kdnr(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag_no_case("kdnr")), nom_ws)(i)
    }

    pub fn lah_stichtag(i: Span<'_>) -> bool {
        tuple((nom_header, tag_no_case("stichtag")))(i).is_ok()
    }

    pub fn nom_tag_stichtag(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag_no_case("stichtag")), nom_ws)(i)
    }

    pub fn lah_bsnr(i: Span<'_>) -> bool {
        tuple((opt(nom_slash_slash), tag_no_case("bsnr")))(i).is_ok()
    }

    pub fn nom_tag_bsnr(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag_no_case("bsnr")), nom_ws)(i)
    }

    pub fn lah_monat(i: Span<'_>) -> bool {
        tuple((nom_header, tag_no_case("monat")))(i).is_ok()
    }

    pub fn nom_tag_monat(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag_no_case("monat")), nom_ws)(i)
    }

    pub fn lah_woche(i: Span<'_>) -> bool {
        tuple((nom_header, tag_no_case("woche")))(i).is_ok()
    }

    pub fn nom_tag_woche(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag_no_case("woche")), nom_ws)(i)
    }

    pub fn lah_tag(i: Span<'_>) -> bool {
        tuple((nom_header, tag_no_case("tag")))(i).is_ok()
    }

    pub fn nom_tag_tag(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag_no_case("tag")), nom_ws)(i)
    }

    pub fn lah_aktion(i: Span<'_>) -> bool {
        tag::<_, _, nom::error::Error<Span<'_>>>("=>")(i).is_ok()
    }

    pub fn nom_tag_aktion(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag("=>")), nom_ws)(i)
    }

    pub fn nom_aktion_aktion(i: Span<'_>) -> APNomResult<'_> {
        terminated(
            recognize(alt((
                tag("Überwintern"),
                tag("Direktsaat"),
                tag("Pflanzen"),
            ))),
            nom_ws,
        )(i)
    }

    pub fn lah_pflanzort(i: Span<'_>) -> bool {
        alt((
            recognize(nchar::<_, nom::error::Error<Span<'_>>>('@')),
            tag_no_case("parzelle"),
        ))(i)
        .is_ok()
    }

    pub fn nom_tag_pflanzort(i: Span<'_>) -> APNomResult<'_> {
        terminated(
            recognize(alt((recognize(nchar('@')), tag_no_case("parzelle")))),
            nom_ws,
        )(i)
    }

    pub fn nom_tag_w(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(one_of("wW")), nom_ws)(i)
    }

    pub fn lah_kunde(i: Span<'_>) -> bool {
        tuple((opt(nom_star_star), tag_no_case("kunde")))(i).is_ok()
    }

    pub fn lah_lieferant(i: Span<'_>) -> bool {
        tuple((opt(nom_star_star), tag_no_case("lieferant")))(i).is_ok()
    }

    pub fn nom_tag_kunde(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag_no_case("kunde")), nom_ws)(i)
    }

    pub fn nom_tag_lieferant(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag_no_case("lieferant")), nom_ws)(i)
    }

    pub fn lah_markt(i: Span<'_>) -> bool {
        tuple((opt(nom_star_star), tag_no_case("markt")))(i).is_ok()
    }

    pub fn nom_tag_markt(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag_no_case("markt")), nom_ws)(i)
    }

    pub fn lah_kommentar(i: Span<'_>) -> bool {
        nchar::<_, nom::error::Error<Span<'_>>>('#')(i).is_ok()
    }

    pub fn nom_kommentar_tag(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag("#")), nom_ws)(i)
    }

    pub fn nom_kommentar(i: Span<'_>) -> APNomResult<'_> {
        terminated(take_till(|c: char| c == '\n'), nom_ws)(i)
    }

    pub fn lah_notiz(i: Span<'_>) -> bool {
        tag_no_case::<_, _, nom::error::Error<Span<'_>>>("##")(i).is_ok()
    }

    pub fn nom_notiz_tag(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tag("##")), nom_ws)(i)
    }

    pub fn nom_notiz(i: Span<'_>) -> APNomResult<'_> {
        terminated(take_till(|c: char| c == '\n'), nom_ws)(i)
    }

    pub fn nom_name(i: Span<'_>) -> APNomResult<'_> {
        terminated(
            recognize(take_while1(|c: char| {
                c.is_alphanumeric() || c == ' ' || "\'+-²/_.".contains(c)
            })),
            nom_ws,
        )(i)
    }

    pub fn nom_name_kurz(i: Span<'_>) -> APNomResult<'_> {
        terminated(
            recognize(take_while1(|c: char| {
                c.is_alphanumeric() || "\'+-²/_.".contains(c)
            })),
            nom_ws,
        )(i)
    }

    pub fn nom_kw(i: Span<'_>) -> APNomResult<'_> {
        terminated(preceded(tag_no_case("KW"), digit1), nom_ws)(i)
    }

    pub fn lah_number(i: Span<'_>) -> bool {
        digit1::<_, nom::error::Error<Span<'_>>>(i).is_ok()
    }

    pub fn nom_number(i: Span<'_>) -> APNomResult<'_> {
        terminated(digit1, nom_ws)(i)
    }

    pub fn nom_dot(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(nchar('.')), nom_ws)(i)
    }

    pub fn nom_comma(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(nchar(',')), nom_ws)(i)
    }

    pub fn lah_plus(i: Span<'_>) -> bool {
        nchar::<_, nom::error::Error<Span<'_>>>('+')(i).is_ok()
    }

    pub fn nom_plus(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(nchar('+')), nom_ws)(i)
    }

    pub fn nom_colon(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(nchar(':')), nom_ws)(i)
    }

    pub fn nom_star_star(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tuple((nchar('*'), nchar('*')))), nom_ws)(i)
    }

    pub fn nom_slash_slash(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(tuple((nchar('/'), nchar('/')))), nom_ws)(i)
    }

    pub fn nom_header(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(many_m_n(0, 6, nchar('='))), nom_ws)(i)
    }

    pub fn lah_brop(i: Span<'_>) -> bool {
        nchar::<_, nom::error::Error<Span<'_>>>('(')(i).is_ok()
    }

    pub fn nom_brop(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(nchar('(')), nom_ws)(i)
    }

    pub fn nom_brcl(i: Span<'_>) -> APNomResult<'_> {
        terminated(recognize(nchar(')')), nom_ws)(i)
    }

    pub fn nom_ws(i: Span<'_>) -> APNomResult<'_> {
        i.split_at_position_complete(|item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t')
        })
    }

    pub fn nom_ws2(i: Span<'_>) -> Span<'_> {
        match i.split_at_position_complete::<_, nom::error::Error<Span<'_>>>(|item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t')
        }) {
            Ok((rest, _)) => rest,
            Err(_) => i,
        }
    }

    pub fn nom_ws_nl(i: Span<'_>) -> Span<'_> {
        match i.split_at_position_complete::<_, nom::error::Error<Span<'_>>>(|item| {
            let c = item.as_char();
            !(c == ' ' || c == '\t' || c == '\n' || c == '\r')
        }) {
            Ok((rest, _)) => rest,
            Err(_) => i,
        }
    }

    pub fn nom_is_nl(i: Span<'_>) -> bool {
        terminated(
            recognize(take_while1(|c: char| c == '\n' || c == '\r')),
            nom_ws,
        )(i)
        .is_ok()
    }

    pub fn nom_is_comment_or_notiz(i: Span<'_>) -> bool {
        terminated(recognize(take_while1(|c: char| c == '#')), nom_ws)(i).is_ok()
    }

    pub fn nom_empty(i: Span<'_>) -> Span<'_> {
        i.take(0)
    }
}