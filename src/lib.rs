//!
//! Adds tracking functions to a nom parser.
//!
//!
//! ```rust no_run
//! use std::fmt::{Display, Formatter};
//! use nom::bytes::complete::{tag_no_case, take_while1};
//! use nom::combinator::recognize;
//! use nom::{AsChar, InputTakeAtPosition};
//! use nom::character::complete::{char as nchar};
//! use nom::multi::many_m_n;
//! use nom::sequence::terminated;
//! use kparse::prelude::*;
//! use kparse::{Code, Context, TrackingContext, TrackParserError};
//! use kparse::spans::SpanExt;
//!
//! fn run_parser() {
//!     let src = "...".to_string();
//!
//!     let ctx = TrackingContext::new(true);
//!     let span = ctx.span(src.as_ref());
//!
//!     match parse_plan(span) {
//!         Ok(v) => {
//!             // ...
//!         }
//!         Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
//!             println!("{:?}", ctx.results());
//!         }
//!         _ => {}
//!     }
//! }
//!
//! pub fn parse_plan(input: APSpan<'_>) -> APParserResult<'_, APPlan<'_>> {
//!     Context.enter(APCPlan, &input);
//!
//!     let (rest, h0) = nom_header(input).track_as(APCHeader)?;
//!     let (rest, _) = nom_tag_plan(rest).track_as(APCPlan)?;
//!     let (rest, plan) = token_name(rest).track()?;
//!     let (rest, h1) = nom_header(rest).track_as(APCHeader)?;
//!
//!     let span = input.span_union(&h0, &h1);
//!     
//!     Context.ok(rest, span, APPlan { name: plan, span })
//! }
//!
//! #[derive(Clone, Copy, PartialEq, Eq, Debug)]
//! pub enum APCode {
//!     APCNomError,
//!
//!     APCHeader,
//!     APCPlan,
//!     APCName,
//! }
//!
//! use APCode::*;
//!
//! impl Code for APCode {
//!     const NOM_ERROR: Self = Self::APCNomError;
//! }
//!
//! pub type APSpan<'s> = kparse::Span<'s, &'s str, APCode>;
//! pub type APParserResult<'s, O> = kparse::ParserResult<'s, O, &'s str, APCode, ()>;
//! pub type APNomResult<'s> = kparse::ParserNomResult<'s, &'s str, APCode, ()>;
//!
//!
//! pub struct APPlan<'s> {
//!     pub name: APName<'s>,
//!     pub span: APSpan<'s>,
//! }
//!
//! pub struct APName<'s> {
//!     pub span: APSpan<'s>,
//! }
//!
//! pub fn nom_header(i: APSpan<'_>) -> APNomResult<'_> {
//!     terminated(recognize(many_m_n(0, 6, nchar('='))), nom_ws)(i)
//! }
//!
//! pub fn nom_tag_plan(i: APSpan<'_>) -> APNomResult<'_> {
//!     terminated(recognize(tag_no_case("plan")), nom_ws)(i)
//! }
//!
//! pub fn token_name(rest: APSpan<'_>) -> APParserResult<'_, APName<'_>> {
//!     match nom_name(rest) {
//!         Ok((rest, tok)) => Ok((rest, APName { span: tok })),
//!         Err(e) => Err(e.with_code(APCName)),
//!     }
//! }
//!
//! pub fn nom_name(i: APSpan<'_>) -> APNomResult<'_> {
//!     terminated(
//!         recognize(take_while1(|c: char| {
//!             c.is_alphanumeric() || "\'+-²/_.".contains(c)
//!         })),
//!         nom_ws,
//!     )(i)
//! }
//!
//! pub fn nom_ws(i: APSpan<'_>) -> APNomResult<'_> {
//!     i.split_at_position_complete(|item| {
//!         let c = item.as_char();
//!         !(c == ' ' || c == '\t')
//!     })
//! }
//!
//! impl Display for APCode {
//!     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Ok(()) }
//! }
//!
//! ```
//!

#![doc(html_root_url = "https://docs.rs/kparse")]
#![warn(absolute_paths_not_starting_with_crate)]
// NO #![warn(box_pointers)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![warn(keyword_idents)]
#![warn(macro_use_extern_crate)]
#![warn(meta_variable_misuse)]
#![warn(missing_abi)]
// NOT_ACCURATE #![warn(missing_copy_implementations)]
// #![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(noop_method_call)]
// NO #![warn(or_patterns_back_compat)]
#![warn(pointer_structural_match)]
#![warn(semicolon_in_expressions_from_macros)]
// NOT_ACCURATE #![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
// #![warn(unsafe_code)]
// #![warn(unsafe_op_in_unsafe_fn)]
#![warn(unstable_features)]
// NO #![warn(unused_crate_dependencies)]
// NO #![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
// NO #![warn(unused_results)]
#![warn(variant_size_differences)]

