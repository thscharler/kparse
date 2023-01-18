//!
//! Additional functionality surrounding nom.
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

use nom::Parser;
use nom_locate::LocatedSpan;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

mod conversion;
pub mod data_frame;
pub mod debug;
mod error;
mod no_context;
mod str_context;
pub mod test;
mod tracker;
mod tracking_context;

use crate::data_frame::str_union;
#[allow(unreachable_pub)]
pub use conversion::*;
pub use data_frame::{SpanIter, SpanLines, StrIter, StrLines};
pub use error::{AppendParserError, Hints, Nom, ParserError, SpanAndCode};
pub use no_context::NoContext;
pub use str_context::StrContext;
#[allow(unreachable_pub)]
pub use tracker::*;
pub use tracking_context::{
    DebugTrack, EnterTrack, ErrTrack, ExitTrack, InfoTrack, OkTrack, Track, TrackingContext,
    WarnTrack,
};

/// Prelude.
/// There are a lot of traits ...
pub mod prelude {
    pub use crate::{error_code, transform};
    pub use crate::{AppendParserError, ParserError, TrackParserError, WithCode, WithSpan};
    pub use crate::{Code, NoContext, ParseContext, StrContext, TrackingContext};
    pub use crate::{Context, ParserNomResult, ParserResult, Span};
}

// sneaky comment

/// Standard input type.
pub type Span<'s, C> = LocatedSpan<&'s str, DynContext<'s, C>>;

/// Result type.
pub type ParserResult<'s, O, C, Y> = Result<(Span<'s, C>, O), nom::Err<ParserError<'s, C, Y>>>;

/// Type alias for a nom parser. Use this to create a ParserError directly in nom.
pub type ParserNomResult<'s, C, Y> =
    Result<(Span<'s, C>, Span<'s, C>), nom::Err<ParserError<'s, C, Y>>>;

/// Parser state codes.
///
/// These are used for error handling and parser results and
/// everything else.
pub trait Code: Copy + Display + Debug + Eq {
    /// Default error code for nom-errors.
    const NOM_ERROR: Self;
}

