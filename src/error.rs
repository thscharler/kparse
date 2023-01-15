use crate::debug::error::debug_parse_error;
use crate::debug::{restrict, DebugWidth};
use crate::{Code, Span};
use nom::error::ErrorKind;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};
use std::num::NonZeroUsize;

/// Parser error.
pub struct ParserError<'s, C: Code, Y: Copy = ()> {
    /// Error code
    pub code: C,
    /// Error span
    pub span: Span<'s, C>,
    /// Extra information
    pub hints: Vec<Hints<'s, C, Y>>,
}

/// Extra information added to a ParserError.
pub enum Hints<'s, C: Code, Y: Copy> {
    /// Contains any nom error that occurred.
    Nom(Nom<'s, C>),
    /// Contains the nom needed information.
    Needed(NonZeroUsize),
    /// Expected outcome of the parser.
    Expect(SpanAndCode<'s, C>),
    /// Suggestions from the parser.
    Suggest(SpanAndCode<'s, C>),
    /// External cause for the error.
    Cause(Box<dyn Error>),
    /// Extra user context.
    UserData(Y),
}

/// Contains the data of a nom error.
#[derive(Clone, Copy)]
pub struct Nom<'s, C: Code> {
    /// nom ErrorKind
    pub kind: ErrorKind,
    /// Span
    pub span: Span<'s, C>,
    /// Optional char from error.
    pub ch: Option<char>,
}

/// Contains a error code and the span.
#[derive(Clone, Copy)]
pub struct SpanAndCode<'s, C: Code> {
    /// Error code
    pub code: C,
    /// Span
    pub span: Span<'s, C>,
}

/// Combines two ParserErrors.
pub trait AppendParserError<Rhs = Self> {
    /// Result of the append. Usually (), but for nom::Err::Incomplete the error is not
    /// appended but passed through.
    type Output;
    /// Appends
    fn append(&mut self, err: Rhs) -> Self::Output;
}

impl<'s, C: Code, Y: Copy> AppendParserError<ParserError<'s, C, Y>> for ParserError<'s, C, Y> {
    type Output = ();

    fn append(&mut self, err: ParserError<'s, C, Y>) {
        self.append(err);
    }
}

impl<'s, C: Code, Y: Copy> AppendParserError<ParserError<'s, C, Y>>
    for Option<ParserError<'s, C, Y>>
{
    type Output = ();

    fn append(&mut self, err: ParserError<'s, C, Y>) {
        match self {
            None => *self = Some(err),
            Some(v) => v.append(err),
        }
    }
}

impl<'s, C: Code, Y: Copy> AppendParserError<nom::Err<ParserError<'s, C, Y>>>
    for Option<ParserError<'s, C, Y>>
{
    type Output = Result<(), nom::Err<ParserError<'s, C, Y>>>;

    fn append(
        &mut self,
        err: nom::Err<ParserError<'s, C, Y>>,
    ) -> Result<(), nom::Err<ParserError<'s, C, Y>>> {
        match self {
            None => match err {
                nom::Err::Incomplete(e) => return Err(nom::Err::Incomplete(e)),
                nom::Err::Error(e) => *self = Some(e),
                nom::Err::Failure(e) => *self = Some(e),
            },
            Some(v) => match err {
                nom::Err::Incomplete(e) => return Err(nom::Err::Incomplete(e)),
                nom::Err::Error(e) => v.append(e),
                nom::Err::Failure(e) => v.append(e),
            },
        };
        Ok(())
    }
}

impl<'s, C: Code, Y: Copy> nom::error::ParseError<Span<'s, C>> for ParserError<'s, C, Y> {
    fn from_error_kind(input: Span<'s, C>, kind: ErrorKind) -> Self {
        ParserError {
            code: C::NOM_ERROR,
            span: input,
            hints: vec![Hints::Nom(Nom {
                kind,
                span: input,
                ch: None,
            })],
        }
    }

    fn append(input: Span<'s, C>, kind: ErrorKind, mut other: Self) -> Self {
        other.hints.push(Hints::Nom(Nom {
            kind,
            span: input,
            ch: None,
        }));
        other
    }

    fn from_char(input: Span<'s, C>, ch: char) -> Self {
        ParserError {
            code: C::NOM_ERROR,
            span: input,
            hints: vec![Hints::Nom(Nom {
                kind: ErrorKind::Char,
                span: input,
                ch: Some(ch),
            })],
        }
    }

    /// Combines two parser errors.
    fn or(mut self, other: Self) -> Self {
        self.append(other);
        self
    }
}

