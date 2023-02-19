//!
//! example parser
//!

use crate::token_error::TokenizerError;
use crate::tracker::TrackSpan;
use crate::{Code, ParserError, ParserResult, TokenizerResult};
use std::fmt::{Display, Formatter};
pub use ExCode::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExCode {
    ExNomError,

    ExTagA,
    ExTagB,
    ExNumber,

    ExAthenB,
    ExAoptB,
    ExAstarB,
    ExABstar,
    ExAorB,
    ExABNum,
}

impl Display for ExCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ExNomError => "nom",
                ExTagA => "a",
                ExTagB => "b",
                ExNumber => "number",
                ExAthenB => "A B",
                ExAoptB => "A? B",
                ExAstarB => "A* B",
                ExABstar => "(A | B)*",
                ExAorB => "A | B",
                ExABNum => "A B Number",
            }
        )
    }
}

impl Code for ExCode {
    const NOM_ERROR: Self = Self::ExNomError;
}

#[cfg(debug_assertions)]
pub type ExSpan<'s> = TrackSpan<'s, ExCode, &'s str>;
#[cfg(not(debug_assertions))]
pub type ExSpan<'s> = &'s str;
pub type ExParserResult<'s, O> = ParserResult<ExCode, ExSpan<'s>, O>;
pub type ExTokenizerResult<'s, O> = TokenizerResult<ExCode, ExSpan<'s>, O>;
pub type ExParserError<'s> = ParserError<ExCode, ExSpan<'s>>;
pub type ExTokenizerError<'s> = TokenizerError<ExCode, ExSpan<'s>>;
