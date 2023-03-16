//!
//! Addons for a nom parser.
//!
//! * A error code trait.
//! * A richer error type ParserError.
//! * A thin error type TokenizerError.
//!
//! * A tracking/logging system for the parser.
//!
//! * A simple framework to test parser functions.
//!
//! * Took some inspiration from nom_supreme and implemented a similar subset of
//!   postfix functions. They are integrated with the error Code and the error types of
//!   this crate.
//!
//! * SourceStr and SourceBytes to get context information around a Span.
//!   Can retrieve line/column information and context data without
//!   burdening every single span.
//!
//! * Uses LocatedSpan for debug builds and replaces with plan `&str` or `&[u8]` for release
//!   builds. Tracking is compiled away completely for release builds.
//!

#![doc(html_root_url = "https://docs.rs/kparse")]
#![warn(absolute_paths_not_starting_with_crate)]
#![allow(box_pointers)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![warn(keyword_idents)]
#![warn(macro_use_extern_crate)]
#![warn(meta_variable_misuse)]
#![warn(missing_abi)]
// #![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(noop_method_call)]
#![warn(pointer_structural_match)]
#![warn(semicolon_in_expressions_from_macros)]
#![allow(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![allow(unsafe_code)]
#![allow(unsafe_op_in_unsafe_fn)]
#![warn(unstable_features)]
#![allow(unused_crate_dependencies)]
#![allow(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![allow(unused_results)]
#![warn(variant_size_differences)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::type_complexity)]

pub mod combinators;
mod debug;
pub mod examples;
pub mod parser_error;
mod parser_ext;
pub mod provider;
pub mod source;
pub mod spans;
pub mod test;
pub mod token_error;

pub use crate::parser_error::ParserError;
pub use crate::token_error::TokenizerError;
use std::borrow::Borrow;

use crate::parser_ext::{
    AllConsuming, Complete, Consumed, Cut, DelimitedBy, FromStrParser, IntoErr, MapRes,
    OptPrecedes, Optional, OrElse, PNot, Peek, Precedes, Recognize, Terminated, Value, Verify,
    WithCode, WithContext,
};
use crate::provider::{StdTracker, TrackData, TrackProvider};
use crate::source::{SourceBytes, SourceStr};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Parser, Slice};
use nom_locate::LocatedSpan;
use std::fmt::{Debug, Display};
use std::ops::RangeTo;
use std::str::FromStr;

/// Prelude for all traits.
pub mod prelude {
    pub use crate::parser_error::AppendParserError;
    pub use crate::provider::TrackProvider;
    pub use crate::source::Source;
    pub use crate::spans::{SpanFragment, SpanUnion};
    pub use crate::test::Report;
    pub use crate::{
        define_span, Code, ErrInto, ErrOrNomErr, KParseError, KParser, ParseSpan, Track,
        TrackResult, TrackedSpan,
    };
}

/// Standard input type. This is a LocatedSpan with a TrackProvider.
pub type DynTrackProvider<'s, C, T> = &'s (dyn TrackProvider<C, T> + Send);
pub type ParseSpan<'s, C, T> = LocatedSpan<T, DynTrackProvider<'s, C, T>>;

/// Defines a type alias for the span type.
/// Switches between ParseSpan<> in debug mode and plain type in release mode.
#[macro_export]
macro_rules! define_span {
    ($v:vis $name:ident = $code:ty, $typ:ty) => {
        #[cfg(debug_assertions)]
        $v type $name<'a> = ParseSpan<'a, $code, &'a $typ>;
        #[cfg(not(debug_assertions))]
        $v type $name<'a> = &'a $typ;
    };
}

/// ParserResult for ParserError.
/// Equivalent to [nom::IResult]<(I, O), ParserError<C, I>>
pub type ParserResult<C, I, O> = Result<(I, O), nom::Err<ParserError<C, I>>>;

/// ParserResult for TokenizerError.  
/// Equivalent to [nom::IResult]<(I, O), TokenizerError<C, I>>
pub type TokenizerResult<C, I, O> = Result<(I, O), nom::Err<TokenizerError<C, I>>>;

/// Parser error code.
pub trait Code: Copy + Display + Debug + Eq + Send {
    /// Default error code for nom-errors.
    const NOM_ERROR: Self;
}

