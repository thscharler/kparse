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

use crate::debug::error::debug_parse_error;
use crate::debug::{restrict, DebugWidth};
use crate::prelude::SpanFragment;
use crate::{Code, ErrOrNomErr, KParseError};
use nom::error::ErrorKind;
use nom::{InputIter, InputLength, InputTake};
use std::any::Any;
#[cfg(debug_assertions)]
use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

/// Parser error.
pub struct ParserError<C, I> {
    /// Error code
    pub code: C,
    /// Error span
    pub span: I,
    /// Extra information
    pub hints: Vec<Hints<C, I>>,
    #[cfg(debug_assertions)]
    pub backtrace: Backtrace,
}

/// Extra information added to a ParserError.
pub enum Hints<C, I> {
    /// Expected outcome of the parser.
    Expect(SpanAndCode<C, I>),
    /// Suggestions from the parser.
    Suggest(SpanAndCode<C, I>),
    /// External cause for the error.
    Cause(Box<dyn Error>),
    /// Extra user context.
    UserData(Box<dyn Any>),
}

impl<C, I> ErrOrNomErr for ParserError<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    type WrappedError = ParserError<C, I>;

    fn wrap(self) -> nom::Err<Self::WrappedError> {
        nom::Err::Error(self)
    }
}

impl<C, I> ErrOrNomErr for nom::Err<ParserError<C, I>>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    type WrappedError = ParserError<C, I>;

    fn wrap(self) -> nom::Err<Self::WrappedError> {
        self
    }
}

impl<C, I> KParseError<C, I> for ParserError<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    type WrappedError = ParserError<C, I>;

    fn from(code: C, span: I) -> Self {
        ParserError::new(code, span)
    }

    fn with_code(self, code: C) -> Self {
        ParserError::with_code(self, code)
    }

    fn code(&self) -> Option<C> {
        Some(self.code)
    }

    fn span(&self) -> Option<I> {
        Some(self.span.clone())
    }

    fn err(&self) -> Option<&Self::WrappedError> {
        Some(self)
    }

    fn parts(&self) -> Option<(C, I, &Self::WrappedError)> {
        Some((self.code, self.span.clone(), self))
    }
}

impl<C, I> KParseError<C, I> for nom::Err<ParserError<C, I>>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    type WrappedError = ParserError<C, I>;

    fn from(code: C, span: I) -> Self {
        nom::Err::Error(KParseError::from(code, span))
    }

    fn with_code(self, code: C) -> Self {
        match self {
            nom::Err::Incomplete(_) => self,
            nom::Err::Error(e) => nom::Err::Error(e.with_code(code)),
            nom::Err::Failure(e) => nom::Err::Failure(e.with_code(code)),
        }
    }

    fn code(&self) -> Option<C> {
        match self {
            nom::Err::Incomplete(_) => None,
            nom::Err::Error(e) => Some(e.code),
            nom::Err::Failure(e) => Some(e.code),
        }
    }

    fn span(&self) -> Option<I> {
        match self {
            nom::Err::Incomplete(_) => None,
            nom::Err::Error(e) => Some(e.span.clone()),
            nom::Err::Failure(e) => Some(e.span.clone()),
        }
    }

    fn err(&self) -> Option<&Self::WrappedError> {
        match self {
            nom::Err::Incomplete(_) => None,
            nom::Err::Error(e) => Some(e),
            nom::Err::Failure(e) => Some(e),
        }
    }

    fn parts(&self) -> Option<(C, I, &Self::WrappedError)> {
        match self {
            nom::Err::Incomplete(_) => None,
            nom::Err::Error(e) => Some((e.code, e.span.clone(), e)),
            nom::Err::Failure(e) => Some((e.code, e.span.clone(), e)),
        }
    }
}

