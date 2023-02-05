//!
//! Addons for a nom parser.
//!
//! * A error code trait.
//! * A richer error type ParserError.
//! * Traits to integrate external errors.
//! * A tracking system for the parser.
//! * A simple framework to test parser functions.
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
use nom::AsBytes;
use nom_locate::LocatedSpan;
use std::fmt::{Debug, Display};

pub mod combinators;
pub mod error;
pub mod spans;
pub mod test;
pub mod tracker;

mod context;
mod debug;

pub use crate::context::Context;
pub use crate::error::ParserError;

/// Prelude, import the traits.
pub mod prelude {
    pub use crate::error::AppendParserError;
    pub use crate::spans::{SpanFragment, SpanLocation, SpanUnion};
    pub use crate::tracker::{FindTracker, TrackError};
    pub use crate::{ResultWithSpan, WithCode, WithSpan};
}

/// Alias for LocatedSpan.
/// No special properties, just for completeness.
pub type ParserSpan<T, X> = LocatedSpan<T, X>;

/// ParserResult without tracking.  
/// Equivalent to [nom::IResult]<(I, O), ParserError<C, I>>
pub type ParserResult<C, I, O, Y> = Result<(I, O), nom::Err<ParserError<C, I, Y>>>;

/// Parser error code.
pub trait Code: Copy + Display + Debug + Eq {
    /// Default error code for nom-errors.
    const NOM_ERROR: Self;
}

/// Convert an external error into a ParserError and add an error code and a span.
pub trait WithSpan<C: Code, I, E> {
    /// Convert an external error into a ParserError.
    /// Usually uses nom::Err::Failure to indicate the finality of the error.
    fn with_span(self, code: C, span: I) -> nom::Err<E>;
}

/// This is used internally to work with Result instead of an error type.
pub trait ResultWithSpan<C: Code, I, R> {
    /// Convert an external error into a ParserError.
    /// Usually uses nom::Err::Failure to indicate the finality of the error.
    fn with_span(self, code: C, span: I) -> R;
}

/// Change the error code.
///
/// Could do a conversion from an external error too, but usually there is no span to work with.
/// For external errors [WithSpan] is the right thing most of the time.
///
/// There are implementations for [ParserError], [nom::Err]&lt;E&gt; and [Result]&lt;O, E&gt;.
/// And there is one for a classic nom::error::Error too.
pub trait WithCode<C: Code, R> {
    /// Translate the error code to a new one.
    fn with_code(self, code: C) -> R;
}

// -----------------------------------------------------------------------
// conversions
// -----------------------------------------------------------------------

//
// std::num::ParseIntError
//

// from the std::wilds
impl<C, I, Y> WithSpan<C, I, ParserError<C, I, Y>> for std::num::ParseIntError
where
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn with_span(self, code: C, span: I) -> nom::Err<ParserError<C, I, Y>> {
        nom::Err::Failure(ParserError::new(code, span))
    }
}

//
// std::num::ParseFloatError
//

// from the std::wilds
impl<C, I, Y> WithSpan<C, I, ParserError<C, I, Y>> for std::num::ParseFloatError
where
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn with_span(self, code: C, span: I) -> nom::Err<ParserError<C, I, Y>> {
        nom::Err::Failure(ParserError::new(code, span))
    }
}

//
// ()
//

// from the std::wilds
impl<C, I, Y> WithSpan<C, I, ParserError<C, I, Y>> for ()
where
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn with_span(self, code: C, span: I) -> nom::Err<ParserError<C, I, Y>> {
        nom::Err::Failure(ParserError::new(code, span))
    }
}

//
// nom::error::Error
//

// take everything from nom::error::Error
impl<C, I, Y> WithSpan<C, I, ParserError<C, I, Y>> for nom::error::Error<I>
where
    I: AsBytes + Copy,
    C: Code,
    Y: Copy,
{
    fn with_span(self, code: C, span: I) -> nom::Err<ParserError<C, I, Y>> {
        nom::Err::Error(ParserError::new(code, span).with_nom(self.input, self.code))
    }
}

