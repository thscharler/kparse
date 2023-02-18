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
use nom_locate::LocatedSpan;
use std::fmt::{Debug, Display};

pub mod combinators;
pub mod error;
pub mod spans;
pub mod test;
pub mod token_error;
pub mod tracker;

mod context;
mod debug;

pub use crate::context::Context;
pub use crate::error::ParserError;
use crate::token_error::TokenizerError;

/// Prelude, import the traits.
pub mod prelude {
    pub use crate::error::AppendParserError;
    pub use crate::spans::{SpanFragment, SpanLocation, SpanUnion};
    pub use crate::token_error::{IntoParserError, IntoParserErrorExtra};
    pub use crate::tracker::{ResultTracking, Tracking};
    pub use crate::{ErrWrapped, ParseErrorExt, WithCode};
}

/// Alias for LocatedSpan.
/// No special properties, just for completeness.
pub type ParserSpan<T, X> = LocatedSpan<T, X>;

/// ParserResult without tracking.  
/// Equivalent to [nom::IResult]<(I, O), ParserError<C, I>>
pub type ParserResult<C, I, O, Y> = Result<(I, O), nom::Err<ParserError<C, I, Y>>>;

/// ParserResult without tracking.  
/// Equivalent to [nom::IResult]<(I, O), TokenizerError<C, I>>
pub type TokenizerResult<C, I, O> = Result<(I, O), nom::Err<TokenizerError<C, I>>>;

/// Parser error code.
pub trait Code: Copy + Display + Debug + Eq {
    /// Default error code for nom-errors.
    const NOM_ERROR: Self;
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

/// Minimal information for a ParserError.
pub trait ParseErrorExt<C, I> {
    fn code(&self) -> C;
    fn span(&self) -> I;
}

/// Any type that can be wrapped in a nom::Err.
pub trait ErrWrapped {
    /// The wrapped type.
    type WrappedType;
    /// Return as wrapped.
    fn wrapped(self) -> nom::Err<Self::WrappedType>;
    /// Return a reference to the type itself.
    fn as_ref(&self) -> Option<&Self::WrappedType>;
}
