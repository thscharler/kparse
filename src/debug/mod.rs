//!
//! Some utilities for debug output.
//!

pub(crate) mod error;
pub(crate) mod tracks;

use nom::{AsBytes, InputIter, InputLength, InputTake};

/// Maps a width value from the formatstring to a variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DebugWidth {
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

/// Cuts off the text at 20/40/60 characters.
pub(crate) fn restrict_ref<T: AsBytes + Clone>(w: DebugWidth, text: &T) -> T
where
    T: InputTake + InputLength + InputIter,
{
    match w {
        DebugWidth::Short => restrict_ref_n(20, text),
        DebugWidth::Medium => restrict_ref_n(40, text),
        DebugWidth::Long => restrict_ref_n(60, text),
    }
}

/// Cuts off the text at max_len characters.
pub(crate) fn restrict_ref_n<T: AsBytes + Clone>(max_len: usize, text: &T) -> T
where
    T: InputTake + InputLength + InputIter,
{
    let mut n = 0;
    let mut x = 0;
    for (idx, _) in text.iter_indices() {
        x = idx;
        n += 1;
        if n >= max_len {
            break;
        }
    }

    text.take(x)
}

/// Cuts off the text at 20/40/60 characters.
pub(crate) fn restrict<I>(w: DebugWidth, span: I) -> I
where
    I: Clone,
    I: InputTake + InputLength + InputIter,
{
    match w {
        DebugWidth::Short => restrict_n(20, span),
        DebugWidth::Medium => restrict_n(40, span),
        DebugWidth::Long => restrict_n(60, span),
    }
}

/// Cuts off the text at max_len characters.
pub(crate) fn restrict_n<I>(max_len: usize, span: I) -> I
where
    I: Clone,
    I: InputTake + InputLength + InputIter,
{
    let mut n = 0;
    let mut x = 0;
    for (idx, _) in span.iter_indices() {
        x = idx;
        n += 1;
        if n >= max_len {
            break;
        }
    }

    span.take(x)
}