use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Parser, Slice};
use nom_locate::LocatedSpan;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::{RangeFrom, RangeTo};

pub mod error;
pub mod no_context;
pub mod spans;
pub mod test;
pub mod tracking_context;

mod conversion;
mod debug;

#[allow(unreachable_pub)]
pub use conversion::*;

pub use error::ParserError;
pub use no_context::NoContext;
pub use tracking_context::TrackingContext;

/// Prelude, imports the traits.
pub mod prelude {
    pub use crate::error::AppendParserError;
    pub use crate::{ResultWithSpan, TrackParserError, WithCode, WithSpan};
}

/// Standard input type.
pub type Span<'s, T, C> = LocatedSpan<T, DynContext<'s, T, C>>;

/// Result type.
pub type ParserResult<'s, O, T, C, Y> =
    Result<(Span<'s, T, C>, O), nom::Err<ParserError<'s, T, C, Y>>>;

/// Type alias for a nom parser. Use this to create a ParserError directly in nom.
pub type ParserNomResult<'s, T, C, Y> =
    Result<(Span<'s, T, C>, Span<'s, T, C>), nom::Err<ParserError<'s, T, C, Y>>>;

/// Parser state codes.
///
/// These are used for error handling and parser results and
/// everything else.
pub trait Code: Copy + Display + Debug + Eq {
    /// Default error code for nom-errors.
    const NOM_ERROR: Self;
}

/// This trait defines the tracking functions.
///
/// Create an [TrackingContext] or a [NoContext] before starting the
/// first parser function.
///
/// This trait is not used directly, use the functions of [Context].
///
pub trait ParseContext<'s, T, C: Code> {
    /// Tracks entering a parser function.
    fn enter(&self, func: C, span: &Span<'s, T, C>);

    /// Debugging
    fn debug(&self, span: &Span<'s, T, C>, debug: String);

    /// Track something.
    fn info(&self, span: &Span<'s, T, C>, info: &'static str);

    /// Track something more important.
    fn warn(&self, span: &Span<'s, T, C>, warn: &'static str);

    /// Tracks an Ok result of a parser function.
    fn exit_ok(&self, span: &Span<'s, T, C>, parsed: &Span<'s, T, C>);

    /// Tracks an Err result of a parser function.    
    fn exit_err(&self, span: &Span<'s, T, C>, code: C, err: &dyn Error);
}

/// An instance of this struct ist kept in the extra field of LocatedSpan.
/// This way it's propagated all the way through the parser.
///
/// Access the tracking functions via [Context].
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DynContext<'s, T, C: Code>(Option<&'s dyn ParseContext<'s, T, C>>);

impl<'s, T, C: Code> Debug for DynContext<'s, T, C> {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

/// Each produced span contains a reference to a [ParseContext] in the extra field.
/// This struct makes using it more accessible.
///
/// ```rust ignore
/// use kparse::{Context, Span};
///
/// fn parse_xyz(span: APSpan<'_>) -> APParserResult<'_, u32> {
///     Context.enter(APCode::APCHeader, &span);
///     // ...
///     Context.ok(span, parsed, v32)
/// }
/// ```
pub struct Context;

impl Context {
    /// Creates an Ok-Result from the parameters.
    /// Tracks an exit_ok with the ParseContext.
    pub fn ok<'s, O, T, C: Code, Y: Copy>(
        &self,
        remainder: Span<'s, T, C>,
        parsed: Span<'s, T, C>,
        value: O,
    ) -> ParserResult<'s, O, T, C, Y> {
        Context.exit_ok(&remainder, &parsed);
        Ok((remainder, value))
    }

    /// Creates a Err-ParserResult from the given ParserError.
    /// Tracks an exit_err with the ParseContext.
    pub fn err<
        's,
        O,
        T: AsBytes + Copy + Debug,
        C: Code,
        Y: Copy,
        E: Into<nom::Err<ParserError<'s, T, C, Y>>>,
    >(
        &self,
        err: E,
    ) -> ParserResult<'s, O, T, C, Y>
    where
        T: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
    {
        let err: nom::Err<ParserError<'s, T, C, Y>> = err.into();
        match &err {
            nom::Err::Incomplete(_) => {}
            nom::Err::Error(e) => Context.exit_err(&e.span, e.code, &e),
            nom::Err::Failure(e) => Context.exit_err(&e.span, e.code, &e),
        }
        Err(err)
    }

    /// Enter a parser function. For tracking.
    pub fn enter<'s, T, C: Code>(&self, func: C, span: &Span<'s, T, C>) {
        if let Some(ctx) = span.extra.0 {
            ctx.enter(func, span)
        }
    }

    /// Track some debug info.
    pub fn debug<'s, T, C: Code>(&self, span: &Span<'s, T, C>, debug: String) {
        if let Some(ctx) = span.extra.0 {
            ctx.debug(span, debug)
        }
    }

    /// Track some other info.
    pub fn info<'s, T, C: Code>(&self, span: &Span<'s, T, C>, info: &'static str) {
        if let Some(ctx) = span.extra.0 {
            ctx.info(span, info)
        }
    }

    /// Track some warning.
    pub fn warn<'s, T, C: Code>(&self, span: &Span<'s, T, C>, warn: &'static str) {
        if let Some(ctx) = span.extra.0 {
            ctx.warn(span, warn)
        }
    }

    /// Calls exit_ok() on the ParseContext. You might want to use ok() instead.
    pub fn exit_ok<'s, T, C: Code>(&self, span: &Span<'s, T, C>, parsed: &Span<'s, T, C>) {
        if let Some(ctx) = span.extra.0 {
            ctx.exit_ok(span, parsed)
        }
    }

    /// Calls exit_err() on the ParseContext. You might want to use err() instead.
    pub fn exit_err<'s, T, C: Code>(&self, span: &Span<'s, T, C>, code: C, err: &dyn Error) {
        if let Some(ctx) = span.extra.0 {
            ctx.exit_err(span, code, err)
        }
    }
}

