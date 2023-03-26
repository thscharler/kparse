//!
//! Additions to LocatedSpan, str and \[u8\]
//!

use nom::{AsBytes, InputLength, Slice};
use nom_locate::LocatedSpan;
use std::fmt::Debug;
use std::ops::Range;

/// Extension trait for Spans.
pub trait SpanUnion {
    /// Return a new Span that encompasses both parameters.
    ///
    /// # Safety
    /// Uses the offset from both spans and corrects order and bounds. So the result might
    /// be nonsensical but safe.
    fn span_union<'a>(&self, first: &'a Self, second: &'a Self) -> Self;
}

impl<'s> SpanUnion for &'s str {
    /// Can be implemented reasonably sane for &str.
    fn span_union<'a>(&self, first: &'a Self, second: &'a Self) -> Self {
        let self_ptr = self.as_ptr();

        let offset_1 = unsafe { first.as_ptr().offset_from(self_ptr) };
        let offset_2 = unsafe { second.as_ptr().offset_from(self_ptr) };

        let offset_1 = if offset_1 >= 0 { offset_1 as usize } else { 0 };
        let offset_2 = if offset_2 >= 0 { offset_2 as usize } else { 0 };

        let (offset, len) = if offset_1 <= offset_2 {
            (offset_1, offset_2 - offset_1 + second.len())
        } else {
            (offset_2, offset_1 - offset_2 + first.len())
        };

        let offset = if offset > self.len() {
            self.len()
        } else {
            offset
        };
        let len = if offset + len > self.len() {
            self.len() - offset
        } else {
            len
        };

        &self[offset..offset + len]
    }
}

impl<'s> SpanUnion for &'s [u8] {
    /// Can be implemented reasonably sane for &\[u8\].
    fn span_union<'a>(&self, first: &'a Self, second: &'a Self) -> Self {
        let self_ptr = self.as_ptr();

        let offset_1 = unsafe { first.as_ptr().offset_from(self_ptr) };
        let offset_2 = unsafe { second.as_ptr().offset_from(self_ptr) };

        let offset_1 = if offset_1 >= 0 { offset_1 as usize } else { 0 };
        let offset_2 = if offset_2 >= 0 { offset_2 as usize } else { 0 };

        let (offset, len) = if offset_1 <= offset_2 {
            (offset_1, offset_2 - offset_1 + second.len())
        } else {
            (offset_2, offset_1 - offset_2 + first.len())
        };

        let offset = if offset > self.len() {
            self.len()
        } else {
            offset
        };
        let len = if offset + len > self.len() {
            self.len() - offset
        } else {
            len
        };

        &self[offset..offset + len]
    }
}

impl<T, X> SpanUnion for LocatedSpan<T, X>
where
    T: AsBytes + InputLength + Slice<Range<usize>>,
    X: Clone,
{
    fn span_union<'a>(
        &self,
        first: &'a LocatedSpan<T, X>,
        second: &'a LocatedSpan<T, X>,
    ) -> LocatedSpan<T, X> {
        let offset_0 = self.location_offset();

        let offset_1 = first.location_offset() - offset_0;
        let offset_2 = second.location_offset() - offset_0;

        let (offset, line, len, extra) = if offset_1 <= offset_2 {
            (
                offset_1,
                first.location_line(),
                offset_2 - offset_1 + second.input_len(),
                first.extra.clone(),
            )
        } else {
            (
                offset_2,
                second.location_line(),
                offset_1 - offset_2 + first.input_len(),
                second.extra.clone(),
            )
        };

        let offset = if offset > self.input_len() {
            self.input_len()
        } else {
            offset
        };
        let len = if offset + len > self.input_len() {
            self.input_len() - offset
        } else {
            len
        };

        let slice = self.fragment().slice(offset..offset + len);

        unsafe { LocatedSpan::new_from_raw_offset(offset_0 + offset, line, slice, extra) }
    }
}

/// Get the fragment from a span.
pub trait SpanFragment {
    /// Type of the fragment.
    type Result: ?Sized + Debug;

    /// Equivalent to LocatedSpan::fragment()
    fn fragment(&self) -> &Self::Result;
}

impl<T, X> SpanFragment for LocatedSpan<T, X>
where
    T: Clone + AsBytes + Debug,
{
    type Result = T;

    fn fragment(&self) -> &Self::Result {
        LocatedSpan::fragment(self)
    }
}

impl<'s> SpanFragment for &'s str {
    type Result = &'s str;

    fn fragment(&self) -> &Self::Result {
        self
    }
}

impl<'s> SpanFragment for &'s [u8] {
    type Result = &'s [u8];

    fn fragment(&self) -> &Self::Result {
        self
    }
}
