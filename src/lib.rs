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
//! use kparse::spans::LocatedSpanExt;
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
//! pub type APSpan<'s> = kparse::CtxSpan<'s, &'s str, APCode>;
//! pub type APParserResult<'s, O> = kparse::CtxParserResult<'s, O, &'s str, APCode, ()>;
//! pub type APNomResult<'s> = kparse::CtxParserNomResult<'s, &'s str, APCode, ()>;
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
    pub use crate::spans::LocatedSpanExt;
    pub use crate::{ResultWithSpan, TrackParserError, WithCode, WithSpan};
}

/// Standard input type.
///
/// It holds a dyn ParseContext in the extra field of LocatedSpan to distribute
/// the context.
pub type CtxSpan<'s, T, C> = LocatedSpan<T, DynContext<'s, T, C>>; // todo order

/// Standard result type in conjunction with CtxSpan.
pub type CtxParserResult<'s, O, T, C, Y> =
    Result<(CtxSpan<'s, T, C>, O), nom::Err<ParserError<C, CtxSpan<'s, T, C>, Y>>>; // todo order

/// Type alias for a nom parser. Use this to create a ParserError directly in nom.
pub type CtxParserNomResult<'s, T, C, Y> =
    Result<(CtxSpan<'s, T, C>, CtxSpan<'s, T, C>), nom::Err<ParserError<C, CtxSpan<'s, T, C>, Y>>>; // todo order

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
pub trait ParseContext<T, C>
where
    C: Code,
{
    /// Tracks entering a parser function.
    fn enter(&self, func: C, span: &LocatedSpan<T, ()>);

    /// Debugging
    fn debug(&self, span: &LocatedSpan<T, ()>, debug: String);

    /// Track something.
    fn info(&self, span: &LocatedSpan<T, ()>, info: &'static str);

    /// Track something more important.
    fn warn(&self, span: &LocatedSpan<T, ()>, warn: &'static str);

    /// Tracks an Ok result of a parser function.
    fn exit_ok(&self, span: &LocatedSpan<T, ()>, parsed: &LocatedSpan<T, ()>);

    /// Tracks an Err result of a parser function.    
    fn exit_err(&self, span: &LocatedSpan<T, ()>, code: C, err: &dyn Error);
}

/// An instance of this struct ist kept in the extra field of LocatedSpan.
/// This way it's propagated all the way through the parser.
///
/// Access the tracking functions via [Context].
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DynContext<'c, T, C>(Option<&'c dyn ParseContext<T, C>>)
where
    C: Code;

impl<'c, T, C> Debug for DynContext<'c, T, C>
where
    C: Code,
{
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
    pub fn ok<'s, O, T, C, Y>(
        &self,
        remainder: CtxSpan<'s, T, C>,
        parsed: CtxSpan<'s, T, C>,
        value: O,
    ) -> CtxParserResult<'s, O, T, C, Y>
    where
        T: AsBytes + Copy,
        C: Code,
        Y: Copy,
    {
        Context.exit_ok(&remainder, &parsed);
        Ok((remainder, value))
    }

    /// Creates a Err-ParserResult from the given ParserError.
    /// Tracks an exit_err with the ParseContext.
    pub fn err<'s, O, T, C, Y, E>(&self, err: E) -> CtxParserResult<'s, O, T, C, Y>
    where
        E: Into<nom::Err<ParserError<C, CtxSpan<'s, T, C>, Y>>>,
        C: Code,
        Y: Copy,
        T: Copy + Debug,
        T: Offset
            + InputTake
            + InputIter
            + InputLength
            + AsBytes
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
    {
        let err: nom::Err<ParserError<C, CtxSpan<'s, T, C>, Y>> = err.into();
        match &err {
            nom::Err::Incomplete(_) => {}
            nom::Err::Error(e) => Context.exit_err(&e.span, e.code, &e),
            nom::Err::Failure(e) => Context.exit_err(&e.span, e.code, &e),
        }
        Err(err)
    }

    fn clear_span<'s, T, C>(span: &CtxSpan<'s, T, C>) -> LocatedSpan<T, ()>
    where
        T: AsBytes + Copy + 's,
        C: Code,
    {
        unsafe {
            LocatedSpan::new_from_raw_offset(
                span.location_offset(),
                span.location_line(),
                *span.fragment(),
                (),
            )
        }
    }

    /// Enter a parser function. For tracking.
    pub fn enter<'s, T, C>(&self, func: C, span: &CtxSpan<'s, T, C>)
    where
        T: AsBytes + Copy,
        C: Code,
    {
        if let Some(ctx) = span.extra.0 {
            ctx.enter(func, &Self::clear_span(span))
        }
    }

    /// Track some debug info.
    pub fn debug<'s, T, C: Code>(&self, span: &CtxSpan<'s, T, C>, debug: String)
    where
        T: AsBytes + Copy,
        C: Code,
    {
        if let Some(ctx) = span.extra.0 {
            ctx.debug(&Self::clear_span(span), debug)
        }
    }

    /// Track some other info.
    pub fn info<'s, T, C: Code>(&self, span: &CtxSpan<'s, T, C>, info: &'static str)
    where
        T: AsBytes + Copy,
        C: Code,
    {
        if let Some(ctx) = span.extra.0 {
            ctx.info(&Self::clear_span(span), info)
        }
    }

    /// Track some warning.
    pub fn warn<'s, T, C: Code>(&self, span: &CtxSpan<'s, T, C>, warn: &'static str)
    where
        T: AsBytes + Copy,
        C: Code,
    {
        if let Some(ctx) = span.extra.0 {
            ctx.warn(&Self::clear_span(span), warn)
        }
    }

    /// Calls exit_ok() on the ParseContext. You might want to use ok() instead.
    pub fn exit_ok<'s, T, C: Code>(&self, span: &CtxSpan<'s, T, C>, parsed: &CtxSpan<'s, T, C>)
    where
        T: AsBytes + Copy,
        C: Code,
    {
        if let Some(ctx) = span.extra.0 {
            ctx.exit_ok(&Self::clear_span(span), &Self::clear_span(parsed))
        }
    }

    /// Calls exit_err() on the ParseContext. You might want to use err() instead.
    pub fn exit_err<'s, T, C>(&self, span: &CtxSpan<'s, T, C>, code: C, err: &dyn Error)
    where
        T: AsBytes + Copy,
        C: Code,
    {
        if let Some(ctx) = span.extra.0 {
            ctx.exit_err(&Self::clear_span(span), code, err)
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
pub trait TrackParserError<'s, C, I, Y, O, E>
where
    C: Code,
    I: AsBytes + Copy + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + AsBytes
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    Y: Copy,
    E: Into<ParserError<C, I, Y>>,
    Self: Into<Result<(I, O), nom::Err<E>>>,
{
    /// Track if this is an error.
    fn track(self) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        let ego = self.into();
        match ego {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<C, I, Y> = e.into();
                Self::exit_err(p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    /// Track if this is an error. Set a new code too.
    fn track_as(self, code: C) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        let ego = self.into();
        match ego {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<C, I, Y> = e.into();
                let p_err = p_err.with_code(code);
                Self::exit_err(p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    /// Track if this is an error. And if this is ok too.
    fn track_ok(self, parsed: I) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        let ego = self.into();
        match ego {
            Ok((span, v)) => {
                Self::exit_ok(parsed, span);
                Ok((span, v))
            }
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<C, I, Y> = e.into();
                Self::exit_err(p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    fn exit_ok(span: I, parsed: I);

    fn exit_err(span: I, code: C, err: &dyn Error);
}

/// Convert an external error into a ParserError and add an error code and a span.
pub trait WithSpan<C: Code, I, E> {
    /// Convert an external error into a ParserError.
    /// Usually uses nom::Err::Failure to indicate the finality of the error.
    fn with_span(self, code: C, span: I) -> nom::Err<E>;
}

/// Convert an external error into a ParserError and add an error code and a span.
/// This trait is used internally and works in conjunction with WithSpan.
/// Rather use [WithSpan]
pub trait ResultWithSpan<C: Code, I, R> {
    /// Convert an external error into a ParserError.
    /// Usually uses nom::Err::Failure to indicate the finality of the error.
    fn with_span(self, code: C, span: I) -> R;
}

/// Translate the error code to a new one.
/// This is implemented for Result<O, E> where E is a WithCode. No need to unwrap.
///
/// To convert an external error to a ParserError rather use [WithSpan] or [std::convert::From].
pub trait WithCode<C: Code, R> {
    /// Translate the error code to a new one.
    fn with_code(self, code: C) -> R;
}

/// Takes a parser and converts the error via the WithCode trait.
pub fn error_code<PA, C, I, O, E0, E1>(
    mut parser: PA,
    code: C,
) -> impl FnMut(I) -> Result<(I, O), nom::Err<E1>>
where
    C: Code,
    PA: Parser<I, O, E0>,
    E0: WithCode<C, E1>,
{
    move |i| -> Result<(I, O), nom::Err<E1>> {
        match parser.parse(i) {
            Ok((r, v)) => Ok((r, v)),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_code(code))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Error(e.with_code(code))),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
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
pub fn transform<PA, TRFn, C, I, O1, O2, E0, E1, E2>(
    mut parser: PA,
    transform: TRFn,
    code: C,
) -> impl FnMut(I) -> Result<(I, O2), nom::Err<E2>>
where
    C: Code,
    O1: Copy,
    PA: Parser<I, O1, E0>,
    TRFn: Fn(O1) -> Result<O2, E1>,
    E0: WithCode<C, E2>,
    E1: WithSpan<C, O1, E2>,
{
    move |i| -> Result<(I, O2), nom::Err<E2>> {
        let r = parser.parse(i);
        match r {
            Ok((rest, token)) => {
                let o = transform(token);
                match o {
                    Ok(o) => Ok((rest, o)),
                    Err(e) => Err(e.with_span(code, token)),
                }
            }
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_code(code))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.with_code(code))),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}