/// This trait catches the essentials for an error type within this library.
///
/// It is implemented for `E`, `nom::Err<E>` and `Result<(I,O), nom::Err<E>>`.
///
// todo: necessary for Result?
pub trait KParseError<C, I> {
    /// The base error type.
    type WrappedError: Debug;

    /// Create a matching error.
    fn from(code: C, span: I) -> Self;

    /// Changes the error code.
    fn with_code(self, code: C) -> Self;

    /// Returns the error code if self is `Result::Err` and it's not `nom::Err::Incomplete`.
    fn code(&self) -> Option<C>;
    /// Returns the error span if self is `Result::Err` and it's not `nom::Err::Incomplete`.
    fn span(&self) -> Option<I>;
    /// Returns the error if self is `Result::Err` and it's not `nom::Err::Incomplete`.
    fn err(&self) -> Option<&Self::WrappedError>;

    /// Returns all the parts if self is `Result::Err` and it's not `nom::Err::Incomplete`.
    fn parts(&self) -> Option<(C, I, &Self::WrappedError)>;
}

/// Analog function for err_into() working on a parser, but working on the Result instead.
pub trait ErrInto<E2> {
    /// Result of the conversion.
    type Result;

    /// Converts the error value of the result.
    fn err_into(self) -> Self::Result;
}

impl<I, O, E1, E2> ErrInto<E2> for Result<(I, O), nom::Err<E1>>
where
    E2: From<E1>,
{
    type Result = Result<(I, O), nom::Err<E2>>;

    fn err_into(self) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.into())),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.into())),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}

/// This trait is used for Track.err() where the function wants to accept both
/// `E` and `nom::Err<E>`.
pub trait ErrOrNomErr {
    /// The base error type.
    type WrappedError: Debug;

    /// Converts self to a `nom::Err` wrapped error.
    fn wrap(self) -> nom::Err<Self::WrappedError>;
}

/// Adds some common parser combinators as postfix operators to parser.
pub trait KParser<I, O, E>
where
    Self: Sized,
{
    /// Converts the error to the target error.
    fn err_into<E2>(self) -> IntoErr<Self, O, E, E2>
    where
        E: Into<E2>;

    /// Changes the error code.
    fn with_code<C>(self, code: C) -> WithCode<Self, C>
    where
        C: Code,
        E: KParseError<C, I>;

    /// Adds some context.
    fn with_context<C, Y>(self, context: Y) -> WithContext<Self, C, E, Y>
    where
        C: Code,
        I: Clone,
        E: Into<ParserError<C, I>>,
        Y: Clone + 'static;

    /// Map the output.
    fn map_res<TR, O2>(self, map: TR) -> MapRes<Self, O, TR, O2>
    where
        TR: Fn(O) -> Result<O2, nom::Err<E>>;

    /// Convert the output with the FromStr trait.
    fn parse_from_str<C, O2>(self, code: C) -> FromStrParser<Self, C, O, O2>
    where
        C: Code,
        O: InputIter<Item = char>,
        O2: FromStr,
        E: KParseError<C, I>;

    /// Replace the output with the value.
    fn value<O2>(self, value: O2) -> Value<Self, O, O2>
    where
        O2: Clone;

    /// Fails if not everything has been processed.
    fn all_consuming<C>(self, code: C) -> AllConsuming<Self, C>
    where
        C: Code,
        I: InputLength,
        E: KParseError<C, I>;

    /// Converts nom::Err::Incomplete to a error code.
    fn complete<C>(self, code: C) -> Complete<Self, C>
    where
        C: Code,
        I: Clone,
        E: KParseError<C, I>;

    /// Convert from nom::Err::Error to nom::Err::Failure
    fn cut(self) -> Cut<Self>;

    /// Optional parser.
    fn opt(self) -> Optional<Self>;

    /// Run the parser and return the parsed input.
    fn recognize(self) -> Recognize<Self, O>
    where
        I: Clone + Slice<RangeTo<usize>> + Offset;

    /// Run the parser and return the parser output and the parsed input.
    fn consumed(self) -> Consumed<Self>
    where
        I: Clone + Slice<RangeTo<usize>> + Offset;

    /// Runs the parser and the terminator and just returns the result of the parser.
    fn terminated<PA, O2>(self, terminator: PA) -> Terminated<Self, PA, O2>
    where
        PA: Parser<I, O2, E>;

    /// Runs the parser and the successor and only returns the result of the
    /// successor.
    fn precedes<PA, O2>(self, successor: PA) -> Precedes<Self, PA, O>
    where
        PA: Parser<I, O2, E>;

    /// Runs the parser and the successor and returns the result of the successor.
    /// The parser itself may fail too.
    fn opt_precedes<PA, O2>(self, successor: PA) -> OptPrecedes<Self, PA, O>
    where
        PA: Parser<I, O2, E>,
        I: Clone;

    /// Runs the delimiter before and after the main parser, and returns just
    /// the result of the main parser.
    fn delimited_by<PA, O2>(self, delimiter: PA) -> DelimitedBy<Self, PA, O2>
    where
        PA: Parser<I, O2, E>;

    /// Runs the parser but doesn't change the input.
    fn peek(self) -> Peek<Self>
    where
        I: Clone;

    /// Fails if the parser succeeds and vice versa.
    fn not<C>(self, code: C) -> PNot<Self, C, O>
    where
        C: Code,
        E: KParseError<C, I>,
        I: Clone;

    /// Or. Returns a `(Option<A>, Option<B>)`
    fn or_else<PE, OE>(self, other: PE) -> OrElse<Self, PE, OE>
    where
        PE: Parser<I, OE, E>;

    /// Runs a verify function on the parser result.
    fn verify<V, C, O2>(self, verify: V, code: C) -> Verify<Self, V, C, O2>
    where
        C: Code,
        V: Fn(&O2) -> bool,
        O: Borrow<O2>,
        O2: ?Sized,
        E: KParseError<C, I>;
}

