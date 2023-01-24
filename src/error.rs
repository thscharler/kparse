//!
//! Error type, nom::error::Error replacement.
//!
//! It's main content is an error code and a span.
//! Additionally
//! * nom error codes
//! * extra codes indicating expected input
//! * extra codes for suggestions
//! * cause
//! * other user data.
//!
//! To change the error code during parse use with_code(). This keeps the
//! old error code as expected value. with_code() also exists for Result's
//! that contain a ParserError.
//!
//! To convert some error to a ParserError the trait WithSpan can be used.
//! A From conversion works fine too.
//!
//!

use crate::debug::error::debug_parse_error;
use crate::debug::{restrict, DebugWidth};
use crate::spans::LocatedSpanExt;
use crate::Code;
use nom::error::ErrorKind;
use nom::{InputIter, InputLength, InputTake, Offset, Slice};
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::ops::{RangeFrom, RangeTo};

/// Parser error.
pub struct ParserError<C: Code, I, Y: Copy = ()> {
    /// Error code
    pub code: C,
    /// Error span
    pub span: I,
    /// Extra information
    pub hints: Vec<Hints<C, I, Y>>,
}

/// Extra information added to a ParserError.
pub enum Hints<C: Code, I, Y: Copy> {
    /// Contains any nom error that occurred.
    Nom(Nom<C, I>),
    /// Contains the nom needed information.
    Needed(NonZeroUsize),
    /// Expected outcome of the parser.
    Expect(SpanAndCode<C, I>),
    /// Suggestions from the parser.
    Suggest(SpanAndCode<C, I>),
    /// External cause for the error.
    Cause(Box<dyn Error>),
    /// Extra user context.
    UserData(Y),
}

/// Contains the data of a nom error.
#[derive(Clone, Copy)]
pub struct Nom<C: Code, I> {
    /// nom ErrorKind
    pub kind: ErrorKind,
    /// Span
    pub span: I,
    /// Optional char from error.
    pub ch: Option<char>,
    /// ...
    pub _phantom: PhantomData<C>,
}

/// Contains a error code and the span.
#[derive(Clone, Copy)]
pub struct SpanAndCode<C: Code, I> {
    /// Error code
    pub code: C,
    /// Span
    pub span: I,
}

/// Combines two ParserErrors.
pub trait AppendParserError<Rhs = Self> {
    /// Result of the append. Usually (), but for nom::Err::Incomplete the error is not
    /// appended but passed through.
    type Output;
    /// Appends
    fn append(&mut self, err: Rhs) -> Self::Output;
}

impl<C, I, Y> AppendParserError<ParserError<C, I, Y>> for ParserError<C, I, Y>
where
    C: Code,
    I: Copy,
    Y: Copy,
{
    type Output = ();

    fn append(&mut self, err: ParserError<C, I, Y>) {
        ParserError::append(self, err);
    }
}

impl<C, I, Y> AppendParserError<ParserError<C, I, Y>> for Option<ParserError<C, I, Y>>
where
    C: Code,
    I: Copy,
    Y: Copy,
{
    type Output = ();

    fn append(&mut self, err: ParserError<C, I, Y>) {
        match self {
            None => *self = Some(err),
            Some(v) => v.append(err),
        }
    }
}

impl<C, I, Y> AppendParserError<nom::Err<ParserError<C, I, Y>>> for Option<ParserError<C, I, Y>>
where
    C: Code,
    I: Copy,
    Y: Copy,
{
    type Output = Result<(), nom::Err<ParserError<C, I, Y>>>;

    fn append(
        &mut self,
        err: nom::Err<ParserError<C, I, Y>>,
    ) -> Result<(), nom::Err<ParserError<C, I, Y>>> {
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

impl<C, I, Y> nom::error::ParseError<I> for ParserError<C, I, Y>
where
    C: Code,
    I: Copy,
    Y: Copy,
{
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        ParserError {
            code: C::NOM_ERROR,
            span: input,
            hints: vec![Hints::Nom(Nom {
                kind,
                span: input,
                ch: None,
                _phantom: Default::default(),
            })],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.hints.push(Hints::Nom(Nom {
            kind,
            span: input,
            ch: None,
            _phantom: Default::default(),
        }));
        other
    }

    fn from_char(input: I, ch: char) -> Self {
        ParserError {
            code: C::NOM_ERROR,
            span: input,
            hints: vec![Hints::Nom(Nom {
                kind: ErrorKind::Char,
                span: input,
                ch: Some(ch),
                _phantom: Default::default(),
            })],
        }
    }

    /// Combines two parser errors.
    fn or(mut self, other: Self) -> Self {
        self.append(other);
        self
    }
}

impl<C, I, Y> Display for ParserError<C, I, Y>
where
    C: Code,
    I: Copy + Display,
    Y: Copy,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} expects ", self.code)?;

        for (i, exp) in self.iter_expected().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}:{}", exp.code, restrict(DebugWidth::Short, exp.span))?;
        }
        // no suggest
        write!(f, " for span {}", restrict(DebugWidth::Short, self.span))?;
        Ok(())
    }
}