/// This trait is used for error tracking.
///
/// The methods can be squeezed between a parser fn call and the ?.
/// This is equivalent to a Context.err() call.
/// There is one wide implementation, no need to implement this.
///
/// ```rust ignore
/// let (rest, h0) = nom_header(input).track_as(APCHeader)?;
/// let (rest, _) = nom_tag_plan(rest).track_as(APCPlan)?;
/// let (rest, plan) = token_name(rest).track()?;
/// let (rest, h1) = nom_header(rest).track_as(APCHeader)?;
/// ```
pub trait TrackParserError<'s, 't, T, C: Code, Y: Copy> {
    /// Result type of the track fn.
    type Result;

    /// Track if this is an error.
    fn track(self) -> Self::Result;

    /// Track if this is an error. Set a new code too.
    fn track_as(self, code: C) -> Self::Result;

    /// Track if this is an error. And if this is ok too.
    fn track_ok(self, parsed: Span<'s, T, C>) -> Self::Result;
}

/// Convert an external error into a ParserError and add an error code and a span.
pub trait WithSpan<'s, T, C: Code, Y: Copy> {
    /// Convert an external error into a ParserError.
    /// Usually uses nom::Err::Failure to indicate the finality of the error.
    fn with_span(self, code: C, span: Span<'s, T, C>) -> nom::Err<ParserError<'s, T, C, Y>>;
}

/// Convert an external error into a ParserError and add an error code and a span.
/// This trait is used internally and works in conjunction with WithSpan.
/// Rather use [WithSpan]
pub trait ResultWithSpan<'s, T, C: Code, R> {
    /// Convert an external error into a ParserError.
    /// Usually uses nom::Err::Failure to indicate the finality of the error.
    fn with_span(self, code: C, span: Span<'s, T, C>) -> R;
}

/// Translate the error code to a new one.
/// This is implemented for Result<O, E> where E is a WithCode. No need to unwrap.
///
/// To convert an external error to a ParserError rather use [WithSpan] or [std::convert::From].
pub trait WithCode<C: Code, R> {
    /// Translate the error code to a new one.
    fn with_code(self, code: C) -> R;
}

/// Make the trait WithCode work as a parser.
struct ErrorCode<C: Code, Y, PA, E1> {
    code: C,
    parser: PA,
    _phantom: PhantomData<(Y, E1)>,
}

impl<C: Code, Y, PA, E1> ErrorCode<C, Y, PA, E1> {
    fn new(parser: PA, code: C) -> Self {
        Self {
            code,
            parser,
            _phantom: Default::default(),
        }
    }
}

/// Takes a parser and converts the error via the WithCode trait.
pub fn error_code<'s, O, T, C, Y, PA, E1>(
    parser: PA,
    code: C,
) -> impl FnMut(Span<'s, T, C>) -> Result<(Span<'s, T, C>, O), nom::Err<ParserError<'s, T, C, Y>>>
where
    T: AsBytes + Copy + 's,
    C: Code + 's,
    Y: Copy + 's,
    E1: WithCode<C, ParserError<'s, T, C, Y>>,
    PA: Parser<Span<'s, T, C>, O, E1>,
{
    let mut a = ErrorCode::new(parser, code);
    move |s: Span<'s, T, C>| a.parse(s)
}