// take everything from nom::error::Error
impl<C, I, Y> WithSpan<C, I, ParserError<C, I, Y>> for nom::Err<nom::error::Error<I>>
where
    I: AsBytes + Copy,
    C: Code,
    Y: Copy,
{
    fn with_span(self, code: C, span: I) -> nom::Err<ParserError<C, I, Y>> {
        match self {
            nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            nom::Err::Error(e) => {
                nom::Err::Error(ParserError::new(code, span).with_nom(e.input, e.code))
            }
            nom::Err::Failure(e) => {
                nom::Err::Failure(ParserError::new(code, span).with_nom(e.input, e.code))
            }
        }
    }
}

// take everything from nom::error::Error
impl<C, I, Y> WithCode<C, ParserError<C, I, Y>> for nom::error::Error<I>
where
    I: AsBytes + Copy,
    C: Code,
    Y: Copy,
{
    fn with_code(self, code: C) -> ParserError<C, I, Y> {
        ParserError::new(code, self.input).with_nom(self.input, self.code)
    }
}

// ***********************************************************************
// LAYER 1 - useful conversions
// ***********************************************************************

//
// ParserError to nom::Err<ParserError>, useful shortcut when creating
// a fresh ParserError.
//
impl<C, I, Y> From<ParserError<C, I, Y>> for nom::Err<ParserError<C, I, Y>>
where
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn from(e: ParserError<C, I, Y>) -> Self {
        nom::Err::Error(e)
    }
}

impl<C, I, Y> WithCode<C, ParserError<C, I, Y>> for ParserError<C, I, Y>
where
    I: AsBytes + Copy,
    C: Code,
    Y: Copy,
{
    fn with_code(self, code: C) -> ParserError<C, I, Y> {
        ParserError::with_code(self, code)
    }
}

// ***********************************************************************
// LAYER 2 - wrapped in a nom::Err
// ***********************************************************************

//
// nom::Err::<E>
//

// for ease of use in case of a nom::Err wrapped something.
//
// 1. just to call with_code on an existing ParserError.
// 2. to convert whatever to a ParserError and give it a code.
impl<C, I, E, Y> WithCode<C, nom::Err<ParserError<C, I, Y>>> for nom::Err<E>
where
    C: Code,
    I: AsBytes + Copy,
    E: WithCode<C, ParserError<C, I, Y>>,
    Y: Copy,
{
    fn with_code(self, code: C) -> nom::Err<ParserError<C, I, Y>> {
        match self {
            nom::Err::Incomplete(e) => nom::Err::Incomplete(e),
            nom::Err::Error(e) => {
                let p_err: ParserError<C, I, Y> = e.with_code(code);
                nom::Err::Error(p_err)
            }
            nom::Err::Failure(e) => {
                let p_err: ParserError<C, I, Y> = e.with_code(code);
                nom::Err::Failure(p_err)
            }
        }
    }
}

// info: cannot implement this:
//
// impl<C, I, E, Y> WithSpan<C, I, nom::Err<ParserError<C, I, Y>>> for nom::Err<E>
// where
//     C: Code,
//     I: AsBytes + Copy,
//     E: WithSpan<C, I, ParserError<C, I, Y>>,
//     Y: Copy,
//
// WithSpan returns a nom::Err wrapped ParserError, and self is a nom::Err too.
// There is no clear indication which of the nom::Err should be used for the result.

// ***********************************************************************
// LAYER 3 - wrapped in a Result
// ***********************************************************************

//
// Result
//

// Any result that wraps an error type that can be converted via with_span is fine.
impl<C, I, O, E, Y> ResultWithSpan<C, I, Result<O, nom::Err<ParserError<C, I, Y>>>> for Result<O, E>
where
    E: WithSpan<C, I, ParserError<C, I, Y>>,
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn with_span(self, code: C, span: I) -> Result<O, nom::Err<ParserError<C, I, Y>>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(e.with_span(code, span)),
        }
    }
}

// everything needs a new code sometimes ... continued ...
//
// 1. this is a ParserResult with a nom::Err with a ParserError.
// 2. this is a Result with a whatever which has a WithCode<ParserError>
impl<C, I, O, E, Y> WithCode<C, Result<(I, O), nom::Err<ParserError<C, I, Y>>>>
    for Result<(I, O), E>
where
    E: WithCode<C, nom::Err<ParserError<C, I, Y>>>,
    C: Code,
    I: AsBytes + Copy,
    Y: Copy,
{
    fn with_code(self, code: C) -> Result<(I, O), nom::Err<ParserError<C, I, Y>>> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let p_err: nom::Err<ParserError<C, I, Y>> = e.with_code(code);
                Err(p_err)
            }
        }
    }
}