///
/// Context and tracking for a parser.
///
pub trait ParseContext<'s, C: Code> {
    /// Returns a span that encloses all of the current parser.
    fn original(&self, span: &Span<'s, C>) -> Span<'s, C>;

    /// Tracks entering a parser function.
    fn enter(&self, func: C, span: &Span<'s, C>);

    /// Debugging
    fn debug(&self, span: &Span<'s, C>, debug: String);

    /// Track something.
    fn info(&self, span: &Span<'s, C>, info: &'static str);

    /// Track something more important.
    fn warn(&self, span: &Span<'s, C>, warn: &'static str);

    /// Tracks an Ok result of a parser function.
    fn exit_ok(&self, span: &Span<'s, C>, parsed: &Span<'s, C>);

    /// Tracks an Err result of a parser function.    
    fn exit_err(&self, span: &Span<'s, C>, code: C, err: &dyn Error);
}

/// Hold the context.
/// Needed to block the debug implementation for LocatedSpan.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DynContext<'s, C: Code>(Option<&'s dyn ParseContext<'s, C>>);

impl<'s, C: Code> Debug for DynContext<'s, C> {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

///
/// Makes the Context hidden in the Span more accessible.
///
pub struct Context;

impl Context {
    /// Creates an Ok-Result from the parameters.
    /// Tracks an exit_ok with the ParseContext.
    pub fn ok<'s, C: Code, T, Y: Copy>(
        &self,
        remainder: Span<'s, C>,
        parsed: Span<'s, C>,
        value: T,
    ) -> ParserResult<'s, T, C, Y> {
        Context.exit_ok(&remainder, &parsed);
        Ok((remainder, value))
    }

    /// Creates a Err-ParserResult from the given ParserError.
    /// Tracks an exit_err with the ParseContext.
    pub fn err<'s, C: Code, T, Y: Copy, E: Into<nom::Err<ParserError<'s, C, Y>>>>(
        &self,
        err: E,
    ) -> ParserResult<'s, T, C, Y> {
        let err: nom::Err<ParserError<'s, C, Y>> = err.into();
        match &err {
            nom::Err::Incomplete(_) => {}
            nom::Err::Error(e) => Context.exit_err(&e.span, e.code, &e),
            nom::Err::Failure(e) => Context.exit_err(&e.span, e.code, &e),
        }
        Err(err)
    }

    /// Returns the union of the two Spans
    ///
    /// # Safety
    /// There are assertions that the offsets for the result are within the
    /// bounds of the original().
    ///
    /// But it can't be assured that first and second are derived from it,
    /// so UB cannot be ruled out.
    ///
    /// So the prerequisite is that both first and second are derived from original().
    pub unsafe fn span_union<'a, 'b, C: Code>(
        &self,
        first: &Span<'a, C>,
        second: &Span<'b, C>,
    ) -> Span<'b, C> {
        // take the second argument. both should return the same original

        // but if we use ()-Context the original might be truncated before the second fragment.
        // it's not possible to extend the buffer towards the end, but with LocatedSpan it's
        // always possible to extend to the very beginning. so if we take the second span here
        // it will always include the first span too.
        let original = Context.original(second);
        let str = str_union(original.fragment(), first.fragment(), second.fragment());

        Span::new_from_raw_offset(
            first.location_offset(),
            first.location_line(),
            str,
            second.extra,
        )
    }

    /// Returns the original string for the parser.
    pub fn original<'s, C: Code>(&self, span: &Span<'s, C>) -> Span<'s, C> {
        match span.extra.0 {
            Some(ctx) => ctx.original(span),
            None => NoContext.original(span),
        }
    }

    /// Enter a parser function. For tracking.
    pub fn enter<'s, C: Code>(&self, func: C, span: &Span<'s, C>) {
        if let Some(ctx) = span.extra.0 {
            ctx.enter(func, span)
        }
    }

    /// Track some debug info.
    pub fn debug<'s, C: Code>(&self, span: &Span<'s, C>, debug: String) {
        if let Some(ctx) = span.extra.0 {
            ctx.debug(span, debug)
        }
    }

    /// Track some other info.
    pub fn info<'s, C: Code>(&self, span: &Span<'s, C>, info: &'static str) {
        if let Some(ctx) = span.extra.0 {
            ctx.info(span, info)
        }
    }

    /// Track some warning.
    pub fn warn<'s, C: Code>(&self, span: &Span<'s, C>, warn: &'static str) {
        if let Some(ctx) = span.extra.0 {
            ctx.warn(span, warn)
        }
    }

    /// Calls exit_ok() on the ParseContext. You might want to use ok() instead.
    pub fn exit_ok<'s, C: Code>(&self, span: &Span<'s, C>, parsed: &Span<'s, C>) {
        if let Some(ctx) = span.extra.0 {
            ctx.exit_ok(span, parsed)
        }
    }

    /// Calls exit_err() on the ParseContext. You might want to use err() instead.
    pub fn exit_err<'s, C: Code>(&self, span: &Span<'s, C>, code: C, err: &dyn Error) {
        if let Some(ctx) = span.extra.0 {
            ctx.exit_err(span, code, err)
        }
    }
}

/// Tracks the error path with the context.
pub trait TrackParserError<'s, 't, C: Code, Y: Copy> {
    /// Result type of the track fn.
    type Result;

    /// Track if this is an error.
    fn track(self) -> Self::Result;

    /// Track if this is an error. Set a new code too.
    fn track_as(self, code: C) -> Self::Result;

