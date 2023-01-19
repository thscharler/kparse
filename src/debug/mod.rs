//!
//! Some utilities for debug output.
//!

pub(crate) mod error;
pub mod tracks;

use crate::{Code, Span};
use nom::bytes::complete::take_while_m_n;
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use std::ops::{RangeFrom, RangeTo};

/// Maps a width value from the formatstring to a variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Cuts off the text at 20/40/60 characters.
pub fn restrict_str<T: AsBytes + Copy>(w: DebugWidth, text: T) -> T
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match w {
        DebugWidth::Short => restrict_str_n(20, text),
        DebugWidth::Medium => restrict_str_n(40, text),
        DebugWidth::Long => restrict_str_n(60, text),
    }
}

/// Cuts off the text at max_len characters.
pub fn restrict_str_n<T: AsBytes + Copy>(max_len: usize, text: T) -> T
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match take_while_m_n::<_, _, nom::error::Error<T>>(0, max_len, |_c| true)(text) {
        Ok((_, v)) => v,
        Err(_) => text,
    }
}

/// Cuts off the text at 20/40/60 characters.
pub fn restrict<T: AsBytes + Copy, C: Code>(w: DebugWidth, span: Span<'_, T, C>) -> Span<'_, T, C>
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match w {
        DebugWidth::Short => restrict_n(20, span),
        DebugWidth::Medium => restrict_n(40, span),
        DebugWidth::Long => restrict_n(60, span),
    }
}

/// Cuts off the text at max_len characters.
pub fn restrict_n<T: AsBytes + Copy, C: Code>(
    max_len: usize,
    span: Span<'_, T, C>,
) -> Span<'_, T, C>
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match take_while_m_n::<_, _, nom::error::Error<Span<'_, T, C>>>(0, max_len, |_c| true)(span) {
        Ok((_, v)) => v,
        Err(_) => span,
    }
}