impl<'s, C: Code, Y: Copy> Display for ParserError<'s, C, Y> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} expects ", self.code)?;

        for (i, exp) in self.iter_expected().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(
                f,
                "{}:\"{}\"",
                exp.code,
                restrict(DebugWidth::Short, exp.span)
            )?;
        }
        // no suggest
        write!(
            f,
            " for span {} \"{}\"",
            self.span.location_offset(),
            restrict(DebugWidth::Short, self.span)
        )?;
        Ok(())
    }
}

impl<'s, C: Code, Y: Copy> Debug for ParserError<'s, C, Y> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_parse_error(f, self)
    }
}

impl<'s, C: Code, Y: Debug + Copy> Debug for Hints<'s, C, Y> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hints::Nom(v) => write!(f, "Nom {:?}", v),
            Hints::Needed(v) => write!(f, "Needed {:?}", v),
            Hints::Expect(v) => write!(f, "Expect {:?}", v),
            Hints::Suggest(v) => write!(f, "Suggest {:?}", v),
            Hints::Cause(v) => write!(f, "Cause {:?}", v),
            Hints::UserData(v) => write!(f, "UserData {:?}", v),
        }
    }
}

impl<'s, C: Code> Debug for Nom<'s, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = f.width().into();
        write!(f, "{:?}:\"{}\"", self.kind, restrict(w, self.span))?;
        Ok(())
    }
}

impl<'s, C: Code> Debug for SpanAndCode<'s, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = f.width().into();
        write!(f, "{}:\"{}\"", self.code, restrict(w, self.span))?;
        Ok(())
    }
}

impl<'s, C: Code, Y: Copy> Error for ParserError<'s, C, Y> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.hints
            .iter()
            .find(|v| matches!(v, Hints::Cause(_)))
            .and_then(|v| {
                if let Hints::Cause(e) = v {
                    Some(e.as_ref())
                } else {
                    None
                }
            })
    }
}

impl<'s, C: Code, Y: Copy> ParserError<'s, C, Y> {
    /// New error.
    pub fn new(code: C, span: Span<'s, C>) -> Self {
        Self {
            code,
            span,
            hints: Vec::new(),
        }
    }

    /// New error adds the code as Suggestion too.
    pub fn new_suggest(code: C, span: Span<'s, C>) -> Self {
        Self {
            code,
            span,
            hints: vec![Hints::Suggest(SpanAndCode { code, span })],
        }
    }

    /// Finds the first (single) cause.
    pub fn cause(&self) -> Option<&dyn Error> {
        self.hints
            .iter()
            .find(|v| matches!(v, Hints::Cause(_)))
            .and_then(|v| match v {
                Hints::Cause(e) => Some(e.as_ref()),
                _ => None,
            })
    }

    /// Finds the first (single) user data.
    pub fn user_data(&self) -> Option<&Y> {
        self.hints
            .iter()
            .find(|v| matches!(v, Hints::UserData(_)))
            .and_then(|v| match v {
                Hints::UserData(e) => Some(e),
                _ => None,
            })
    }

    /// Adds information from the other parser error to this on.
    ///
    /// Adds the others code and span as expect values.
    /// Adds all the others expect values.
    pub fn append(&mut self, other: ParserError<'s, C, Y>) {
        self.expect(other.code, other.span);
        for expect in other.iter_expected() {
            self.expect(expect.code, expect.span);
        }
    }

    /// Convert to a new error code.
    /// If the old one differs, it is added to the expect list.
    pub fn with_code(mut self, code: C) -> Self {
        if self.code != code {
            self.hints.push(Hints::Expect(SpanAndCode {
                code: self.code,
                span: self.span,
            }));
        }
        self.code = code;
        self
    }

    /// Is this one of the nom ErrorKind codes?
    pub fn is_error_kind(&self, kind: ErrorKind) -> bool {
        for n in &self.hints {
            if let Hints::Nom(n) = n {
                if n.kind == kind {
                    return true;
                }
            }
        }
        false
    }

