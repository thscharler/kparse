pub mod error;

use crate::{Code, Span};
use nom::bytes::complete::take_while_m_n;
use nom::InputIter;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DebugWidth {
    /// Debug flag, can be set with width=0.
    Short,
    /// Debug flag, can be set with width=1.
    Medium,
    /// Debug flag, can be set with width=2.
    Long,
}

impl From<Option<usize>> for DebugWidth {
    fn from(value: Option<usize>) -> Self {
        match value {
            None | Some(0) => DebugWidth::Short,
            Some(1) => DebugWidth::Medium,
            Some(2) => DebugWidth::Long,
            _ => DebugWidth::Short,
        }
    }
}

pub fn restrict<C: Code>(w: DebugWidth, span: Span<'_, C>) -> String {
    match w {
        DebugWidth::Short => restrict_n(20, span),
        DebugWidth::Medium => restrict_n(40, span),
        DebugWidth::Long => restrict_n(60, span),
    }
}

pub fn restrict_n<C: Code>(max_len: usize, span: Span<'_, C>) -> String {
    let shortened =
        match take_while_m_n::<_, _, nom::error::Error<Span<'_, C>>>(0, max_len, |_c| true)(span) {
            Ok((_rest, short)) => *short,
            Err(_) => "?error?",
        };

    if span.len() > max_len {
        shortened
            .escape_default()
            .chain("...".iter_elements())
            .collect()
    } else {
        shortened.escape_default().collect()
    }
}
