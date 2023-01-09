extern crate iparse;

use iparse::{Code, ParserNomResult, ParserResult, Span};
use std::fmt::{Display, Formatter};

pub use diagnostics::{
    dump_diagnostics as dump_diagnostics_v4, dump_diagnostics_info as dump_diagnostics_info_v4,
    dump_trace as dump_trace_v4,
};

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
    const NOM_FAILURE: Self = Self::APCNomFailure;
    const PARSE_INCOMPLETE: Self = Self::APCParseIncomplete;
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

pub type APParserResult<'s, O> = ParserResult<'s, APCode, (Span<'s>, O)>;
pub type APNomResult<'s> = ParserNomResult<'s, APCode>;

pub mod diagnostics {
    use crate::APCode;
    use iparse::error::{DebugWidth, ParserError};
    use iparse::restrict_n;
    use iparse::span::get_lines_around;
    use iparse::tracer::CTracer;
    use std::ffi::OsStr;
    use std::path::Path;

    /// Write out the Tracer.
    #[allow(dead_code)]
    pub fn dump_trace(trace: &CTracer<'_, APCode, true>) {
        let mut buf = String::new();
        let _ = trace.write(&mut buf, DebugWidth::Medium, &|_v| true);
        println!("{}", buf);
    }

    /// Write some diagnostics.
    #[allow(clippy::collapsible_else_if)]
    #[allow(clippy::collapsible_if)]
    pub fn dump_diagnostics(src: &Path, err: &ParserError<'_, APCode>, msg: &str, is_err: bool) {
        let text1 = get_lines_around(err.span, 3);

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

        let expect = err.expect_grouped_by_line();

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

        for (_line, sugg) in err.suggest_grouped_by_line() {
            for sug in sugg {
                println!("Hinweis: {}", sug.code);
            }
        }

        for n in err.nom() {
            println!(
                "Parser-Details: {:?} {}:{}:\"{}\"",
                n.kind,
                n.span.location_line(),
                n.span.get_utf8_column(),
                restrict_n(60, n.span)
            );
        }
    }

    /// Write some diagnostics.
    pub fn dump_diagnostics_info(src: &Path, err: &ParserError<'_, APCode>, msg: &str) {
        let text1 = get_lines_around(err.span, 0);

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
    use crate::APCode::*;
    use chrono::NaiveDate;
    use iparse::Span;
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
pub mod parser {
    use crate::ast::{
        APAktion, APAnbauPlan, APBsNr, APEinheit, APKdNr, APKommentar, APKultur, APKunde, APMarkt,
        APMonat, APPflanzort, APPlan, APSorte, APSorten, APStichtag, APTag, APWoche, APWochen,
        PlanValues,
    };
    use crate::ast::{APLieferant, APNotiz};
    use crate::nom_tokens::*;
    use crate::tokens::{token_datum, token_menge, token_name, token_name_kurz, token_nummer};
    use crate::APCode;
    use crate::APCode::*;
    use iparse::error::ParserError;
    use iparse::span::span_union;
    use iparse::{
        IntoParserResultAddSpan, ParseAsOptional, Parser, ParserResult, Span, Tracer,
        TrackParseResult,
    };
    use nom::sequence::tuple;
    use std::num::ParseIntError;

    impl<'s, T> IntoParserResultAddSpan<'s, APCode, T> for Result<T, ParseIntError> {
        fn into_with_span(self, span: Span<'s>) -> ParserResult<'s, APCode, T> {
            match self {
                Ok(v) => Ok(v),
                Err(_) => Err(ParserError::new(APCInteger, span)),
            }
        }
    }

    pub struct ParseAnbauPlan;

    impl<'s> Parser<'s, APAnbauPlan<'s>, APCode> for ParseAnbauPlan {
        fn id() -> APCode {
            APCAnbauplan
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APAnbauPlan<'s>)> {
            trace.enter(Self::id(), rest);

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

            let (rest, plan) = ParsePlan::parse(trace, rest).track(trace)?;
            let rest = nom_ws_nl(rest);

            let (rest, kdnr) = ParseKdNr::parse(trace, rest).optional().track(trace)?;
            let rest = nom_ws_nl(rest);

            let mut data = Vec::new();
            let mut loop_rest = rest;
            loop {
                let rest2 = loop_rest;

                // todo: continue after error ...

                let rest2 = if ParseStichtag::lah(rest2) {
                    let (rest3, val) = ParseStichtag::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Stichtag(val));
                    rest3
                } else if ParseBsNr::lah(rest2) {
                    let (rest3, val) = ParseBsNr::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::BsNr(val));
                    rest3
                } else if ParseMonat::lah(rest2) {
                    let (rest3, val) = ParseMonat::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Monat(val));
                    rest3
                } else if ParseWoche::lah(rest2) {
                    let (rest3, val) = ParseWoche::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Woche(val));
                    rest3
                } else if ParseTag::lah(rest2) {
                    let (rest3, val) = ParseTag::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Tag(val));
                    rest3
                } else if ParseNotiz::lah(rest2) {
                    let (rest3, val) = ParseNotiz::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Notiz(val));
                    rest3
                } else if ParseKommentar::lah(rest2) {
                    let (rest3, val) = ParseKommentar::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Kommentar(val));
                    rest3
                } else if ParseKunde::lah(rest2) {
                    let (rest3, val) = ParseKunde::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Kunde(val));
                    rest3
                } else if ParseLieferant::lah(rest2) {
                    let (rest3, val) = ParseLieferant::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Lieferant(val));
                    rest3
                } else if ParseMarkt::lah(rest2) {
                    let (rest3, val) = ParseMarkt::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Markt(val));
                    rest3
                } else if ParseAktion::lah(rest2) {
                    let (rest3, val) = ParseAktion::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Aktion(val));
                    rest3
                } else if ParsePflanzort::lah(rest2) {
                    let (rest3, val) = ParsePflanzort::parse(trace, rest2).track(trace)?;
                    data.push(PlanValues::Pflanzort(val));
                    rest3
                } else if rest2.len() == 0 {
                    // eof shouldn't try to parse as kultur
                    rest2
                } else {
                    let (rest3, val) = ParseKultur::parse(trace, rest2).track(trace)?;
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

            trace.ok(rest, rest, APAnbauPlan { plan, kdnr, data })
        }
    }

    pub struct ParsePlan;

    impl<'s> Parser<'s, APPlan<'s>, APCode> for ParsePlan {
        fn id() -> APCode {
            APCPlan
        }

        fn lah(span: Span<'s>) -> bool {
            lah_plan(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APPlan<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, h0) = nom_header(rest).track_as(trace, APCHeader)?;
            let (rest, _) = nom_tag_plan(rest).track_as(trace, APCPlan)?;
            let (rest, plan) = token_name(rest).track(trace)?;
            let (rest, h1) = nom_header(rest).track_as(trace, APCHeader)?;

            let span = span_union(h0, h1);
            trace.ok(rest, span, APPlan { name: plan, span })
        }
    }

    pub struct ParseKdNr;

    impl<'s> Parser<'s, APKdNr<'s>, APCode> for ParseKdNr {
        fn id() -> APCode {
            APCKdNr
        }

        fn lah(span: Span<'s>) -> bool {
            lah_kdnr(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APKdNr<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, _) = nom_star_star(rest).optional().track(trace)?;
            let (rest, _) = nom_slash_slash(rest).optional().track(trace)?;

            let (rest, tag) = nom_tag_kdnr(rest).track(trace)?;
            let (rest, kdnr) = token_nummer(rest).track(trace)?;

            let (rest, _) = nom_star_star(rest).optional().track(trace)?;
            let (rest, _) = nom_slash_slash(rest).optional().track(trace)?;

            let span = span_union(tag, kdnr.span);
            trace.ok(rest, span, APKdNr { kdnr, span })
        }
    }

    pub struct ParseStichtag;

    impl<'s> Parser<'s, APStichtag<'s>, APCode> for ParseStichtag {
        fn id() -> APCode {
            APCStichtag
        }

        fn lah(span: Span<'s>) -> bool {
            lah_stichtag(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APStichtag<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, h0) = nom_header(rest).track(trace)?;
            let (rest, _) = nom_tag_stichtag(rest).track(trace)?;
            let (rest, stichtag) = token_datum(rest).track(trace)?;

            let (rest, _brop) = nom_brop(rest).optional().track(trace)?;
            let (rest, _kw) = nom_kw(rest).optional().track(trace)?;
            let (rest, _brcl) = nom_brcl(rest).optional().track(trace)?;

            let (rest, h1) = nom_header(rest).track(trace)?;

            let span = span_union(h0, h1);
            trace.ok(rest, span, APStichtag { stichtag, span })
        }
    }

    pub struct ParseBsNr;

    impl<'s> Parser<'s, APBsNr<'s>, APCode> for ParseBsNr {
        fn id() -> APCode {
            APCBsNr
        }

        fn lah(span: Span<'s>) -> bool {
            lah_bsnr(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APBsNr<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, _) = nom_slash_slash(rest).optional().track(trace)?;
            let (rest, _) = nom_star_star(rest).optional().track(trace)?;

            let (rest, tag) = nom_tag_bsnr(rest).track(trace)?;
            let (rest, bsnr) = token_nummer(rest).track(trace)?;

            let (rest, _) = nom_star_star(rest).optional().track(trace)?;
            let (rest, _) = nom_slash_slash(rest).optional().track(trace)?;

            let span = span_union(tag, bsnr.span);
            trace.ok(rest, span, APBsNr { bsnr, span })
        }
    }

    pub struct ParseMonat;

    impl<'s> Parser<'s, APMonat<'s>, APCode> for ParseMonat {
        fn id() -> APCode {
            APCMonat
        }

        fn lah(span: Span<'s>) -> bool {
            lah_monat(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APMonat<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, h0) = nom_header(rest).track(trace)?;
            let (rest, _) = nom_tag_monat(rest).track(trace)?;
            let (rest, monat) = token_name(rest).track(trace)?;
            let (rest, h1) = nom_header(rest).track(trace)?;

            let span = span_union(h0, h1);
            trace.ok(rest, span, APMonat { monat, span })
        }
    }

    pub struct ParseWoche;

    impl<'s> Parser<'s, APWoche<'s>, APCode> for ParseWoche {
        fn id() -> APCode {
            APCWoche
        }

        fn lah(span: Span<'s>) -> bool {
            lah_woche(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APWoche<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, h0) = nom_header(rest).track(trace)?;

            let (rest, _) = nom_tag_woche(rest).track(trace)?;
            let (rest, datum) = token_datum(rest).track(trace)?;

            let (rest, _brop) = nom_brop(rest).optional().track(trace)?;
            let (rest, _kw) = nom_kw(rest).optional().track(trace)?;
            let (rest, _brcl) = nom_brcl(rest).optional().track(trace)?;

            let (rest, h1) = nom_header(rest).track(trace)?;

            let span = span_union(h0, h1);
            trace.ok(rest, span, APWoche { datum, span })
        }
    }

    pub struct ParseTag;

    impl<'s> Parser<'s, APTag<'s>, APCode> for ParseTag {
        fn id() -> APCode {
            APCTag
        }

        fn lah(span: Span<'s>) -> bool {
            lah_tag(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APTag<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, h0) = nom_header(rest).track(trace)?;
            let (rest, _) = nom_tag_tag(rest).track(trace)?;
            let (rest, tage) = token_nummer(rest).track(trace)?;
            let (rest, h1) = nom_header(rest).track(trace)?;

            let span = span_union(h0, h1);
            trace.ok(rest, span, APTag { tage, span })
        }
    }

    pub struct ParseAktion;

    impl<'s> Parser<'s, APAktion<'s>, APCode> for ParseAktion {
        fn id() -> APCode {
            APCAktion
        }

        fn lah(span: Span<'s>) -> bool {
            lah_aktion(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APAktion<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, tag) = nom_tag_aktion(rest).track(trace)?;
            let (rest, aktion) = nom_aktion_aktion(rest).track_as(trace, APCAktionTyp)?;

            let span = span_union(tag, aktion);
            trace.ok(rest, span, APAktion { aktion, span })
        }
    }

    pub struct ParsePflanzort;

    impl<'s> Parser<'s, APPflanzort<'s>, APCode> for ParsePflanzort {
        fn id() -> APCode {
            APCPflanzort
        }

        fn lah(span: Span<'s>) -> bool {
            lah_pflanzort(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APPflanzort<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, tag) = nom_tag_pflanzort(rest).track(trace)?;

            let (rest, ort) = token_name_kurz(rest).track(trace)?;

            let (rest, kultur) =
                if !lah_brop(rest) && !ParsePlusWochen::lah(rest) && !ParseWochen::lah(rest) {
                    token_name_kurz(rest).optional().track(trace)?
                } else {
                    (rest, None)
                };

            let (rest, brop) = nom_brop(rest).optional().track(trace)?;
            let (rest, start) = ParseWochen::parse(trace, rest).optional().track(trace)?;
            let (rest, dauer) = ParsePlusWochen::parse(trace, rest)
                .optional()
                .track(trace)?;
            let (rest, brcl) = nom_brcl(rest).optional().track(trace)?;

            let span = if let Some(brcl) = brcl {
                span_union(tag, brcl)
            } else if let Some(dauer) = &dauer {
                span_union(tag, dauer.span)
            } else if let Some(start) = &start {
                span_union(tag, start.span)
            } else if let Some(brop) = brop {
                span_union(tag, brop)
            } else if let Some(kultur) = &kultur {
                span_union(tag, kultur.span)
            } else {
                span_union(tag, ort.span)
            };

            trace.ok(
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
    }

    pub struct ParseWochen;

    impl<'s> Parser<'s, APWochen<'s>, APCode> for ParseWochen {
        fn id() -> APCode {
            APCWochen
        }

        fn lah(i: Span<'s>) -> bool {
            tuple((nom_number, nom_tag_w))(i).is_ok()
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APWochen<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, wochen) = token_nummer(rest).track(trace)?;
            let (rest, w) = nom_tag_w(rest).track(trace)?;

            let span = span_union(wochen.span, w);
            trace.ok(rest, span, APWochen { wochen, span })
        }
    }

    struct ParsePlusWochen;

    impl<'s> Parser<'s, APWochen<'s>, APCode> for ParsePlusWochen {
        fn id() -> APCode {
            APCPlusWochen
        }

        fn lah(rest: Span<'s>) -> bool {
            lah_plus(rest)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APWochen<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, _) = nom_plus(rest).track(trace)?;
            let (rest, wochen) = token_nummer(rest).track(trace)?;
            let (rest, w) = nom_tag_w(rest).track(trace)?;

            let span = span_union(wochen.span, w);
            trace.ok(rest, span, APWochen { wochen, span })
        }
    }

    pub struct ParseKunde;

    impl<'s> Parser<'s, APKunde<'s>, APCode> for ParseKunde {
        fn id() -> APCode {
            APCKunde
        }

        fn lah(span: Span<'s>) -> bool {
            lah_kunde(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APKunde<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, _) = nom_star_star(rest).optional().track(trace)?;
            let (rest, tag) = nom_tag_kunde(rest).track(trace)?;
            let (rest, name) = token_name(rest).track(trace)?;
            let (rest, _) = nom_star_star(rest).optional().track(trace)?;

            let span = span_union(tag, name.span);

            trace.ok(rest, span, APKunde { name, span })
        }
    }

    pub struct ParseLieferant;

    impl<'s> Parser<'s, APLieferant<'s>, APCode> for ParseLieferant {
        fn id() -> APCode {
            APCLieferant
        }

        fn lah(span: Span<'s>) -> bool {
            lah_lieferant(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APLieferant<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, _) = nom_star_star(rest).optional().track(trace)?;
            let (rest, tag) = nom_tag_lieferant(rest).track(trace)?;
            let (rest, name) = token_name(rest).track(trace)?;
            let (rest, _) = nom_star_star(rest).optional().track(trace)?;

            let span = span_union(tag, name.span);

            trace.ok(rest, span, APLieferant { name, span })
        }
    }

    pub struct ParseMarkt;

    impl<'s> Parser<'s, APMarkt<'s>, APCode> for ParseMarkt {
        fn id() -> APCode {
            APCMarkt
        }

        fn lah(span: Span<'s>) -> bool {
            lah_markt(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APMarkt<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, _) = nom_star_star(rest).optional().track(trace)?;
            let (rest, tag) = nom_tag_markt(rest).track(trace)?;
            let (rest, name) = token_name(rest).track(trace)?;
            let (rest, _) = nom_star_star(rest).optional().track(trace)?;

            let span = span_union(tag, name.span);

            trace.ok(rest, span, APMarkt { name, span })
        }
    }

    pub struct ParseKultur;

    impl<'s> Parser<'s, APKultur<'s>, APCode> for ParseKultur {
        fn id() -> APCode {
            APCKultur
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APKultur<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, kultur) = token_name(rest).track(trace)?;

            let (rest, einheit) = ParseEinheit::parse(trace, rest).optional().track(trace)?;

            let (rest, sorten) = match nom_colon(rest).optional() {
                Ok((rest, Some(_colon))) => {
                    //
                    ParseSorten::parse(trace, rest).track(trace)?
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
                Err(e) => return trace.err(e),
            };

            // must be at line end now, and can eat some whitespace
            let rest = if !nom_is_nl(rest) {
                return trace.err(ParserError::new(APCSorten, rest));
            } else {
                nom_ws_nl(rest)
            };

            let span = span_union(kultur.span, sorten.span);

            trace.ok(
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
    }

    pub struct ParseEinheit;

    impl<'s> Parser<'s, APEinheit<'s>, APCode> for ParseEinheit {
        fn id() -> APCode {
            APCEinheit
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APEinheit<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, brop) = nom_brop(rest).track_as(trace, APCBracketOpen)?;
            let (rest, name) = token_name(rest)?;
            let (rest, brcl) = nom_brcl(rest).track_as(trace, APCBracketClose)?;

            let span = span_union(brop, brcl);

            trace.ok(
                rest,
                span,
                APEinheit {
                    einheit: name,
                    span,
                },
            )
        }
    }

    pub struct ParseSorten;

    impl<'s> Parser<'s, APSorten<'s>, APCode> for ParseSorten {
        fn id() -> APCode {
            APCSorten
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APSorten<'s>)> {
            trace.enter(Self::id(), rest);

            let mut sorten = Vec::new();

            let mut rest_loop = rest;
            loop {
                let rest1 = rest_loop;

                let (rest1, sorte) = ParseSorte::parse(trace, rest1).track(trace)?;
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
                                return trace.err(ParserError::new(APCSortenContinue, rest1));
                            }
                        }
                    }
                    Err(_e) => {
                        // no comma, maybe at the line end?
                        if nom_is_nl(rest1) || nom_is_comment_or_notiz(rest1) {
                            // don't eat the new line. that's the job of the caller.
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

            let (rest, notiz) = ParseNotiz::parse(trace, rest).optional().track(trace)?;
            let (rest, kommentar) = ParseKommentar::parse(trace, rest).optional().track(trace)?;

            let first = sorten.first();
            let last = sorten.last();

            let span = if let Some(first) = first {
                if let Some(last) = last {
                    span_union(first.span, last.span)
                } else {
                    unreachable!()
                }
            } else {
                nom_empty(rest)
            };

            trace.ok(
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
    }

    pub struct ParseSorte;

    impl<'s> Parser<'s, APSorte<'s>, APCode> for ParseSorte {
        fn id() -> APCode {
            APCSorte
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APSorte<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, menge) = token_menge(rest).track(trace)?;
            let (rest, name) = token_name(rest).track(trace)?;

            let span = span_union(menge.span, name.span);

            trace.ok(rest, span, APSorte { menge, name, span })
        }
    }

    pub struct ParseKommentar;

    impl<'s> Parser<'s, APKommentar<'s>, APCode> for ParseKommentar {
        fn id() -> APCode {
            APCKommentar
        }

        fn lah(span: Span<'s>) -> bool {
            lah_kommentar(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APKommentar<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, kommentar_tag) = nom_kommentar_tag(rest).track(trace)?;
            let (rest, kommentar) = nom_kommentar(rest).track(trace)?;

            trace.ok(
                rest,
                kommentar,
                APKommentar {
                    tag: kommentar_tag,
                    span: kommentar,
                },
            )
        }
    }

    pub struct ParseNotiz;

    impl<'s> Parser<'s, APNotiz<'s>, APCode> for ParseNotiz {
        fn id() -> APCode {
            APCNotiz
        }

        fn lah(span: Span<'s>) -> bool {
            lah_notiz(span)
        }

        fn parse<'t>(
            trace: &'t mut impl Tracer<'s, APCode>,
            rest: Span<'s>,
        ) -> ParserResult<'s, APCode, (Span<'s>, APNotiz<'s>)> {
            trace.enter(Self::id(), rest);

            let (rest, notiz_tag) = nom_notiz_tag(rest).track(trace)?;
            let (rest, notiz) = nom_notiz(rest).track(trace)?;

            trace.ok(
                rest,
                notiz,
                APNotiz {
                    tag: notiz_tag,
                    span: notiz,
                },
            )
        }
    }
}