    /// Return any nom error codes.
    pub fn nom(&self) -> Vec<&Nom<'s, C>> {
        self.hints
            .iter()
            .filter_map(|v| match v {
                Hints::Nom(n) => Some(n),
                _ => None,
            })
            .collect()
    }

    /// Was this one of the expected errors.
    pub fn is_expected(&self, code: C) -> bool {
        for exp in &self.hints {
            if let Hints::Expect(exp) = exp {
                if exp.code == code {
                    return true;
                }
            }
        }
        false
    }

    /// Add an expected code.
    pub fn expect(&mut self, code: C, span: Span<'s, C>) {
        self.hints.push(Hints::Expect(SpanAndCode { code, span }))
    }

    /// Adds some expected codes.
    pub fn append_expected(&mut self, exp: Vec<SpanAndCode<'s, C>>) {
        for exp in exp.into_iter() {
            self.hints.push(Hints::Expect(exp));
        }
    }

    /// Returns the expected codes.
    pub fn iter_expected(&self) -> impl Iterator<Item = &SpanAndCode<'s, C>> {
        self.hints.iter().rev().filter_map(|v| match v {
            Hints::Expect(n) => Some(n),
            _ => None,
        })
    }

    // maybe: move to standalone fn
    /// Get Expect grouped by offset into the string, starting with max first.
    pub fn expected_grouped_by_offset(&self) -> Vec<(usize, Vec<&SpanAndCode<'s, C>>)> {
        let mut sorted: Vec<&SpanAndCode<'s, C>> = self.iter_expected().collect();
        sorted.sort_by(|a, b| b.span.location_offset().cmp(&a.span.location_offset()));

        // per offset
        let mut grp_offset = 0;
        let mut grp = Vec::new();
        let mut subgrp = Vec::new();
        for exp in &sorted {
            if exp.span.location_offset() != grp_offset {
                if !subgrp.is_empty() {
                    grp.push((grp_offset, subgrp));
                    subgrp = Vec::new();
                }
                grp_offset = exp.span.location_offset();
            }

            subgrp.push(*exp);
        }
        if !subgrp.is_empty() {
            grp.push((grp_offset, subgrp));
        }

        grp
    }

    // maybe: move to standalone fn
    /// Get Expect grouped by line number, starting with max first.
    pub fn expected_grouped_by_line(&self) -> Vec<(u32, Vec<&SpanAndCode<'s, C>>)> {
        let mut sorted: Vec<&SpanAndCode<'s, C>> = self.iter_expected().collect();
        sorted.sort_by(|a, b| b.span.location_line().cmp(&a.span.location_line()));

        // per offset
        let mut grp_line = 0;
        let mut grp = Vec::new();
        let mut subgrp = Vec::new();
        for exp in &sorted {
            if exp.span.location_line() != grp_line {
                if !subgrp.is_empty() {
                    grp.push((grp_line, subgrp));
                    subgrp = Vec::new();
                }
                grp_line = exp.span.location_line();
            }

            subgrp.push(*exp);
        }
        if !subgrp.is_empty() {
            grp.push((grp_line, subgrp));
        }

        grp
    }

    /// Add an suggested code.
    pub fn suggest(&mut self, code: C, span: Span<'s, C>) {
        self.hints.push(Hints::Suggest(SpanAndCode { code, span }))
    }

    /// Adds some suggested codes.
    pub fn append_suggested(&mut self, exp: Vec<SpanAndCode<'s, C>>) {
        for exp in exp.into_iter() {
            self.hints.push(Hints::Suggest(exp));
        }
    }

    /// Returns the suggested codes.
    pub fn iter_suggested(&self) -> impl Iterator<Item = &SpanAndCode<'s, C>> {
        self.hints.iter().rev().filter_map(|v| match v {
            Hints::Suggest(n) => Some(n),
            _ => None,
        })
    }

    // maybe: move to standalone fn
    /// Get Suggest grouped by offset into the string, starting with max first.
    pub fn suggested_grouped_by_offset(&self) -> Vec<(usize, Vec<&SpanAndCode<'s, C>>)> {
        let mut sorted: Vec<&SpanAndCode<'s, C>> = self.iter_suggested().collect();
        sorted.sort_by(|a, b| b.span.location_offset().cmp(&a.span.location_offset()));

        // per offset
        let mut grp_offset = 0;
        let mut grp = Vec::new();
        let mut subgrp = Vec::new();
        for exp in &sorted {
            if exp.span.location_offset() != grp_offset {
                if !subgrp.is_empty() {
                    grp.push((grp_offset, subgrp));
                    subgrp = Vec::new();
                }
                grp_offset = exp.span.location_offset();
            }

            subgrp.push(*exp);
        }
        if !subgrp.is_empty() {
            grp.push((grp_offset, subgrp));
        }

        grp
    }

    // maybe: move to standalone fn
    /// Get Suggest grouped by line number, starting with max first.
    pub fn suggested_grouped_by_line(&self) -> Vec<(u32, Vec<&SpanAndCode<'s, C>>)> {
        let mut sorted: Vec<&SpanAndCode<'s, C>> = self.iter_suggested().collect();
        sorted.sort_by(|a, b| b.span.location_line().cmp(&a.span.location_line()));

        // per offset
        let mut grp_line = 0;
        let mut grp = Vec::new();
        let mut subgrp = Vec::new();
        for exp in &sorted {
            if exp.span.location_line() != grp_line {
                if !subgrp.is_empty() {
                    grp.push((grp_line, subgrp));
                    subgrp = Vec::new();
                }
                grp_line = exp.span.location_line();
            }

            subgrp.push(*exp);
        }
        if !subgrp.is_empty() {
            grp.push((grp_line, subgrp));
        }

        grp
    }
}