    /// Track if this is an error. And if this is ok too.
    fn track_ok(self, parsed: Span<'s, C>) -> Self::Result;
}

/// Convert an external error into a ParserError.
pub trait WithSpan<'s, C: Code, R> {
    /// Convert an external error into a ParserError.
    /// Usually uses nom::Err::Failure to indicate the finality of the error.
    fn with_span(self, code: C, span: Span<'s, C>) -> R;
}

/// Translate the error code to a new one.
pub trait WithCode<C: Code, R> {
    /// Translate the error code to a new one.
    fn with_code(self, code: C) -> R;
}

/// Make the trait WithCode work as a parser.
struct ErrorCode<C: Code, P, E1, E2> {
    code: C,
    parser: P,
    _phantom: PhantomData<(E1, E2)>,
}

impl<C: Code, P, E1, E2> ErrorCode<C, P, E1, E2> {
    fn new(parser: P, code: C) -> Self {
        Self {
            code,
            parser,
            _phantom: Default::default(),
        }
    }
}

/// Takes a parser and converts the error via the WithCode trait.
pub fn error_code<'s, O, C, P, E1, E2>(
    parser: P,
    code: C,
) -> impl FnMut(Span<'s, C>) -> Result<(Span<'s, C>, O), nom::Err<E2>>
where
    C: Code + 's,
    E1: WithCode<C, E2>,
    P: Parser<Span<'s, C>, O, E1>,
{
    let mut a = ErrorCode::new(parser, code);
    move |s: Span<'s, C>| a.parse(s)
}

impl<'s, O, C, P, E1, E2> Parser<Span<'s, C>, O, E2> for ErrorCode<C, P, E1, E2>
where
    C: Code,
    E1: WithCode<C, E2>,
    P: Parser<Span<'s, C>, O, E1>,
{
    fn parse(&mut self, input: Span<'s, C>) -> Result<(Span<'s, C>, O), nom::Err<E2>> {
        let r = self.parser.parse(input);
        match r {
            Ok(v) => Ok(v),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_code(self.code))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.with_code(self.code))),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}

struct Transform<'s, O, C, P, T, E0, E1, E2> {
    code: C,
    parser: P,
    transform: T,
    _phantom: PhantomData<&'s (O, E0, E1, E2)>,
}

impl<'s, O, C, P, T, E0, E1, E2> Transform<'s, O, C, P, T, E0, E1, E2>
where
    O: 's,
    C: Code + 's,
    E0: WithCode<C, E2> + 's,
    E1: WithSpan<'s, C, nom::Err<E2>> + 's,
    E2: 's,
    P: Parser<Span<'s, C>, Span<'s, C>, E0>,
    T: Fn(Span<'s, C>) -> Result<O, E1>,
{
    fn new(parser: P, transform: T, code: C) -> Self {
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
/// Same as Transform but only returns the converted output.
pub fn transform<'s, O, C, P, T, E0, E1, E2>(
    parser: P,
    transform: T,
    code: C,
) -> impl FnMut(Span<'s, C>) -> Result<(Span<'s, C>, O), nom::Err<E2>>
where
    O: 's,
    C: Code + 's,
    E0: WithCode<C, E2> + 's,
    E1: WithSpan<'s, C, nom::Err<E2>> + 's,
    E2: 's,
    P: Parser<Span<'s, C>, Span<'s, C>, E0>,
    T: Fn(Span<'s, C>) -> Result<O, E1>,
{
    let mut t = Transform::new(parser, transform, code);
    move |s: Span<'s, C>| -> Result<(Span<'s, C>, O), nom::Err<E2>> { t.parse(s) }
}

impl<'s, O, C, P, T, E0, E1, E2> Parser<Span<'s, C>, O, E2>
    for Transform<'s, O, C, P, T, E0, E1, E2>
where
    C: Code + 's,
    E0: WithCode<C, E2>,
    E1: WithSpan<'s, C, nom::Err<E2>>,
    P: Parser<Span<'s, C>, Span<'s, C>, E0>,
    T: Fn(Span<'s, C>) -> Result<O, E1>,
{
    fn parse(&mut self, input: Span<'s, C>) -> Result<(Span<'s, C>, O), nom::Err<E2>> {
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