pub mod tokens {
    use crate::ast::{APDatum, APMenge, APName, APNummer};
    use crate::nom_tokens::{nom_dot, nom_name, nom_name_kurz, nom_number};
    use crate::APCode::*;
    use crate::APParserResult;
    use iparse::error::ParserError;
    use iparse::span::span_union;
    use iparse::{IntoParserError, IntoParserResultAddCode, IntoParserResultAddSpan, Span};

    pub fn token_name(rest: Span<'_>) -> APParserResult<'_, APName<'_>> {
        match nom_name(rest) {
            Ok((rest, tok)) => {
                // trim trailing whitespace after the fact.
                let trim = tok.trim_end();

                // the trimmed span is part of original.
                // so reusing the rest ought to be fine.
                let tok = unsafe {
                    Span::new_from_raw_offset(tok.location_offset(), tok.location_line(), trim, ())
                };

                // could rewind the rest too, but since it's whitespace
                // which would be thrown away anyway ...

                Ok((rest, APName { span: tok }))
            }
            Err(e) => Err(e.into_with_code(APCName)),
        }
    }

    pub fn token_name_kurz(rest: Span<'_>) -> APParserResult<'_, APName<'_>> {
        match nom_name_kurz(rest) {
            Ok((rest, tok)) => Ok((rest, APName { span: tok })),
            Err(e) => Err(e.into_with_code(APCNameKurz)),
        }
    }

