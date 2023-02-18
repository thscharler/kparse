use crate::debug::{restrict, DebugWidth};
use crate::{Code, ParseErrorExt, ParserError, WithCode};
use nom::error::ErrorKind;
use nom::{AsBytes, InputIter, InputLength, InputTake};
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

impl<C, I> ParseErrorExt<C, I> for TokenizerError<C, I>
where
    C: Code,
    I: Copy,
{
    fn code(&self) -> C {
        self.code
    }

    fn span(&self) -> I {
        self.span
    }
}

impl<C, I> nom::error::ParseError<I> for TokenizerError<C, I>
where
    C: Code,
    I: Copy + Debug,
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
    I: Copy + Debug,
    I: InputTake + InputLength + InputIter,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)?;
        write!(f, " for span {:?}", restrict(DebugWidth::Short, self.span))?;
        Ok(())
    }
}

impl<C, I> Debug for TokenizerError<C, I>
where
    C: Code,
    I: Copy + Debug,
    I: InputTake + InputLength + InputIter,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dw: DebugWidth = f.width().into();
        write!(f, "{}", self.code)?;
        write!(f, " for span {:?}", restrict(dw, self.span))?;
        Ok(())
    }
}

impl<C, I> Error for TokenizerError<C, I>
where
    C: Code,
    I: Copy + Debug,
    I: InputTake + InputLength + InputIter,
{
}

impl<C, I> TokenizerError<C, I>
where
    C: Code,
    I: Copy,
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
}

/// The From trait can't be used for types wrapped in a nom::Err.
/// We do this for the conversion from TokenizerError to ParserError.
pub trait IntoParserError<R> {
    /// Convert to a form of ParserError.
    fn into_parser_err(self) -> R;
}

/// The From trait can't be used for types wrapped in a nom::Err.
/// We do this for the conversion from TokenizerError to ParserError.
pub trait IntoParserErrorExtra<R, Y> {
    /// Convert to a from of ParserError with
    fn into_parser_err_with(self, extra: Y) -> R;
}

// ***********************************************************************
// LAYER 1 - useful conversions
// ***********************************************************************

//
// ParserError to nom::Err<ParserError>, useful shortcut when creating
// a fresh ParserError.
//
impl<C, I> From<TokenizerError<C, I>> for nom::Err<TokenizerError<C, I>>
where
    C: Code,
    I: AsBytes + Copy,
{
    fn from(e: TokenizerError<C, I>) -> Self {
        nom::Err::Error(e)
    }
}

impl<C, I> WithCode<C, TokenizerError<C, I>> for TokenizerError<C, I>
where
    I: AsBytes + Copy,
    C: Code,
{
    fn with_code(self, code: C) -> TokenizerError<C, I> {
        TokenizerError::with_code(self, code)
    }
}

impl<C, I> IntoParserError<ParserError<C, I, ()>> for TokenizerError<C, I>
where
    C: Code,
    I: Copy,
{
    fn into_parser_err(self) -> ParserError<C, I, ()> {
        ParserError::new(self.code, self.span)
    }
}

impl<C, I, Y> IntoParserErrorExtra<ParserError<C, I, Y>, Y> for TokenizerError<C, I>
where
    C: Code,
    I: Copy,
    Y: Copy,
{
    fn into_parser_err_with(self, extra: Y) -> ParserError<C, I, Y> {
        ParserError::new(self.code, self.span).with_user_data(extra)
    }
}

// ***********************************************************************
// LAYER 2 - wrapped in a nom::Err
// ***********************************************************************

impl<C, I> WithCode<C, nom::Err<TokenizerError<C, I>>> for nom::Err<TokenizerError<C, I>>
where
    C: Code,
    I: AsBytes + Copy,
{
    fn with_code(self, code: C) -> nom::Err<TokenizerError<C, I>> {
        match self {
            nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            nom::Err::Error(e) => {
                let p_err: TokenizerError<C, I> = e.with_code(code);
                nom::Err::Error(p_err)
            }
            nom::Err::Failure(e) => {
                let p_err: TokenizerError<C, I> = e.with_code(code);
                nom::Err::Failure(p_err)
            }
        }
    }
}

impl<C, I> IntoParserError<nom::Err<ParserError<C, I, ()>>> for nom::Err<TokenizerError<C, I>>
where
    C: Code,
    I: Copy,
{
    fn into_parser_err(self) -> nom::Err<ParserError<C, I, ()>> {
        match self {
            nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            nom::Err::Error(e) => nom::Err::Error(e.into_parser_err()),
            nom::Err::Failure(e) => nom::Err::Failure(e.into_parser_err()),
        }
    }
}

impl<C, I, Y> IntoParserErrorExtra<nom::Err<ParserError<C, I, Y>>, Y>
    for nom::Err<TokenizerError<C, I>>
where
    C: Code,
    I: Copy,
    Y: Copy,
{
    fn into_parser_err_with(self, extra: Y) -> nom::Err<ParserError<C, I, Y>> {
        match self {
            nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            nom::Err::Error(e) => nom::Err::Error(e.into_parser_err_with(extra)),
            nom::Err::Failure(e) => nom::Err::Failure(e.into_parser_err_with(extra)),
        }
    }
}

// ***********************************************************************
// LAYER 3 - wrapped in a Result
// ***********************************************************************

impl<C, I, O> WithCode<C, Result<(I, O), nom::Err<TokenizerError<C, I>>>>
    for Result<(I, O), nom::Err<TokenizerError<C, I>>>
where
    C: Code,
    I: AsBytes + Copy,
{
    fn with_code(self, code: C) -> Result<(I, O), nom::Err<TokenizerError<C, I>>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let p_err: nom::Err<TokenizerError<C, I>> = e.with_code(code);
                Err(p_err)
            }
        }
    }
}

impl<C, I, O> IntoParserError<Result<(I, O), nom::Err<ParserError<C, I, ()>>>>
    for Result<(I, O), nom::Err<TokenizerError<C, I>>>
where
    C: Code,
    I: Copy,
{
    fn into_parser_err(self) -> Result<(I, O), nom::Err<ParserError<C, I, ()>>> {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.into_parser_err())),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.into_parser_err())),
        }
    }
}

impl<C, I, O, Y> IntoParserErrorExtra<Result<(I, O), nom::Err<ParserError<C, I, Y>>>, Y>
    for Result<(I, O), nom::Err<TokenizerError<C, I>>>
where
    C: Code,
    I: Copy,
    Y: Copy,
{
    fn into_parser_err_with(self, extra: Y) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.into_parser_err_with(extra))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.into_parser_err_with(extra))),
        }
    }
}