impl<C, I, O> KParseError<C, I> for Result<(I, O), nom::Err<ParserError<C, I>>>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    type WrappedError = ParserError<C, I>;

    fn from(code: C, span: I) -> Self {
        Err(nom::Err::Error(KParseError::from(code, span)))
    }

    fn with_code(self, code: C) -> Self {
        match self {
            Ok((rest, token)) => Ok((rest, token)),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_code(code))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Error(e.with_code(code))),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }

    fn code(&self) -> Option<C> {
        match self {
            Ok(_) => None,
            Err(nom::Err::Error(e)) => Some(e.code),
            Err(nom::Err::Failure(e)) => Some(e.code),
            Err(nom::Err::Incomplete(_)) => None,
        }
    }

    fn span(&self) -> Option<I> {
        match self {
            Ok(_) => None,
            Err(nom::Err::Error(e)) => Some(e.span.clone()),
            Err(nom::Err::Failure(e)) => Some(e.span.clone()),
            Err(nom::Err::Incomplete(_)) => None,
        }
    }

    fn err(&self) -> Option<&Self::WrappedError> {
        match self {
            Ok(_) => None,
            Err(nom::Err::Error(e)) => Some(e),
            Err(nom::Err::Failure(e)) => Some(e),
            Err(nom::Err::Incomplete(_)) => None,
        }
    }

    fn parts(&self) -> Option<(C, I, &Self::WrappedError)> {
        match self {
            Ok(_) => None,
            Err(nom::Err::Error(e)) => Some((e.code, e.span.clone(), e)),
            Err(nom::Err::Failure(e)) => Some((e.code, e.span.clone(), e)),
            Err(nom::Err::Incomplete(_)) => None,
        }
    }
}

/// Combines two ParserErrors.
pub trait AppendParserError<Rhs = Self> {
    /// Result of the append. Usually (), but for nom::Err::Incomplete the error is not
    /// appended but passed through.
    type Output;
    /// Appends
    fn append(&mut self, err: Rhs) -> Self::Output;
}

impl<C, I> AppendParserError<ParserError<C, I>> for ParserError<C, I>
where
    C: Code,
    I: Clone,
{
    type Output = ();

    fn append(&mut self, err: ParserError<C, I>) {
        self.append_err(err);
    }
}

impl<C, I> AppendParserError<ParserError<C, I>> for Option<ParserError<C, I>>
where
    C: Code,
    I: Clone,
{
    type Output = ();

    fn append(&mut self, err: ParserError<C, I>) {
        match self {
            None => *self = Some(err),
            Some(self_err) => self_err.append_err(err),
        }
    }
}

impl<C, I> AppendParserError<nom::Err<ParserError<C, I>>> for Option<ParserError<C, I>>
where
    C: Code,
    I: Clone,
{
    type Output = Result<(), nom::Err<ParserError<C, I>>>;

    fn append(
        &mut self,
        err: nom::Err<ParserError<C, I>>,
    ) -> Result<(), nom::Err<ParserError<C, I>>> {
        match self {
            None => match err {
                nom::Err::Incomplete(e) => return Err(nom::Err::Incomplete(e)),
                nom::Err::Error(e) => *self = Some(e),
                nom::Err::Failure(e) => *self = Some(e),
            },
            Some(self_err) => match err {
                nom::Err::Incomplete(e) => return Err(nom::Err::Incomplete(e)),
                nom::Err::Error(e) => self_err.append_err(e),
                nom::Err::Failure(e) => self_err.append_err(e),
            },
        };
        Ok(())
    }
}

impl<C, I> AppendParserError<ParserError<C, I>> for nom::Err<ParserError<C, I>>
where
    C: Code,
    I: Clone,
{
    type Output = Result<(), nom::Err<ParserError<C, I>>>;

    fn append(&mut self, err: ParserError<C, I>) -> Self::Output {
        match self {
            nom::Err::Incomplete(e) => return Err(nom::Err::Incomplete(*e)),
            nom::Err::Error(e) => e.append_err(err),
            nom::Err::Failure(e) => e.append_err(err),
        }
        Ok(())
    }
}

