use crate::error::ParserError;
use nom_locate::LocatedSpan;
use std::error::Error;
use std::fmt::{Debug, Display};

/// Standard input type.
pub type Span<'s, X> = LocatedSpan<&'s str, X>;

/// Result type.
pub type ParserResult<'s, O, C, X, Y> = Result<(Span<'s, X>, O), nom::Err<ParserError<'s, C, Y>>>;

/// Parser state codes.
///
/// These are used for error handling and parser results and
/// everything else.
pub trait Code: Copy + Display + Debug + Eq {
    const NOM_ERROR: Self;
}

pub trait ParseContext<C> {
    type Span<'s>;

    fn original<'s>(&self, span: &Self::Span<'s>) -> Self::Span<'s>;

    /// Tracks entering a parser function.
    fn enter(&self, func: C, span: &Self::Span<'_>);

    /// Tracks an Ok result of a parser function.
    fn exit_ok(&self, span: &Self::Span<'_>, parsed: &Self::Span<'_>);

    /// Tracks an Err result of a parser function.    
    fn exit_err(&self, span: &Self::Span<'_>, code: C, err: &dyn Error);
}

#[derive(Debug, Clone, Copy)]
pub struct StrContext<'s>(&'s str);

impl<'a, C: Code> ParseContext<C> for StrContext<'a> {
    type Span<'s> = LocatedSpan<&'a str, StrContext<'a>>;

    fn original<'b>(&self, span: &Self::Span<'b>) -> Self::Span<'b> {
        Span::new_extra(self.0, span.extra)
    }

    fn enter(&self, func: C, span: &Self::Span<'a>) {
        println!("enter {} {}", func, span)
    }

    fn exit_ok(&self, span: &Self::Span<'a>, parsed: &Self::Span<'a>) {
        println!("exit ok {} {}", span, parsed)
    }

    fn exit_err(&self, span: &Self::Span<'a>, code: C, err: &dyn Error) {
        println!("exit err {} {} {:?}", span, code, err)
    }
}

impl<'a, C: Code> ParseContext<C> for &'a StrContext<'a> {
    type Span<'s> = LocatedSpan<&'a str, &'a StrContext<'a>>;

    fn original<'b>(&self, span: &Self::Span<'b>) -> Self::Span<'b> {
        Span::new_extra(self.0, span.extra)
    }

    fn enter(&self, func: C, span: &Self::Span<'a>) {
        println!("enter {} {}", func, span)
    }

    fn exit_ok(&self, span: &Self::Span<'a>, parsed: &Self::Span<'a>) {
        println!("exit ok {} {}", span, parsed)
    }

    fn exit_err(&self, span: &Self::Span<'a>, code: C, err: &dyn Error) {
        println!("exit err {} {} {:?}", span, code, err)
    }
}

impl<C> ParseContext<C> for () {
    type Span<'a> = LocatedSpan<&'a str, ()>;

