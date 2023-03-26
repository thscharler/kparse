//!
//! Second, simpler error type. Same size as nom::error::Error.
//!
//! Can only hold one error code and a span.
//!

use crate::debug::{restrict, DebugWidth};
use crate::parser_error::ParserError;
use crate::spans::SpanFragment;
use crate::{Code, ErrOrNomErr, KParseError};
use nom::error::ErrorKind;
use nom::{InputIter, InputLength, InputTake};
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};

/// Shorter error type for the tokenizer stage.
/// Nom parsers fail often, so it's good to keep this minimal.
pub struct TokenizerError<C, I> {
    /// Error code
    pub code: C,
    /// Error span
    pub span: I,
}

impl<C, I> ErrOrNomErr for TokenizerError<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    type WrappedError = TokenizerError<C, I>;

    fn wrap(self) -> nom::Err<Self::WrappedError> {
        nom::Err::Error(self)
    }
}

impl<C, I> ErrOrNomErr for nom::Err<TokenizerError<C, I>>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    type WrappedError = TokenizerError<C, I>;

    fn wrap(self) -> nom::Err<Self::WrappedError> {
        self
    }
}

impl<C, I> KParseError<C, I> for TokenizerError<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    type WrappedError = TokenizerError<C, I>;

    fn from(code: C, span: I) -> Self {
        TokenizerError::new(code, span)
    }

    fn with_code(self, code: C) -> Self {
        TokenizerError::with_code(self, code)
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

impl<C, I> From<TokenizerError<C, I>> for ParserError<C, I>
where
    C: Code,
    I: Clone,
{
    fn from(value: TokenizerError<C, I>) -> Self {
        ParserError::new(value.code, value.span)
    }
}

impl<C, I> KParseError<C, I> for nom::Err<TokenizerError<C, I>>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    type WrappedError = TokenizerError<C, I>;

    fn from(code: C, span: I) -> Self {
        nom::Err::Error(KParseError::from(code, span))
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

    fn with_code(self, code: C) -> Self {
        match self {
            nom::Err::Incomplete(_) => self,
            nom::Err::Error(e) => nom::Err::Error(e.with_code(code)),
            nom::Err::Failure(e) => nom::Err::Failure(e.with_code(code)),
        }
    }
}

impl<C, I, O> KParseError<C, I> for Result<(I, O), nom::Err<TokenizerError<C, I>>>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    type WrappedError = TokenizerError<C, I>;

    fn from(code: C, span: I) -> Self {
        Err(nom::Err::Error(KParseError::from(code, span)))
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

    fn with_code(self, code: C) -> Self {
        match self {
            Ok((rest, token)) => Ok((rest, token)),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_code(code))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Error(e.with_code(code))),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}

impl<C, I> nom::error::ParseError<I> for TokenizerError<C, I>
where
    C: Code,
    I: Clone + Debug,
    I: InputTake + InputLength + InputIter,
{
    fn from_error_kind(input: I, _kind: ErrorKind) -> Self {
        TokenizerError {
            code: C::NOM_ERROR,
            span: input,
        }
    }

    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        // could max overwrite something useful.
        other
    }

    fn from_char(input: I, _char: char) -> Self {
        TokenizerError {
            code: C::NOM_ERROR,
            span: input,
        }
    }

    fn or(mut self, other: Self) -> Self {
        self.append_err(other);
        self
    }
}

impl<C, I> Display for TokenizerError<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)?;
        write!(
            f,
            " for span {:?}",
            restrict(DebugWidth::Short, self.span.clone()).fragment()
        )?;
        Ok(())
    }
}

impl<C, I> Debug for TokenizerError<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dw: DebugWidth = f.width().into();
        write!(f, "{}", self.code)?;
        write!(
            f,
            " for span {:?}",
            restrict(dw, self.span.clone()).fragment()
        )?;
        Ok(())
    }
}

impl<C, I> Error for TokenizerError<C, I>
where
    C: Code,
    I: Clone + Debug + SpanFragment,
    I: InputTake + InputLength + InputIter,
{
}

impl<C, I> TokenizerError<C, I>
where
    C: Code,
    I: Clone,
{
    /// New error.
    pub fn new(code: C, span: I) -> Self {
        Self { code, span }
    }

    /// Replaces the information with the other error.
    /// Unless the other contains only the generic NOM_ERROR.
    pub fn append_err(&mut self, other: TokenizerError<C, I>) {
        if other.code != C::NOM_ERROR {
            self.code = other.code;
            self.span = other.span;
        }
    }

    /// Convert to a new error code.
    /// If the old one differs, it is added to the expect list.
    pub fn with_code(mut self, code: C) -> Self {
        self.code = code;
        self
    }

    /// Convert to a nom::Err::Error.
    pub fn error(self) -> nom::Err<Self> {
        nom::Err::Error(self)
    }

    /// Convert to a nom::Err::Failure.
    pub fn failure(self) -> nom::Err<Self> {
        nom::Err::Failure(self)
    }
}