impl<C, I> AppendParserError<nom::Err<ParserError<C, I>>> for ParserError<C, I>
where
    C: Code,
    I: Clone,
{
    type Output = Result<(), nom::Err<ParserError<C, I>>>;

    fn append(&mut self, err: nom::Err<ParserError<C, I>>) -> Self::Output {
        match err {
            nom::Err::Incomplete(e) => return Err(nom::Err::Incomplete(e)),
            nom::Err::Error(e) => self.append_err(e),
            nom::Err::Failure(e) => self.append_err(e),
        }
        Ok(())
    }
}

impl<C, I> AppendParserError<nom::Err<ParserError<C, I>>> for nom::Err<ParserError<C, I>>
where
    C: Code,
    I: Clone,
{
    type Output = Result<(), nom::Err<ParserError<C, I>>>;

    fn append(&mut self, err: nom::Err<ParserError<C, I>>) -> Self::Output {
        match self {
            nom::Err::Incomplete(e) => return Err(nom::Err::Incomplete(*e)),
            nom::Err::Error(e) | nom::Err::Failure(e) => match err {
                nom::Err::Incomplete(_) => return Err(err),
                nom::Err::Error(e2) | nom::Err::Failure(e2) => e.append_err(e2),
            },
        }
        Ok(())
    }
}

impl<C, I> nom::error::ParseError<I> for ParserError<C, I>
where
    C: Code,
    I: Clone,
{
    fn from_error_kind(input: I, _kind: ErrorKind) -> Self {
        ParserError {
            code: C::NOM_ERROR,
            span: input,
            hints: Default::default(),
            #[cfg(debug_assertions)]
            backtrace: Backtrace::capture(),
        }
    }

    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: I, _ch: char) -> Self {
        ParserError {
            code: C::NOM_ERROR,
            span: input,
            hints: Default::default(),
            #[cfg(debug_assertions)]
            backtrace: Backtrace::capture(),
        }
    }

    /// Combines two parser errors.
    fn or(mut self, other: Self) -> Self {
        self.append_err(other);
        self
    }
}

impl<C, I> Display for ParserError<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)?;

        if self.iter_expected().next().is_some() {
            write!(f, " expected ")?;
        }
        for (i, exp) in self.iter_expected().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", exp.code)?;
        }

        if self.iter_suggested().next().is_some() {
            write!(f, " suggested ")?;
        }
        for (i, sug) in self.iter_suggested().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", sug.code)?;
        }

        if let Some(cause) = self.cause() {
            write!(f, " cause {:0?}, ", cause)?;
        }

        // no suggest
        write!(
            f,
            " for span {:?}",
            restrict(DebugWidth::Short, self.span.clone()).fragment()
        )?;
        Ok(())
    }
}

impl<C, I> Debug for ParserError<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_parse_error(f, self)
    }
}

impl<C, I> Debug for Hints<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hints::Expect(v) => write!(f, "Expect {:?} ", v),
            Hints::Suggest(v) => write!(f, "Suggest {:?} ", v),
            Hints::Cause(v) => write!(f, "Cause {:?}", v),
            Hints::UserData(v) => write!(f, "UserData {:?}", v),
        }
    }
}

impl<C, I> Error for ParserError<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    fn source(&self) -> Option<&(dyn ::std::error::Error + 'static)> {
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

/// Contains a error code and the span.
#[derive(Clone, Copy)]
pub struct SpanAndCode<C, I> {
    /// Error code
    pub code: C,
    /// Span
    pub span: I,
}

impl<C, I> Debug for SpanAndCode<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = f.width().into();
        write!(
            f,
            "{:?}:{:?}",
            self.code,
            restrict(w, self.span.clone()).fragment()
        )?;
        Ok(())
    }
}