impl<T, I, O, E> KParser<I, O, E> for T
where
    T: Parser<I, O, E>,
{
    #[inline]
    fn err_into<E2>(self) -> IntoErr<Self, O, E, E2>
    where
        E: Into<E2>,
    {
        IntoErr {
            parser: self,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn with_code<C>(self, code: C) -> WithCode<Self, C>
    where
        C: Code,
        E: KParseError<C, I>,
    {
        WithCode { parser: self, code }
    }

    #[inline]
    fn with_context<C, Y>(self, context: Y) -> WithContext<Self, C, E, Y>
    where
        C: Code,
        I: Clone,
        E: Into<ParserError<C, I>>,
        Y: Clone + 'static,
    {
        WithContext {
            parser: self,
            context,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn map_res<TR, O2>(self, map: TR) -> MapRes<Self, O, TR, O2>
    where
        TR: Fn(O) -> Result<O2, nom::Err<E>>,
    {
        MapRes {
            parser: self,
            map,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn parse_from_str<C, O2>(self, code: C) -> FromStrParser<Self, C, O, O2>
    where
        C: Code,
        O: InputIter<Item = char>,
        O2: FromStr,
        E: KParseError<C, I>,
    {
        FromStrParser {
            parser: self,
            code,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn value<O2>(self, value: O2) -> Value<Self, O, O2>
    where
        O2: Clone,
    {
        Value {
            parser: self,
            value,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn all_consuming<C>(self, code: C) -> AllConsuming<Self, C>
    where
        C: Code,
        I: InputLength,
        E: KParseError<C, I>,
    {
        AllConsuming { parser: self, code }
    }

    #[inline]
    fn complete<C>(self, code: C) -> Complete<Self, C>
    where
        C: Code,
        I: Clone,
        E: KParseError<C, I>,
    {
        Complete { parser: self, code }
    }

    #[inline]
    fn cut(self) -> Cut<Self> {
        Cut { parser: self }
    }

    #[inline]
    fn opt(self) -> Optional<Self> {
        Optional { parser: self }
    }

    #[inline]
    fn recognize(self) -> Recognize<Self, O>
    where
        I: Clone + Slice<RangeTo<usize>> + Offset,
    {
        Recognize {
            parser: self,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn consumed(self) -> Consumed<Self>
    where
        I: Clone + Slice<RangeTo<usize>> + Offset,
    {
        Consumed { parser: self }
    }

    #[inline]
    fn terminated<PA, O2>(self, terminator: PA) -> Terminated<Self, PA, O2>
    where
        PA: Parser<I, O2, E>,
    {
        Terminated {
            parser: self,
            terminator,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn precedes<PS, O2>(self, successor: PS) -> Precedes<Self, PS, O>
    where
        PS: Parser<I, O2, E>,
    {
        Precedes {
            parser: self,
            successor,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn opt_precedes<PS, O2>(self, successor: PS) -> OptPrecedes<Self, PS, O>
    where
        PS: Parser<I, O2, E>,
        I: Clone,
    {
        OptPrecedes {
            parser: self,
            successor,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn delimited_by<PA, O2>(self, delimiter: PA) -> DelimitedBy<Self, PA, O2>
    where
        PA: Parser<I, O2, E>,
    {
        DelimitedBy {
            parser: self,
            delimiter,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn peek(self) -> Peek<Self>
    where
        I: Clone,
    {
        Peek { parser: self }
    }

    #[inline]
    fn not<C>(self, code: C) -> PNot<Self, C, O> {
        PNot {
            parser: self,
            code,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn or_else<PE, OE>(self, other: PE) -> OrElse<Self, PE, OE>
    where
        PE: Parser<I, OE, E>,
    {
        OrElse {
            parser: self,
            other,
            _phantom: Default::default(),
        }
    }

    #[inline]
    fn verify<V, C, O2>(self, verify: V, code: C) -> Verify<Self, V, C, O2>
    where
        C: Code,
        V: Fn(&O2) -> bool,
        O: Borrow<O2>,
        O2: ?Sized,
        E: KParseError<C, I>,
    {
        Verify {
            parser: self,
            verify,
            code,
            _phantom: Default::default(),
        }
    }
}

/// Central struct for tracking.
///
/// - Create a TrackProvider with ```Track::new_tracker()```
/// - Create a matching span with ```new_span()```. Switches between debug and release mode,
///   and tracking is only active in debug mode.
/// - Create a SourceStr/SourceBytes for row/column information.
///
/// - Call the actual tracking functions:
///   - Track.enter(), Track.ok(), Track.err(), ...
///
pub struct Track;

impl Track {
    /// Provider/Container for tracking data.
    pub fn new_tracker<C, I>() -> StdTracker<C, I>
    where
        C: Code,
        I: Clone + Debug + AsBytes,
        I: InputTake + InputLength + InputIter,
    {
        StdTracker::new()
    }

    /// Create a tracking span for the given text and TrackProvider.
    #[cfg(debug_assertions)]
    pub fn new_span<'s, C, I>(
        provider: &'s impl TrackProvider<C, I>,
        text: I,
    ) -> LocatedSpan<I, DynTrackProvider<'s, C, I>>
    where
        C: Code,
        I: Clone + Debug + AsBytes,
        I: InputTake + InputLength + InputIter,
        I: 's,
    {
        provider.track_span(text)
    }

    #[cfg(not(debug_assertions))]
    pub fn new_span<'s, C, I>(_provider: &'s impl TrackProvider<C, I>, text: I) -> I
    where
        C: Code,
        I: Clone + Debug + AsBytes,
        I: InputTake + InputLength + InputIter,
        I: 's,
    {
        text
    }

    /// Create a source text map for the given text.
    pub fn source_str(text: &str) -> SourceStr<'_> {
        SourceStr::new(text)
    }

    /// Create a source text map for the given text.
    pub fn source_bytes(text: &[u8]) -> SourceBytes<'_> {
        SourceBytes::new(text)
    }

    /// Creates an Ok() Result from the parameters and tracks the result.
    #[inline(always)]
    pub fn ok<C, I, O, E>(&self, rest: I, input: I, value: O) -> Result<(I, O), nom::Err<E>>
    where
        C: Code,
        I: Clone + Debug,
        I: TrackedSpan<C>,
        I: InputTake + InputLength + InputIter,
        E: KParseError<C, I> + Debug,
    {
        rest.track_ok(input);
        rest.track_exit();
        Ok((rest, value))
    }

    /// Tracks the error and creates a Result.
    #[inline(always)]
    pub fn err<C, I, O, E>(
        &self,
        err: E,
    ) -> Result<(I, O), nom::Err<<E as ErrOrNomErr>::WrappedError>>
    where
        C: Code,
        I: Clone + Debug,
        I: TrackedSpan<C>,
        I: InputTake + InputLength + InputIter,
        E: KParseError<C, I> + ErrOrNomErr + Debug,
    {
        match err.parts() {
            None => Err(err.wrap()),
            Some((code, span, e)) => {
                span.track_err(code, e);
                span.track_exit();
                Err(err.wrap())
            }
        }
    }

    /// When multiple Context.enter() calls are used within one function
    /// (to denote some separation), this can be used to exit such a compartment
    /// with an ok track.
    #[inline(always)]
    pub fn ok_section<C, I>(&self, rest: I, input: I)
    where
        C: Code,
        I: TrackedSpan<C>,
    {
        rest.track_ok(input);
    }

    /// When multiple Context.enter() calls are used within one function
    /// (to denote some separation), this can be used to exit such a compartment
    /// with an ok track.
    #[inline(always)]
    pub fn err_section<C, I, E>(&self, err: &E)
    where
        C: Code,
        I: Clone + Debug,
        I: TrackedSpan<C>,
        I: InputTake + InputLength + InputIter,
        E: KParseError<C, I> + Debug,
    {
        match err.parts() {
            None => {}
            Some((code, span, e)) => {
                span.track_err(code, e);
            }
        }
    }

    /// Enter a parser function.
    #[inline(always)]
    pub fn enter<C, I>(&self, func: C, span: I)
    where
        C: Code,
        I: TrackedSpan<C>,
    {
        span.track_enter(func);
    }

    /// Track some debug info.
    #[inline(always)]
    pub fn debug<C, I>(&self, span: I, debug: String)
    where
        C: Code,
        I: TrackedSpan<C>,
    {
        span.track_debug(debug);
    }

    /// Track some other info.
    #[inline(always)]
    pub fn info<C, I>(&self, span: I, info: &'static str)
    where
        C: Code,
        I: TrackedSpan<C>,
    {
        span.track_info(info);
    }

    /// Track some warning.
    #[inline(always)]
    pub fn warn<C, I>(&self, span: I, warn: &'static str)
    where
        C: Code,
        I: TrackedSpan<C>,
    {
        span.track_warn(warn);
    }
}

/// This is an extension trait for nom-Results.
///
/// This is for inline tracking of parser results.
///
/// ```rust ignore
/// let (rest, h0) = nom_header(input).track_as(APCHeader)?;
/// let (rest, _) = nom_tag_plan(rest).track_as(APCPlan)?;
/// let (rest, plan) = token_name(rest).track()?;
/// let (rest, h1) = nom_header(rest).track_as(APCHeader)?;
/// ```
pub trait TrackResult<C, I>
where
    C: Code,
    I: Clone + Debug,
    I: TrackedSpan<C>,
    I: InputTake + InputLength + InputIter + AsBytes,
{
    /// Track an Err() result.
    fn track(self) -> Self;

    /// Track an Err() result and modify the error code in one go.
    fn track_as(self, code: C) -> Self;
}

impl<C, I, O, E> TrackResult<C, I> for Result<(I, O), nom::Err<E>>
where
    C: Code,
    I: Clone + Debug,
    I: TrackedSpan<C>,
    I: InputTake + InputLength + InputIter + AsBytes,
    E: Debug,
    nom::Err<E>: KParseError<C, I>,
{
    /// Tracks the result if it is an error.
    #[inline(always)]
    fn track(self) -> Self {
        match self {
            Ok((rest, token)) => Ok((rest, token)),
            Err(e) => match e.parts() {
                None => Err(e),
                Some((code, span, err)) => {
                    span.track_err(code, err);
                    span.track_exit();
                    Err(e)
                }
            },
        }
    }

    /// Changes the error code and tracks the result.
    #[inline(always)]
    fn track_as(self, code: C) -> Self {
        match self {
            Ok((rest, token)) => Ok((rest, token)),
            Err(e) => {
                let e = e.with_code(code);
                match e.parts() {
                    None => Err(e),
                    Some((code, span, err)) => {
                        span.track_err(code, err);
                        span.track_exit();
                        Err(e)
                    }
                }
            }
        }
    }
}

/// This trait is implemented for an input type. It takes a tracking event and
/// its raw data, converts if necessary and sends it to the actual tracker.
pub trait TrackedSpan<C>
where
    C: Code,
    Self: Sized,
{
    /// Enter a parser function.
    fn track_enter(&self, func: C);

    /// Track some debug info.
    fn track_debug(&self, debug: String);

    /// Track some other info.
    fn track_info(&self, info: &'static str);

    /// Track some warning.
    fn track_warn(&self, warn: &'static str);

    /// Calls exit_ok() on the ParseContext. You might want to use ok() instead.
    fn track_ok(&self, parsed: Self);

    /// Calls exit_err() on the ParseContext. You might want to use err() instead.
    fn track_err<E: Debug>(&self, code: C, err: &E);

    /// Calls exit() on the ParseContext. You might want to use err() or ok() instead.
    fn track_exit(&self);
}

impl<'s, C, T> TrackedSpan<C> for LocatedSpan<T, DynTrackProvider<'s, C, T>>
where
    C: Code,
    T: Clone + Debug + AsBytes + InputTake + InputLength,
{
    #[inline(always)]
    fn track_enter(&self, func: C) {
        self.extra.track(TrackData::Enter(func, clear_span(self)));
    }

    #[inline(always)]
    fn track_debug(&self, debug: String) {
        self.extra.track(TrackData::Debug(clear_span(self), debug));
    }

    #[inline(always)]
    fn track_info(&self, info: &'static str) {
        self.extra.track(TrackData::Info(clear_span(self), info));
    }

    #[inline(always)]
    fn track_warn(&self, warn: &'static str) {
        self.extra.track(TrackData::Warn(clear_span(self), warn));
    }

    #[inline(always)]
    fn track_ok(&self, parsed: LocatedSpan<T, DynTrackProvider<'s, C, T>>) {
        self.extra
            .track(TrackData::Ok(clear_span(self), clear_span(&parsed)));
    }

    #[inline(always)]
    fn track_err<E: Debug>(&self, code: C, err: &E) {
        self.extra
            .track(TrackData::Err(clear_span(self), code, format!("{:?}", err)));
    }

    #[inline(always)]
    fn track_exit(&self) {
        self.extra.track(TrackData::Exit());
    }
}

fn clear_span<C, T>(span: &LocatedSpan<T, DynTrackProvider<'_, C, T>>) -> LocatedSpan<T, ()>
where
    C: Code,
    T: AsBytes + Clone,
{
    unsafe {
        LocatedSpan::new_from_raw_offset(
            span.location_offset(),
            span.location_line(),
            span.fragment().clone(),
            (),
        )
    }
}

impl<C, T> TrackedSpan<C> for LocatedSpan<T, ()>
where
    T: Clone + Debug,
    T: InputTake + InputLength + AsBytes,
    C: Code,
{
    #[inline(always)]
    fn track_enter(&self, _func: C) {}

    #[inline(always)]
    fn track_debug(&self, _debug: String) {}

    #[inline(always)]
    fn track_info(&self, _info: &'static str) {}

    #[inline(always)]
    fn track_warn(&self, _warn: &'static str) {}

    #[inline(always)]
    fn track_ok(&self, _parsed: LocatedSpan<T, ()>) {}

    #[inline(always)]
    fn track_err<E>(&self, _func: C, _err: &E) {}

    #[inline(always)]
    fn track_exit(&self) {}
}

impl<'s, C> TrackedSpan<C> for &'s str
where
    C: Code,
{
    #[inline(always)]
    fn track_enter(&self, _func: C) {}

    #[inline(always)]
    fn track_debug(&self, _debug: String) {}

    #[inline(always)]
    fn track_info(&self, _info: &'static str) {}

    #[inline(always)]
    fn track_warn(&self, _warn: &'static str) {}

    #[inline(always)]
    fn track_ok(&self, _input: Self) {}

    #[inline(always)]
    fn track_err<E>(&self, _func: C, _err: &E) {}

    #[inline(always)]
    fn track_exit(&self) {}
}

impl<'s, C> TrackedSpan<C> for &'s [u8]
where
    C: Code,
{
    #[inline(always)]
    fn track_enter(&self, _func: C) {}

    #[inline(always)]
    fn track_debug(&self, _debug: String) {}

    #[inline(always)]
    fn track_info(&self, _info: &'static str) {}

    #[inline(always)]
    fn track_warn(&self, _warn: &'static str) {}

    #[inline(always)]
    fn track_ok(&self, _input: Self) {}

    #[inline(always)]
    fn track_err<E>(&self, _func: C, _err: &E) {}

    #[inline(always)]
    fn track_exit(&self) {}
}