    fn original<'b>(&self, span: &Self::Span<'b>) -> Self::Span<'b> {
        *span
    }

    fn enter(&self, _func: C, _span: &Self::Span<'_>) {}

    fn exit_ok(&self, _span: &Self::Span<'_>, _parsed: &Self::Span<'_>) {}

    fn exit_err(&self, _span: &Self::Span<'_>, _code: C, _err: &dyn Error) {}
}

mod error {
    use crate::debug::{restrict, DebugWidth};
    use crate::{Code, Span};
    use nom::error::ErrorKind;
    use nom_locate::LocatedSpan;
    use std::error::Error;
    use std::fmt;
    use std::fmt::{Debug, Display, Formatter};
    use std::num::NonZeroUsize;

    type ErrSpan<'s> = LocatedSpan<&'s str, ()>;

    fn drop_extra<'a, X>(span: &LocatedSpan<&'a str, X>) -> LocatedSpan<&'a str, ()> {
        unsafe {
            LocatedSpan::new_from_raw_offset(
                span.location_offset(),
                span.location_line(),
                span.fragment(),
                (),
            )
        }
    }

    pub struct ParserError<'s, C, Y = ()> {
        /// Error code
        pub code: C,
        /// Error span
        pub span: ErrSpan<'s>,
        /// Extra information
        pub hints: Vec<Hints<'s, C, Y>>,
    }

    /// Extra information added to a ParserError.
    pub enum Hints<'s, C, Y> {
        /// Contains any nom error that occurred.
        Nom(Nom<'s>),
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

    #[derive(Clone, Copy)]
    pub struct Nom<'s> {
        /// nom ErrorKind
        pub kind: ErrorKind,
        /// Span
        pub span: ErrSpan<'s>,
        /// Optional char from error.
        pub ch: Option<char>,
    }

    #[derive(Clone, Copy)]
    pub struct SpanAndCode<'s, C> {
        /// Error code
        pub code: C,
        /// Span
        pub span: ErrSpan<'s>,
    }

    /// Combines two ParserErrors.
    pub trait CombineParserError<'s, C, Y, Rhs = Self> {
        fn add(&mut self, err: Rhs) -> Result<(), nom::Err<ParserError<'s, C, Y>>>;
    }

    impl<'s, C, Y> CombineParserError<'s, C, Y, ParserError<'s, C, Y>> for ParserError<'s, C, Y>
    where
        C: Code,
    {
        fn add(
            &mut self,
            err: ParserError<'s, C, Y>,
        ) -> Result<(), nom::Err<ParserError<'s, C, Y>>> {
            self.append(err);
            Ok(())
        }
    }

    impl<'s, C, Y> CombineParserError<'s, C, Y, ParserError<'s, C, Y>> for Option<ParserError<'s, C, Y>>
    where
        C: Code,
    {
        fn add(
            &mut self,
            err: ParserError<'s, C, Y>,
        ) -> Result<(), nom::Err<ParserError<'s, C, Y>>> {
            match self {
                None => *self = Some(err),
                Some(v) => v.append(err),
            }
            Ok(())
        }
    }

    impl<'s, C, Y> CombineParserError<'s, C, Y, nom::Err<ParserError<'s, C, Y>>>
        for Option<ParserError<'s, C, Y>>
    where
        C: Code,
    {
        fn add(
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

    impl<'s, C, X, Y> nom::error::ParseError<Span<'s, X>> for ParserError<'s, C, Y>
    where
        C: Code,
    {
        fn from_error_kind(input: Span<'s, X>, kind: ErrorKind) -> Self {
            ParserError {
                code: C::NOM_ERROR,
                span: drop_extra(&input),
                hints: vec![Hints::Nom(Nom {
                    kind,
                    span: drop_extra(&input),
                    ch: None,
                })],
            }
        }

        fn append(input: Span<'s, X>, kind: ErrorKind, mut other: Self) -> Self {
            other.hints.push(Hints::Nom(Nom {
                kind,
                span: drop_extra(&input),
                ch: None,
            }));
            other
        }

        fn from_char(input: Span<'s, X>, ch: char) -> Self {
            ParserError {
                code: C::NOM_ERROR,
                span: drop_extra(&input),
                hints: vec![Hints::Nom(Nom {
                    kind: ErrorKind::Char,
                    span: drop_extra(&input),
                    ch: Some(ch),
                })],
            }
        }

        // todo: what is self and what is other
        fn or(mut self, other: Self) -> Self {
            self.append(other);
            self
        }
    }

    impl<'s, C, Y> Display for ParserError<'s, C, Y>
    where
        C: Code,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

    impl<'s, C, Y> Debug for ParserError<'s, C, Y> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match f.width() {
                // None | Some(0) => debug_parse_of_error_short(f, self),
                // Some(1) => debug_parse_of_error_medium(f, self),
                // Some(2) => debug_parse_of_error_long(f, self),
                _ => Ok(()),
            }
        }
    }

    impl<'s, C> Debug for SpanAndCode<'s, C>
    where
        C: Code,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let w = f.width().into();
            write!(f, "{}:\"{}\"", self.code, restrict(w, self.span))?;
            Ok(())
        }
    }

    impl<'s, C, Y> Error for ParserError<'s, C, Y>
    where
        C: Code,
    {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.hints
                .iter()
                .filter(|v| matches!(v, Hints::Cause(_)))
                .next()
                .map(|v| {
                    if let Hints::Cause(e) = v {
                        Some(e.as_ref())
                    } else {
                        None
                    }
                })
                .flatten()
        }
    }

    impl<'s, C, Y> ParserError<'s, C, Y>
    where
        C: Code,
    {
        pub fn new<X>(code: C, span: Span<'s, X>) -> Self {
            Self {
                code,
                span: drop_extra(&span),
                hints: Vec::new(),
            }
        }

        /// New error adds the code as Suggestion too.
        pub fn new_suggest<X>(code: C, span: Span<'s, X>) -> Self {
            Self {
                code,
                span: drop_extra(&span),
                hints: vec![Hints::Suggest(SpanAndCode {
                    code,
                    span: drop_extra(&span),
                })],
            }
        }

        // todo: something missing?

        /// Adds information from the other parser error to this on.
        ///
        /// Adds the others code and span as expect values.
        /// Adds all the others expect values.
        ///
        /// TODO: may need completion
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
        pub fn nom(&self) -> Vec<&Nom<'s>> {
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
        pub fn expect<X>(&mut self, code: C, span: Span<'s, X>) {
            self.hints.push(Hints::Expect(SpanAndCode {
                code,
                span: drop_extra(&span),
            }))
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
        pub fn suggest<X>(&mut self, code: C, span: Span<'s, X>) {
            self.hints.push(Hints::Suggest(SpanAndCode {
                code,
                span: drop_extra(&span),
            }))
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
}

mod debug {
    use crate::Span;
    use nom::bytes::complete::take_while_m_n;
    use nom::InputIter;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum DebugWidth {
        /// Debug flag, can be set with width=0.
        Short,
        /// Debug flag, can be set with width=1.
        Medium,
        /// Debug flag, can be set with width=2.
        Long,
    }

    impl From<Option<usize>> for DebugWidth {
        fn from(value: Option<usize>) -> Self {
            match value {
                None | Some(0) => DebugWidth::Short,
                Some(1) => DebugWidth::Medium,
                Some(2) => DebugWidth::Long,
                _ => DebugWidth::Short,
            }
        }
    }

    pub fn restrict<X: Copy>(w: DebugWidth, span: Span<'_, X>) -> String {
        match w {
            DebugWidth::Short => restrict_n(20, span),
            DebugWidth::Medium => restrict_n(40, span),
            DebugWidth::Long => restrict_n(60, span),
        }
    }

    pub fn restrict_n<X: Copy>(max_len: usize, span: Span<'_, X>) -> String {
        let shortened =
            match take_while_m_n::<_, _, nom::error::Error<Span<'_, X>>>(0, max_len, |_c| true)(
                span,
            ) {
                Ok((_rest, short)) => *short,
                Err(_) => "?error?",
            };

        if span.len() > max_len {
            shortened
                .escape_default()
                .chain("...".iter_elements())
                .collect()
        } else {
            shortened.escape_default().collect()
        }
    }
}

mod context2 {
    use crate::{ParseContext, Span};
    use std::error::Error;

    pub fn original<'s, C, X>(_func: C, span: &Span<'s, X>) -> Span<'s, X>
    where
        X: ParseContext<C, Span<'s> = Span<'s, X>>,
    {
        span.extra.original(span)
    }
}

