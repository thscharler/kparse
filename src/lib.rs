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
//! * SpanLines and SpanBytes to get the context around a span.
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
// #![warn(missing_copy_implementations)]
// #![warn(missing_debug_implementations)]
#![warn(missing_docs)]
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

use nom::{InputIter, InputLength, Offset, Parser, Slice};
use nom_locate::LocatedSpan;
use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::ops::RangeTo;
use std::str::FromStr;

pub mod combinators;
pub mod error;
pub mod examples;
pub mod spans;
pub mod test;
pub mod token_error;
pub mod tracker;

mod context;
mod debug;
mod parser_ext;

pub use crate::context::Context;
pub use crate::error::ParserError;
use crate::token_error::TokenizerError;
pub use parser_ext::*;

/// Prelude, import the traits.
pub mod prelude {
    pub use crate::error::AppendParserError;
    pub use crate::spans::{SpanFragment, SpanLocation, SpanUnion};
    pub use crate::tracker::{ResultTracking, Tracking};
    pub use crate::{KParseError, KParser};
}

/// Alias for LocatedSpan.
/// No special properties, just for completeness.
pub type ParseSpan<T, X> = LocatedSpan<T, X>;

/// ParserResult without tracking.  
/// Equivalent to [nom::IResult]<(I, O), ParserError<C, I>>
pub type ParserResult<C, I, O> = Result<(I, O), nom::Err<ParserError<C, I>>>;

/// ParserResult without tracking.  
/// Equivalent to [nom::IResult]<(I, O), TokenizerError<C, I>>
pub type TokenizerResult<C, I, O> = Result<(I, O), nom::Err<TokenizerError<C, I>>>;

/// Parser error code.
pub trait Code: Copy + Display + Debug + Eq {
    /// Default error code for nom-errors.
    const NOM_ERROR: Self;
}

/// This trait catches the essentials for an error type within this library.
///
/// It is built this way so that it can be implemented for the concrete error
/// and the nom::Err wrapped error.
/// With some restrictions for a Result containing a nom::Err wrapped error too.
///
/// The functions returning an Option return None if
/// * self is nom::Err::Incomplete
/// * self is Result::Ok
///
/// The first case is a special error path aside from parsing, and the second
/// is not an error at all.
///
pub trait KParseError<C, I> {
    /// The base error type.
    type WrappedError: Debug;

    /// Create a matching error.
    fn from(code: C, span: I) -> Self;

    /// Returns the error code if applicable.
    fn code(&self) -> Option<C>;
    /// Returns the error span if applicable.
    fn span(&self) -> Option<I>;
    /// Returns the error if applicable.
    fn err(&self) -> Option<&Self::WrappedError>;

    /// Returns all the parts if applicable.
    fn parts(&self) -> Option<(C, I, &Self::WrappedError)>;

    /// Changes the error code.
    fn with_code(self, code: C) -> Self;
}

/// This trait is used in a few places where the function wants to accept both
/// E and nom::Err<E>.
pub trait ErrWrapped {
    /// The base error type.
    type WrappedError: Debug;

    /// Converts self to a nom::Err wrapped error.
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