    pub fn token_nummer(rest: Span<'_>) -> APParserResult<'_, APNummer<'_>> {
        match nom_number(rest) {
            Ok((rest, tok)) => Ok((
                rest,
                APNummer {
                    nummer: tok.parse::<u32>().into_with_span(rest)?,
                    span: tok,
                },
            )),
            Err(e) => Err(e.into_with_code(APCNummer)),
        }
    }

    pub fn token_menge(rest: Span<'_>) -> APParserResult<'_, APMenge<'_>> {
        match nom_number(rest) {
            Ok((rest, tok)) => Ok((
                rest,
                APMenge {
                    menge: tok.parse::<i32>().into_with_span(rest)?,
                    span: tok,
                },
            )),
            Err(e) => Err(e.into_with_code(APCMenge)),
        }
    }

    pub fn token_datum(rest: Span<'_>) -> APParserResult<'_, APDatum<'_>> {
        let (rest, day) = nom_number(rest).into_with_code(APCDay)?;
        let (rest, _) = nom_dot(rest).into_with_code(APCDot)?;
        let (rest, month) = nom_number(rest).into_with_code(APCMonth)?;
        let (rest, _) = nom_dot(rest).into_with_code(APCDot)?;
        let (rest, year) = nom_number(rest).into_with_code(APCYear)?;

        let iday: u32 = (*day).parse().into_with_span(day)?;
        let imonth: u32 = (*month).parse().into_with_span(month)?;
        let iyear: i32 = (*year).parse().into_with_span(year)?;

        let span = span_union(day, year);
        let datum = chrono::NaiveDate::from_ymd_opt(iyear, imonth, iday);

        if let Some(datum) = datum {
            Ok((rest, APDatum { datum, span }))
        } else {
            Err(ParserError::new(APCDatum, span))
        }
    }
}

pub mod nom_tokens {
    use crate::APNomResult;
    use iparse::Span;
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