mod context {
    use crate::{ParseContext, Span};
    use std::error::Error;

    pub fn original<'s, C, X>(_func: C, span: &Span<'s, X>) -> Span<'s, X>
    where
        X: ParseContext<C, Span<'s> = Span<'s, X>>,
    {
        span.extra.original(span)
    }

    pub fn enter<'s, C, X>(func: C, span: &Span<'s, X>)
    where
        X: ParseContext<C, Span<'s> = Span<'s, X>>,
    {
        span.extra.enter(func, span)
    }

    pub fn exit_ok<'s, C, X>(span: &Span<'s, X>, parsed: &Span<'s, X>)
    where
        X: ParseContext<C, Span<'s> = Span<'s, X>>,
    {
        span.extra.exit_ok(span, parsed)
    }

    pub fn exit_err<'s, C, X>(span: &Span<'s, X>, code: C, err: &dyn Error)
    where
        X: ParseContext<C, Span<'s> = Span<'s, X>>,
    {
        span.extra.exit_err(span, code, err)
    }
}

mod tests {
    use crate::{context, Code, ParserError, Span, StrContext};
    use std::fmt::{Display, Formatter};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestCode {
        Code1,
        Code2,
    }

    use TestCode::*;

    impl Display for TestCode {
        fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }

    impl Code for TestCode {
        const NOM_ERROR: Self = Self::Code1;
    }

    type TParserError<'s, C> = ParserError<'s, C, ()>;

    #[test]
    fn test_convoluted() {
        let c0 = ();
        let s0 = Span::new_extra("wxyz", c0);
        let e0 = TParserError::new(Code2, s0);

        let c1 = StrContext("abcd");
        let s1 = Span::new_extra("wxyz", c1);
        let e1 = TParserError::new(Code2, s1);

        let c22 = &c1;
        let s2 = Span::new_extra("wxyz", c22);
        let e2 = TParserError::new(Code2, s2);

        dbg!(context::original(Code1, &s0));
        dbg!(context::enter(Code1, &s0));
        dbg!(context::exit_err(&s0, Code1, &e0));
        dbg!(context::exit_ok::<TestCode, _>(&s0, &s0));

        dbg!(context::original(Code1, &s1));
        dbg!(context::enter(Code1, &s1));
        dbg!(context::exit_err(&s1, Code1, &e1));
        dbg!(context::exit_ok::<TestCode, _>(&s1, &s1));

        dbg!(context::original(Code1, &s2));
        dbg!(context::enter(Code1, &s2));
        dbg!(context::exit_err(&s2, Code1, &e2));
        dbg!(context::exit_ok::<TestCode, _>(&s2, &s2));
    }
}