impl<C, I, Y> Debug for ParserError<C, I, Y>
where
    C: Code,
    I: Copy + Debug,
    Y: Copy,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_parse_error(f, self)
    }
}

impl<C, I, Y> Debug for Hints<C, I, Y>
where
    C: Code,
    I: Copy + Debug,
    Y: Copy + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
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

impl<C, I> Debug for Nom<C, I>
where
    C: Code,
    I: Copy + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = f.width().into();
        write!(f, "{:?}:{:?}", self.kind, restrict(w, self.span))?;
        Ok(())
    }
}

impl<C, I> Debug for SpanAndCode<C, I>
where
    C: Code,
    I: Copy + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = f.width().into();
        write!(f, "{}:{:?}", self.code, restrict(w, self.span))?;
        Ok(())
    }
}

impl<C, I, Y> Error for ParserError<C, I, Y>
where
    C: Code,
    Y: Copy,
    I: Copy + Display + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
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

impl<C, I, Y> ParserError<C, I, Y>
where
    C: Code,
    I: Copy,
    Y: Copy,
{
    /// New error.
    pub fn new(code: C, span: I) -> Self {
        Self {
            code,
            span,
            hints: Vec::new(),
        }
    }

    /// New error adds the code as Suggestion too.
    pub fn new_suggest(code: C, span: I) -> Self {
        Self {
            code,
            span,
            hints: vec![Hints::Suggest(SpanAndCode { code, span })],
        }
    }

    /// With a cause.
    pub fn with_cause(mut self, err: Box<dyn Error>) -> Self {
        self.hints.push(Hints::Cause(err));
        self
    }

    /// With user data.
    pub fn with_user_data(mut self, user_data: Y) -> Self {
        self.hints.push(Hints::UserData(user_data));
        self
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
    pub fn append(&mut self, other: ParserError<C, I, Y>) {
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
    pub fn nom(&self) -> Vec<&Nom<C, I>> {
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
    pub fn expect(&mut self, code: C, span: I) {
        self.hints.push(Hints::Expect(SpanAndCode { code, span }))
    }

    /// Adds some expected codes.
    pub fn append_expected(&mut self, exp: Vec<SpanAndCode<C, I>>) {
        for exp in exp.into_iter() {
            self.hints.push(Hints::Expect(exp));
        }
    }

    /// Returns the expected codes.
    pub fn iter_expected(&self) -> impl Iterator<Item = &SpanAndCode<C, I>> {
        self.hints.iter().rev().filter_map(|v| match v {
            Hints::Expect(n) => Some(n),
            _ => None,
        })
    }

    // maybe: move to standalone fn
    /// Get Expect grouped by offset into the string, starting with max first.
    pub fn expected_grouped_by_offset(&self) -> Vec<(usize, Vec<&SpanAndCode<C, I>>)>
    where
        I: LocatedSpanExt,
    {
        let mut sorted: Vec<&SpanAndCode<C, I>> = self.iter_expected().collect();
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
    pub fn expected_grouped_by_line(&self) -> Vec<(u32, Vec<&SpanAndCode<C, I>>)>
    where
        I: LocatedSpanExt,
    {
        let mut sorted: Vec<&SpanAndCode<C, I>> = self.iter_expected().collect();
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
    pub fn suggest(&mut self, code: C, span: I) {
        self.hints.push(Hints::Suggest(SpanAndCode { code, span }))
    }

    /// Adds some suggested codes.
    pub fn append_suggested(&mut self, exp: Vec<SpanAndCode<C, I>>) {
        for exp in exp.into_iter() {
            self.hints.push(Hints::Suggest(exp));
        }
    }

    /// Returns the suggested codes.
    pub fn iter_suggested(&self) -> impl Iterator<Item = &SpanAndCode<C, I>> {
        self.hints.iter().rev().filter_map(|v| match v {
            Hints::Suggest(n) => Some(n),
            _ => None,
        })
    }

    // maybe: move to standalone fn
    /// Get Suggest grouped by offset into the string, starting with max first.
    pub fn suggested_grouped_by_offset(&self) -> Vec<(usize, Vec<&SpanAndCode<C, I>>)>
    where
        I: LocatedSpanExt,
    {
        let mut sorted: Vec<&SpanAndCode<C, I>> = self.iter_suggested().collect();
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
    pub fn suggested_grouped_by_line(&self) -> Vec<(u32, Vec<&SpanAndCode<C, I>>)>
    where
        I: LocatedSpanExt,
    {
        let mut sorted: Vec<&SpanAndCode<C, I>> = self.iter_suggested().collect();
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