impl<C, I> ParserError<C, I>
where
    C: Code,
    I: Clone,
{
    /// New error.
    pub fn new(code: C, span: I) -> Self {
        Self {
            code,
            span,
            hints: Vec::new(),
            #[cfg(debug_assertions)]
            backtrace: Backtrace::capture(),
        }
    }

    /// With a cause.
    pub fn with_cause<E>(mut self, err: E) -> Self
    where
        E: Error + 'static,
    {
        self.hints.push(Hints::Cause(Box::new(err)));
        self
    }

    /// With user data.
    pub fn with_user_data<Y>(mut self, user_data: Y) -> Self
    where
        Y: 'static,
    {
        self.hints.push(Hints::UserData(Box::new(user_data)));
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
    pub fn user_data<Y: 'static>(&self) -> Option<&Y> {
        self.hints
            .iter()
            .find(|v| matches!(v, Hints::UserData(_)))
            .and_then(|v| match v {
                Hints::UserData(e) => e.downcast_ref::<Y>(),
                _ => None,
            })
    }

    /// Convert to a nom::Err::Error.
    pub fn error(self) -> nom::Err<Self> {
        nom::Err::Error(self)
    }

    /// Convert to a nom::Err::Failure.
    pub fn failure(self) -> nom::Err<Self> {
        nom::Err::Failure(self)
    }

    /// Adds information from the other parser error to this on.
    ///
    /// Adds the others code and span as expect values.
    /// Adds all the others expect values.
    pub fn append_err(&mut self, other: ParserError<C, I>) {
        if other.code != C::NOM_ERROR {
            self.expect(other.code, other.span);
        }
        for hint in other.hints {
            self.hints.push(hint);
        }
    }

    /// Convert to a new error code.
    /// If the old one differs, it is added to the expect list.
    pub fn with_code(mut self, code: C) -> Self {
        if self.code != code && self.code != C::NOM_ERROR {
            self.hints.push(Hints::Expect(SpanAndCode {
                code: self.code,
                span: self.span.clone(),
            }));
        }
        self.code = code;
        self
    }

    /// Was this one of the expected errors.
    /// The main error code is one of the tested values.
    pub fn is_expected(&self, code: C) -> bool {
        if self.code == code {
            return true;
        }
        for exp in &self.hints {
            if let Hints::Expect(v) = exp {
                if v.code == code {
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
    pub fn append_expected(&mut self, exp_iter: impl Iterator<Item = SpanAndCode<C, I>>) {
        for exp in exp_iter {
            self.hints.push(Hints::Expect(exp));
        }
    }

    /// Returns the expected codes.
    ///
    /// # Beware
    ///
    /// The main error code is not included here.
    pub fn iter_expected(&self) -> impl Iterator<Item = SpanAndCode<C, I>> + '_ {
        self.hints.iter().rev().filter_map(|v| match v {
            Hints::Expect(v) => Some(v.clone()),
            _ => None,
        })
    }

    /// Add an suggested code.
    pub fn suggest(&mut self, code: C, span: I) {
        self.hints.push(Hints::Suggest(SpanAndCode { code, span }))
    }

    /// Was this one of the expected errors.
    /// The main error code is one of the tested values.
    pub fn is_suggested(&self, code: C) -> bool {
        for exp in &self.hints {
            if let Hints::Suggest(v) = exp {
                if v.code == code {
                    return true;
                }
            }
        }
        false
    }

    /// Adds some suggested codes.
    pub fn append_suggested(&mut self, sug_iter: impl Iterator<Item = SpanAndCode<C, I>>) {
        for exp in sug_iter {
            self.hints.push(Hints::Suggest(exp));
        }
    }

    /// Returns the suggested codes.
    pub fn iter_suggested(&self) -> impl Iterator<Item = SpanAndCode<C, I>> + '_ {
        self.hints.iter().rev().filter_map(|v| match v {
            Hints::Suggest(v) => Some(v.clone()),
            _ => None,
        })
    }
}
