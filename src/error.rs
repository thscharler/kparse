use crate::{Code, Span};
use nom;
use nom::error::ErrorKind;
use std::num::NonZeroUsize;

/// Parser error.
///
pub struct ParserError<'s, C: Code, X: Copy = ()> {
    /// Error code
    pub code: C,
    /// Error span
    pub span: ErrorSpan<'s>,
    /// Extra information
    pub hints: Vec<Hints<'s, C, X>>,
}

/// Copy of the span data plus an unknown state.
#[derive(Clone, Copy)]
pub enum ErrorSpan<'s> {
    Span(u32, &'s str, usize),
    Unknown,
}

/// Extra information added to a ParserError.
#[derive(Clone, Copy)]
pub enum Hints<'s, C: Code, X: Copy> {
    /// Contains any nom error that occurred.
    Nom(Nom<'s>),
    /// Contains the nom needed information.
    Needed(NonZeroUsize),
    /// Expected outcome of the parser.
    Expect(Expect<'s, C>),
    /// Extra user context.
    Context(X),
}

#[derive(Clone, Copy)]
pub struct Nom<'s> {
    /// nom ErrorKind
    pub kind: ErrorKind,
    /// Span
    pub span: ErrorSpan<'s>,
    /// Optional char from error.
    pub ch: Option<char>,
}

#[derive(Clone, Copy)]
pub struct Expect<'s, C> {
    /// Error code
    pub code: C,
    /// Span
    pub span: ErrorSpan<'s>,
}

impl<'s> From<Span<'s>> for ErrorSpan<'s> {
    fn from(s: Span<'s>) -> Self {
        ErrorSpan::Span(s.location_line(), s.fragment(), s.location_offset())
    }
}

impl<'s> TryFrom<ErrorSpan<'s>> for Span<'s> {
    type Error = ();

    fn try_from(v: ErrorSpan<'s>) -> Result<Self, Self::Error> {
        match v {
            ErrorSpan::Span(line, fragment, offset) => unsafe {
                Ok(Span::new_from_raw_offset(offset, line, fragment, ()))
            },
            ErrorSpan::Unknown => Err(()),
        }
    }
}

impl<'s, C: Code, X: Copy> nom::error::ParseError<Span<'s>> for ParserError<'s, C, X> {
    fn from_error_kind(input: Span<'s>, kind: ErrorKind) -> Self {
        ParserError {
            code: C::NOM_ERROR,
            span: input.into(),
            hints: vec![Hints::Nom(Nom {
                kind,
                span: input.into(),
                ch: None,
            })],
        }
    }

    fn append(input: Span<'s>, kind: ErrorKind, mut other: Self) -> Self {
        other.hints.push(Hints::Nom(Nom {
            kind,
            span: input.into(),
            ch: None,
        }));
        other
    }

    fn from_char(input: Span<'s>, ch: char) -> Self {
        ParserError {
            code: C::NOM_ERROR,
            span: input.into(),
            hints: vec![Hints::Nom(Nom {
                kind: ErrorKind::Char,
                span: input.into(),
                ch: Some(ch),
            })],
        }
    }

    fn or(self, _other: Self) -> Self {
        // TODO: what is self and what is other
        todo!()
    }
}

impl<'s, C: Code, X: Copy> ParserError<'s, C, X> {
    pub fn new(code: C, span: Span<'s>) -> Self {
        Self {
            code,
            span: span.into(),
            hints: Vec::new(),
        }
    }

    /// Convert to a new error code.
    /// If the old one differs, it is added to the expect list.
    pub fn into_code(mut self, code: C) -> Self {
        if self.code != code {
            self.hints.push(Hints::Expect(Expect {
                code: self.code,
                span: self.span,
            }));
        }
        self.code = code;
        self
    }