impl<'s, O, T, C, Y, PA, E1> Parser<Span<'s, T, C>, O, ParserError<'s, T, C, Y>>
    for ErrorCode<C, Y, PA, E1>
where
    T: AsBytes + Copy + 's,
    C: Code + 's,
    Y: Copy + 's,
    E1: WithCode<C, ParserError<'s, T, C, Y>>,
    PA: Parser<Span<'s, T, C>, O, E1>,
{
    fn parse(
        &mut self,
        input: Span<'s, T, C>,
    ) -> Result<(Span<'s, T, C>, O), nom::Err<ParserError<'s, T, C, Y>>> {
        let r = self.parser.parse(input);
        match r {
            Ok(v) => Ok(v),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_code(self.code))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.with_code(self.code))),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}

struct Transform<'s, O, T, C, Y, PA, TRFn, E0, E1> {
    code: C,
    parser: PA,
    transform: TRFn,
    _phantom: PhantomData<&'s (O, T, Y, E0, E1)>,
}

impl<'s, O, T, C, Y, PA, TRFn, E0, E1> Transform<'s, O, T, C, Y, PA, TRFn, E0, E1>
where
    O: 's,
    T: AsBytes + Copy + 's,
    C: Code + 's,
    Y: Copy + 's,
    E0: WithCode<C, ParserError<'s, T, C, Y>> + 's,
    E1: WithSpan<'s, T, C, Y> + 's,
    PA: Parser<Span<'s, T, C>, Span<'s, T, C>, E0>,
    TRFn: Fn(Span<'s, T, C>) -> Result<O, E1>,
{
    fn new(parser: PA, transform: TRFn, code: C) -> Self {
        Self {
            code,
            parser,
            transform,
            _phantom: Default::default(),
        }
    }
}

/// Takes a parser and a transformation of the parser result.
/// Maps any error to the given error code.
///
/// ```rust ignore
/// use nom::combinator::consumed;
/// use nom::Parser;
/// use kparse::{TrackParserError, transform};
///
/// let (rest, (tok, val)) =
///         consumed(transform(nom_parse_c, |v| (*v).parse::<u32>(), ICInteger))(rest).track()?;
/// ```
pub fn transform<'s, O, T, C, Y, PA, TRFn, E0, E1>(
    parser: PA,
    transform: TRFn,
    code: C,
) -> impl FnMut(Span<'s, T, C>) -> Result<(Span<'s, T, C>, O), nom::Err<ParserError<'s, T, C, Y>>>
where
    O: 's,
    T: AsBytes + Copy + 's,
    C: Code + 's,
    Y: Copy + 's,
    E0: WithCode<C, ParserError<'s, T, C, Y>> + 's,
    E1: WithSpan<'s, T, C, Y> + 's,
    PA: Parser<Span<'s, T, C>, Span<'s, T, C>, E0>,
    TRFn: Fn(Span<'s, T, C>) -> Result<O, E1>,
{
    let mut t = Transform::new(parser, transform, code);
    move |s: Span<'s, T, C>| -> Result<(Span<'s, T, C>, O), nom::Err<ParserError<'s, T, C, Y>>> {
        t.parse(s)
    }
}

impl<'s, O, T, C, Y, PA, TRFn, E0, E1> Parser<Span<'s, T, C>, O, ParserError<'s, T, C, Y>>
    for Transform<'s, O, T, C, Y, PA, TRFn, E0, E1>
where
    T: AsBytes + Copy + 's,
    C: Code + 's,
    Y: Copy + 's,
    E0: WithCode<C, ParserError<'s, T, C, Y>>,
    E1: WithSpan<'s, T, C, Y>,
    PA: Parser<Span<'s, T, C>, Span<'s, T, C>, E0>,
    TRFn: Fn(Span<'s, T, C>) -> Result<O, E1>,
{
    fn parse(
        &mut self,
        input: Span<'s, T, C>,
    ) -> Result<(Span<'s, T, C>, O), nom::Err<ParserError<'s, T, C, Y>>> {
        let r = self.parser.parse(input);
        match r {
            Ok((rest, token)) => {
                let o = (self.transform)(token);
                match o {
                    Ok(o) => Ok((rest, o)),
                    Err(e) => Err(e.with_span(self.code, token)),
                }
            }
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_code(self.code))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.with_code(self.code))),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}