    /// Is this one of the nom errorkind codes?
    pub fn is_kind(&self, kind: ErrorKind) -> bool {
        for n in &self.hints {
            if let Hints::Nom(n) = n {
                if n.kind == kind {
                    return true;
                }
            }
        }
        false
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

    /// Return any nom error codes.
    pub fn nom(&self) -> Vec<&Nom<'s>> {
        self.hints
            .iter()
            .filter_map(|v| match v {
                Hints::Nom(n) => Some(n),
                _ => None,
            })
            .collect()
    }

    /// Adds some expect values.
    pub fn add_expect(&mut self, code: C, span: Span<'s>) {
        self.hints.push(Hints::Expect(Expect {
            code,
            span: span.into(),
        }))
    }

    /// Adds some expect values.
    pub fn append_expect(&mut self, exp: Vec<Expect<'s, C>>) {
        for exp in exp.into_iter() {
            self.hints.push(Hints::Expect(exp));
        }
    }

    /// Returns the expect values.
    pub fn iter_expected(&self) -> impl Iterator<Item = &Expect<'s, C>> {
        self.hints.iter().rev().filter_map(|v| match v {
            Hints::Expect(n) => Some(n),
            _ => None,
        })
    }

    /// Get Expect grouped by offset into the string, starting with max first.
    pub fn expect_grouped_by_offset(&self) -> Vec<(usize, Vec<&Expect<'s, C>>)> {
        let mut sorted: Vec<&Expect<'s, C>> = self
            .iter_expected()
            .filter(|v| matches!(v.span, ErrorSpan::Span(_, _, _)))
            .collect();

        sorted.sort_by(|a, b| match a.span {
            ErrorSpan::Span(_, _, offset_a) => match b.span {
                ErrorSpan::Span(_, _, offset_b) => offset_b.cmp(&offset_a),
                ErrorSpan::Unknown => unreachable!(),
            },
            ErrorSpan::Unknown => unreachable!(),
        });

        // per offset
        let mut grp_offset = 0;
        let mut grp = Vec::new();
        let mut subgrp = Vec::new();
        for exp in &sorted {
            let (grp_changed, new_offset) = match exp.span {
                ErrorSpan::Span(_, _, offset) => (offset != grp_offset, offset),
                ErrorSpan::Unknown => unreachable!(),
            };

            if grp_changed {
                if !subgrp.is_empty() {
                    grp.push((grp_offset, subgrp));
                    subgrp = Vec::new();
                }
                grp_offset = new_offset;
            }

            subgrp.push(*exp);
        }
        if !subgrp.is_empty() {
            grp.push((grp_offset, subgrp));
        }

        grp
    }

    /// Get Expect grouped by offset into the string, starting with max first.
    pub fn expect_grouped_by_line(&self) -> Vec<(u32, Vec<&Expect<'s, C>>)> {
        let mut sorted: Vec<&Expect<'s, C>> = self
            .iter_expected()
            .filter(|v| matches!(v.span, ErrorSpan::Span(_, _, _)))
            .collect();

        sorted.sort_by(|a, b| match a.span {
            ErrorSpan::Span(line_a, _, _) => match b.span {
                ErrorSpan::Span(line_b, _, _) => line_b.cmp(&line_a),
                ErrorSpan::Unknown => unreachable!(),
            },
            ErrorSpan::Unknown => unreachable!(),
        });

        // per offset
        let mut grp_line = 0;
        let mut grp = Vec::new();
        let mut subgrp = Vec::new();
        for exp in &sorted {
            let (grp_changed, new_line) = match exp.span {
                ErrorSpan::Span(line, _, _) => (line != grp_line, line),
                ErrorSpan::Unknown => unreachable!(),
            };

            if grp_changed {
                if !subgrp.is_empty() {
                    grp.push((grp_line, subgrp));
                    subgrp = Vec::new();
                }
                grp_line = new_line;
            }

            subgrp.push(*exp);
        }
        if !subgrp.is_empty() {
            grp.push((grp_line, subgrp));
        }

        grp
    }
}
