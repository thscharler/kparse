//!
//! Additions to LocatedSpan, str and \[u8\]
//!

use bytecount::{count, naive_num_chars, num_chars};
use memchr::{memchr, memrchr};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
use std::ops::{Range, RangeFrom, RangeTo};

/// Extension trait for Spans.
pub trait SpanExt {
    /// Return a new Span that encompasses both parameters.
    ///
    /// # Safety
    /// Uses the offset from both spans and corrects order and bounds. So the result might
    /// be nonsensical but safe.
    fn span_union<'a>(&self, first: &'a Self, second: &'a Self) -> Self;
}

impl<'s> SpanExt for &'s str {
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

impl<'s> SpanExt for &'s [u8] {
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

impl<T, X> SpanExt for LocatedSpan<T, X>
where
    T: AsBytes,
    X: Copy,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<Range<usize>>
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
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
                first.extra,
            )
        } else {
            (
                offset_2,
                second.location_line(),
                offset_1 - offset_2 + first.input_len(),
                second.extra,
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

/// Operations on 'lines' of text.
///
/// Can use any other ASCII value besides \n.
#[derive(Debug)]
pub struct SpanLines<'s, X> {
    sep: u8,
    buf: LocatedSpan<&'s str, X>,
}

impl<'s, X: Copy + 's> SpanLines<'s, X> {
    /// Create a new SpanLines buffer.
    pub fn new(buf: LocatedSpan<&'s str, X>) -> Self {
        Self { sep: b'\n', buf }
    }

    /// Create a new SpanLines buffer.
    ///
    /// # Panics
    /// The separator must be an ASCII value (<128).
    pub fn with_separator(buf: LocatedSpan<&'s str, X>, sep: u8) -> Self {
        assert!(sep < 128);
        Self { sep, buf }
    }

    /// Assumes ASCII text and gives a column.
    pub fn ascii_column<Y>(&self, fragment: &LocatedSpan<&str, Y>, sep: u8) -> usize {
        let prefix = Self::frame_prefix(&self.buf, fragment, sep);
        prefix.len()
    }

    /// Gives a column for UTF8 text.
    pub fn utf8_column<Y>(&self, fragment: &LocatedSpan<&str, Y>, sep: u8) -> usize {
        let prefix = Self::frame_prefix(&self.buf, fragment, sep);
        num_chars(prefix.as_bytes())
    }

    /// Gives a column for UTF8 text.
    pub fn naive_utf8_column<Y>(&self, fragment: &LocatedSpan<&str, Y>, sep: u8) -> usize {
        let prefix = Self::frame_prefix(&self.buf, fragment, sep);
        naive_num_chars(prefix.as_bytes())
    }

    /// Return n lines before and after the fragment, and place the lines of the fragment
    /// between them.
    pub fn get_lines_around<'a, Y>(
        &self,
        fragment: &LocatedSpan<&'a str, Y>,
        n: usize,
    ) -> Vec<LocatedSpan<&'s str, X>> {
        let mut buf: Vec<_> = self.backward_from(fragment).take(n).collect();
        buf.reverse();
        buf.extend(self.current(fragment));
        buf.extend(self.forward_from(fragment).take(n));

        buf
    }

    /// First full line for the fragment.
    pub fn start<'a, Y>(&self, fragment: &LocatedSpan<&'a str, Y>) -> LocatedSpan<&'s str, X> {
        Self::start_frame(&self.buf, fragment, self.sep)
    }

    /// Last full line for the fragment.
    pub fn end<'a, Y>(&self, fragment: &LocatedSpan<&'a str, Y>) -> LocatedSpan<&'s str, X> {
        Self::end_frame(&self.buf, fragment, self.sep)
    }

    /// Expand the fragment to cover full lines and return an Iterator for the lines.
    pub fn current<'a, Y>(&self, fragment: &LocatedSpan<&'a str, Y>) -> SpanIter<'s, X> {
        let current = Self::complete_fragment(&self.buf, fragment, self.sep);

        SpanIter {
            sep: self.sep,
            buf: current,
            fragment: Self::empty_frame(&self.buf, &current),
        }
    }

    /// Iterator for all lines of the buffer.
    pub fn iter(&self) -> SpanIter<'s, X> {
        SpanIter {
            sep: self.sep,
            buf: self.buf,
            fragment: Self::empty_frame(&self.buf, &self.buf),
        }
    }

    /// Iterator over the lines following the last line of the fragment.
    pub fn forward_from<'a, Y>(&self, fragment: &LocatedSpan<&'a str, Y>) -> SpanIter<'s, X> {
        let current = Self::end_frame(&self.buf, fragment, self.sep);
        SpanIter {
            sep: self.sep,
            buf: self.buf,
            fragment: current,
        }
    }

    /// Iterator over the lines preceeding the first line of the fragment.
    /// In descending order.
    pub fn backward_from<'a, Y>(&self, fragment: &LocatedSpan<&'a str, Y>) -> RSpanIter<'s, X> {
        let current = Self::start_frame(&self.buf, fragment, self.sep);
        RSpanIter {
            sep: self.sep,
            buf: self.buf,
            fragment: current,
        }
    }

    /// Returns the part of the frame from the last separator up to the start of the
    /// fragment.
    fn frame_prefix<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> &'s str {
        let offset = fragment.location_offset();
        assert!(offset <= complete.len());

        let self_bytes = complete.as_bytes();

        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };

        &complete[start..offset]
    }

    /// Empty span at the beginning of the fragment.
    fn empty_frame<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
    ) -> LocatedSpan<&'s str, X> {
        let offset = fragment.location_offset();
        assert!(offset <= complete.len());

        unsafe {
            LocatedSpan::new_from_raw_offset(
                offset,
                fragment.location_line(),
                &complete[offset..offset],
                complete.extra,
            )
        }
    }

    /// Return the first full line for the fragment.
    fn start_frame<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> LocatedSpan<&'s str, X> {
        let offset = fragment.location_offset();

        // trim the offset to our bounds.
        assert!(offset <= complete.len());

        // no skip_lines, already correct.

        let self_bytes = complete.as_bytes();
        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(v) => v + 1,
        };
        let end = match memchr(sep, &self_bytes[offset..]) {
            None => complete.len(),
            Some(v) => offset + v + 1,
        };

        unsafe {
            LocatedSpan::new_from_raw_offset(
                start,
                fragment.location_line(),
                &complete[start..end],
                complete.extra,
            )
        }
    }

    /// Returns the last full frame of the fragment.
    fn end_frame<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> LocatedSpan<&'s str, X> {
        let offset = fragment.location_offset() + fragment.len();

        // trim the offset to our bounds.
        assert!(offset <= complete.len());

        // correcting lines.
        let skip_lines = count(fragment.as_bytes(), sep);

        let self_bytes = complete.as_bytes();
        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(v) => v + 1,
        };
        let end = match memchr(sep, &self_bytes[offset..]) {
            None => complete.len(),
            Some(v) => offset + v + 1,
        };

        unsafe {
            LocatedSpan::new_from_raw_offset(
                start,
                fragment.location_line() + skip_lines as u32,
                &complete[start..end],
                complete.extra,
            )
        }
    }

    /// Completes the fragment to a full frame.
    fn complete_fragment<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> LocatedSpan<&'s str, X> {
        let offset = fragment.location_offset();
        let len = fragment.len();

        // trim start and end to our bounds.
        assert!(offset <= complete.len());
        assert!(offset + len <= complete.len());
        let (start, end) = (offset, offset + len);

        // fill up front and back
        let self_bytes = complete.as_bytes();
        // println!("{:?}  {:?}", &self_bytes[..start], &self_bytes[end..]);
        let start = match memrchr(sep, &self_bytes[..start]) {
            None => 0,
            Some(o) => o + 1,
        };
        let end = match memchr(sep, &self_bytes[end..]) {
            None => complete.len(),
            Some(o) => end + o + 1,
        };

        unsafe {
            LocatedSpan::new_from_raw_offset(
                start,
                fragment.location_line(),
                &complete[start..end],
                complete.extra,
            )
        }
    }

    /// Return the following frame..
    ///
    /// If the fragment doesn't end with a separator, the result is the rest up to the
    /// following separator.
    ///
    /// The separator is included at the end of the frame.
    ///
    /// The line-count is corrected.
    fn next_fragment<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> (LocatedSpan<&'s str, X>, Option<LocatedSpan<&'s str, X>>) {
        let offset = fragment.location_offset();
        let len = fragment.len();

        // trim start to our bounds.
        assert!(offset + len <= complete.len());
        let start = offset + len;

        let is_terminal = start == complete.len();

        // real linecount
        let skip_lines = count(fragment.as_bytes(), sep);

        let self_bytes = complete.as_bytes();
        let end = match memchr(sep, &self_bytes[start..]) {
            None => complete.len(),
            Some(o) => start + o + 1,
        };

        let span = unsafe {
            LocatedSpan::new_from_raw_offset(
                start,
                fragment.location_line() + skip_lines as u32,
                &complete[start..end],
                complete.extra,
            )
        };

        (span, if is_terminal { None } else { Some(span) })
    }

    /// Return the preceding frame.
    ///
    /// If the byte immediately preceding the start of the fragment is not the separator,
    /// just a truncated fragment is returned.
    ///
    /// The separator is included at the end of a frame.
    fn prev_fragment<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> (LocatedSpan<&'s str, X>, Option<LocatedSpan<&'s str, X>>) {
        let offset = fragment.location_offset();

        // assert our bounds.
        assert!(offset <= complete.len());
        let end = offset;

        // At the beginning?
        let is_terminal = end == 0;

        // immediately preceeding separator.
        let self_bytes = complete.as_bytes();
        #[allow(clippy::bool_to_int_with_if)]
        let skip_lines = if !is_terminal && self_bytes[end - 1] == sep {
            1
        } else {
            0
        };

        // find separator
        let start = match memrchr(sep, &self_bytes[..end - skip_lines]) {
            None => 0,
            Some(n) => n + 1,
        };

        let span = unsafe {
            LocatedSpan::new_from_raw_offset(
                start,
                fragment.location_line() - skip_lines as u32,
                &complete[start..end],
                complete.extra,
            )
        };

        (span, if is_terminal { None } else { Some(span) })
    }
}

/// Iterates all lines.
pub struct SpanIter<'s, X> {
    sep: u8,
    buf: LocatedSpan<&'s str, X>,
    fragment: LocatedSpan<&'s str, X>,
}

impl<'s, X: Copy + 's> Iterator for SpanIter<'s, X> {
    type Item = LocatedSpan<&'s str, X>;

    fn next(&mut self) -> Option<Self::Item> {
        let (next, result) = SpanLines::next_fragment(&self.buf, &self.fragment, self.sep);
        self.fragment = next;
        result
    }
}

/// Backward iterator.
pub struct RSpanIter<'s, X> {
    sep: u8,
    buf: LocatedSpan<&'s str, X>,
    fragment: LocatedSpan<&'s str, X>,
}

impl<'s, X: Copy + 's> Iterator for RSpanIter<'s, X> {
    type Item = LocatedSpan<&'s str, X>;

    fn next(&mut self) -> Option<Self::Item> {
        let (next, result) = SpanLines::prev_fragment(&self.buf, &self.fragment, self.sep);
        self.fragment = next;
        result
    }
}

/// Operations on 'lines' of text.
///
/// Can use any other ASCII value besides \n.
#[derive(Debug)]
pub struct SpanBytes<'s> {
    sep: u8,
    buf: &'s [u8],
}

impl<'s> SpanBytes<'s> {
    /// Create a new SpanLines buffer.
    pub fn new(buf: &'s [u8]) -> Self {
        Self { sep: b'\n', buf }
    }

    /// Create a new SpanLines buffer.
    ///
    /// # Panics
    /// The separator must be an ASCII value (<128).
    pub fn with_separator(buf: &'s [u8], sep: u8) -> Self {
        assert!(sep < 128);
        Self { sep, buf }
    }

    /// Assumes ASCII text and gives a column.
    pub fn ascii_column<Y>(&self, fragment: &[u8], sep: u8) -> usize {
        let prefix = Self::frame_prefix(self.buf, fragment, sep);
        prefix.len()
    }

    /// Gives a column for UTF8 text.
    pub fn utf8_column<Y>(&self, fragment: &[u8], sep: u8) -> usize {
        let prefix = Self::frame_prefix(self.buf, fragment, sep);
        num_chars(prefix.as_bytes())
    }

    /// Gives a column for UTF8 text.
    pub fn naive_utf8_column<Y>(&self, fragment: &[u8], sep: u8) -> usize {
        let prefix = Self::frame_prefix(self.buf, fragment, sep);
        naive_num_chars(prefix.as_bytes())
    }

    /// Return n lines before and after the fragment, and place the lines of the fragment
    /// between them.
    pub fn get_lines_around<'a>(&self, fragment: &'a [u8], n: usize) -> Vec<&'s [u8]> {
        let mut buf: Vec<_> = self.backward_from(fragment).take(n).collect();
        buf.reverse();
        buf.extend(self.current(fragment));
        buf.extend(self.forward_from(fragment).take(n));

        buf
    }

    /// First full line for the fragment.
    pub fn start<'a>(&self, fragment: &'a [u8]) -> &'s [u8] {
        Self::start_frame(self.buf, fragment, self.sep)
    }

    /// Last full line for the fragment.
    pub fn end<'a>(&self, fragment: &'a [u8]) -> &'s [u8] {
        Self::end_frame(self.buf, fragment, self.sep)
    }

    /// Expand the fragment to cover full lines and return an Iterator for the lines.
    pub fn current<'a>(&self, fragment: &'a [u8]) -> BytesIter<'s> {
        let current = Self::complete_fragment(self.buf, fragment, self.sep);

        BytesIter {
            sep: self.sep,
            buf: current,
            fragment: Self::empty_frame(self.buf, current),
        }
    }

    /// Iterator for all lines of the buffer.
    pub fn iter(&self) -> BytesIter<'s> {
        BytesIter {
            sep: self.sep,
            buf: self.buf,
            fragment: Self::empty_frame(self.buf, self.buf),
        }
    }

    /// Iterator over the lines following the last line of the fragment.
    pub fn forward_from<'a>(&self, fragment: &'a [u8]) -> BytesIter<'s> {
        let current = Self::end_frame(self.buf, fragment, self.sep);
        BytesIter {
            sep: self.sep,
            buf: self.buf,
            fragment: current,
        }
    }

    /// Iterator over the lines preceeding the first line of the fragment.
    /// In descending order.
    pub fn backward_from<'a>(&self, fragment: &'a [u8]) -> RBytesIter<'s> {
        let current = Self::start_frame(self.buf, fragment, self.sep);
        RBytesIter {
            sep: self.sep,
            buf: self.buf,
            fragment: current,
        }
    }

    /// Returns the part of the frame from the last separator up to the start of the
    /// fragment.
    fn frame_prefix<'a>(complete: &'s [u8], fragment: &'a [u8], sep: u8) -> &'s [u8] {
        let offset = Self::offset_from(complete, fragment);
        assert!(offset <= complete.len());

        let self_bytes = complete.as_bytes();

        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };

        &complete[start..offset]
    }

    /// Empty span at the beginning of the fragment.
    fn empty_frame<'a>(complete: &'s [u8], fragment: &'a [u8]) -> &'s [u8] {
        let offset = Self::offset_from(complete, fragment);
        assert!(offset <= complete.len());

        &complete[offset..offset]
    }

    /// Return the first full line for the fragment.
    fn start_frame<'a>(complete: &'s [u8], fragment: &'a [u8], sep: u8) -> &'s [u8] {
        let offset = Self::offset_from(complete, fragment);

        // trim the offset to our bounds.
        assert!(offset <= complete.len());

        // no skip_lines, already correct.

        let self_bytes = complete.as_bytes();
        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(v) => v + 1,
        };
        let end = match memchr(sep, &self_bytes[offset..]) {
            None => complete.len(),
            Some(v) => offset + v + 1,
        };

        &complete[start..end]
    }

    /// Returns the last full frame of the fragment.
    fn end_frame<'a>(complete: &'s [u8], fragment: &'a [u8], sep: u8) -> &'s [u8] {
        let offset = Self::offset_from(complete, fragment) + fragment.len();

        // trim the offset to our bounds.
        assert!(offset <= complete.len());

        let self_bytes = complete.as_bytes();
        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(v) => v + 1,
        };
        let end = match memchr(sep, &self_bytes[offset..]) {
            None => complete.len(),
            Some(v) => offset + v + 1,
        };

        &complete[start..end]
    }

    /// Completes the fragment to a full frame.
    fn complete_fragment<'a>(complete: &'s [u8], fragment: &'a [u8], sep: u8) -> &'s [u8] {
        let offset = Self::offset_from(complete, fragment);
        let len = fragment.len();

        // trim start and end to our bounds.
        assert!(offset <= complete.len());
        assert!(offset + len <= complete.len());
        let (start, end) = (offset, offset + len);

        // fill up front and back
        let self_bytes = complete.as_bytes();
        // println!("{:?}  {:?}", &self_bytes[..start], &self_bytes[end..]);
        let start = match memrchr(sep, &self_bytes[..start]) {
            None => 0,
            Some(o) => o + 1,
        };
        let end = match memchr(sep, &self_bytes[end..]) {
            None => complete.len(),
            Some(o) => end + o + 1,
        };

        &complete[start..end]
    }

    /// Return the following frame..
    ///
    /// If the fragment doesn't end with a separator, the result is the rest up to the
    /// following separator.
    ///
    /// The separator is included at the end of the frame.
    ///
    /// The line-count is corrected.
    fn next_fragment<'a>(
        complete: &'s [u8],
        fragment: &'a [u8],
        sep: u8,
    ) -> (&'s [u8], Option<&'s [u8]>) {
        let offset = Self::offset_from(complete, fragment);
        let len = fragment.len();

        // trim start to our bounds.
        assert!(offset + len <= complete.len());
        let start = offset + len;

        let is_terminal = start == complete.len();

        let self_bytes = complete.as_bytes();
        let end = match memchr(sep, &self_bytes[start..]) {
            None => complete.len(),
            Some(o) => start + o + 1,
        };

        let span = &complete[start..end];

        (span, if is_terminal { None } else { Some(span) })
    }

    /// Return the preceding frame.
    ///
    /// If the byte immediately preceding the start of the fragment is not the separator,
    /// just a truncated fragment is returned.
    ///
    /// The separator is included at the end of a frame.
    fn prev_fragment<'a>(
        complete: &'s [u8],
        fragment: &'a [u8],
        sep: u8,
    ) -> (&'s [u8], Option<&'s [u8]>) {
        let offset = Self::offset_from(complete, fragment);

        // assert our bounds.
        assert!(offset <= complete.len());
        let end = offset;

        // At the beginning?
        let is_terminal = end == 0;

        // immediately preceeding separator.
        let self_bytes = complete.as_bytes();
        #[allow(clippy::bool_to_int_with_if)]
        let skip_lines = if !is_terminal && self_bytes[end - 1] == sep {
            1
        } else {
            0
        };

        // find separator
        let start = match memrchr(sep, &self_bytes[..end - skip_lines]) {
            None => 0,
            Some(n) => n + 1,
        };

        let span = &complete[start..end];

        (span, if is_terminal { None } else { Some(span) })
    }

    fn offset_from<'a>(complete: &'s [u8], fragment: &'a [u8]) -> usize {
        let offset = unsafe { fragment.as_ptr().offset_from(complete.as_ptr()) };
        assert!(offset >= 0);
        offset as usize
    }
}

/// Iterates all lines.
pub struct BytesIter<'s> {
    sep: u8,
    buf: &'s [u8],
    fragment: &'s [u8],
}

impl<'s> Iterator for BytesIter<'s> {
    type Item = &'s [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let (next, result) = SpanBytes::next_fragment(self.buf, self.fragment, self.sep);
        self.fragment = next;
        result
    }
}

/// Backward iterator.
pub struct RBytesIter<'s> {
    sep: u8,
    buf: &'s [u8],
    fragment: &'s [u8],
}

impl<'s> Iterator for RBytesIter<'s> {
    type Item = &'s [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let (next, result) = SpanBytes::prev_fragment(self.buf, self.fragment, self.sep);
        self.fragment = next;
        result
    }
}

#[cfg(test)]
mod tests_spanlines {
    use crate::spans::SpanLines;
    use bytecount::count;
    use nom_locate::LocatedSpan;

    const SEP: u8 = b'\n';

    fn mk_fragment<'a, X: Copy>(
        span: &LocatedSpan<&'a str, X>,
        start: usize,
        end: usize,
    ) -> LocatedSpan<&'a str, X> {
        let line = count(&span.as_bytes()[..start], SEP) + 1;
        unsafe {
            LocatedSpan::new_from_raw_offset(start, line as u32, &span[start..end], span.extra)
        }
    }

    // take the list with the sep positions and turn into line bounds.
    fn test_bounds(txt: &str, occ: &[usize]) -> Vec<[usize; 2]> {
        let mut bounds = Vec::new();

        let mut st = 0usize;
        for b in occ {
            bounds.push([st, *b + 1]);
            st = *b + 1;
        }
        bounds.push([st, txt.len()]);

        // for bb in &bounds {
        //     println!("{:?} {:?}", bb, &txt[bb[0]..bb[1]]);
        // }
        bounds
    }

    #[test]
    fn test_frame_prefix() {
        fn run(txt: &str, occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            let txt = LocatedSpan::new(txt);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let mut cb = check_bounds_complete_fragment(*txt, i, i, &bounds);
                    // always override end.
                    cb.1 = i;
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let prefix = SpanLines::frame_prefix(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{} -> {}:{} <> {}",
                    //     frag.location_offset(),
                    //     frag.location_offset() + frag.len(),
                    //     frag.fragment().escape_debug(),
                    //     prefix.escape_debug(),
                    //     cmp.location_offset(),
                    //     cmp.fragment().escape_debug()
                    // );

                    assert_eq!(prefix, *cmp);
                }
            }
        }

        run("", &[]);
        run("a", &[]);
        run("aaaa", &[]);
        run("\naaaa", &[0]);
        run("aaaa\n", &[4]);
        run("\naaaa\n", &[0, 5]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    #[test]
    fn test_start_frame() {
        fn run(txt: &str, occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            let txt = LocatedSpan::new(txt);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let cb = check_bounds_complete_fragment(*txt, i, i, &bounds);
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let next = SpanLines::start_frame(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{} -> {}:{} <> {}:{}",
                    //     frag.location_offset(),
                    //     frag.location_offset() + frag.len(),
                    //     frag.fragment().escape_debug(),
                    //     next.location_offset(),
                    //     next.fragment().escape_debug(),
                    //     cmp.location_offset(),
                    //     cmp.fragment().escape_debug()
                    // );

                    assert_eq!(next, cmp);
                }
            }
        }

        run("", &[]);
        run("a", &[]);
        run("aaaa", &[]);
        run("\naaaa", &[0]);
        run("aaaa\n", &[4]);
        run("\naaaa\n", &[0, 5]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    #[test]
    fn test_end_frame() {
        fn run(txt: &str, occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            let txt = LocatedSpan::new(txt);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let cb = check_bounds_complete_fragment(*txt, j, j, &bounds);
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let next = SpanLines::end_frame(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{} -> {}:{} <> {}:{}",
                    //     frag.location_offset(),
                    //     frag.location_offset() + frag.len(),
                    //     frag.fragment().escape_debug(),
                    //     next.location_offset(),
                    //     next.fragment().escape_debug(),
                    //     cmp.location_offset(),
                    //     cmp.fragment().escape_debug()
                    // );

                    assert_eq!(next, cmp);
                }
            }
        }

        run("", &[]);
        run("a", &[]);
        run("aaaa", &[]);
        run("\naaaa", &[0]);
        run("aaaa\n", &[4]);
        run("\naaaa\n", &[0, 5]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    fn check_bounds_complete_fragment(
        txt: &str,
        start: usize,
        end: usize,
        bounds: &Vec<[usize; 2]>,
    ) -> (usize, usize) {
        let btxt = txt.as_bytes();

        let start_0 = 'loop_val: {
            for (_idx, b) in bounds.iter().enumerate() {
                if b[0] <= start && start < b[1] {
                    break 'loop_val b[0];
                } else if b[0] <= start && start == b[1] {
                    if start > 0 && btxt[start - 1] == SEP {
                        break 'loop_val start;
                    } else {
                        break 'loop_val b[0];
                    }
                }
            }
            panic!();
        };
        let end_0 = 'loop_val: {
            for (idx, b) in bounds.iter().enumerate() {
                if b[0] <= end && end < b[1] {
                    break 'loop_val b[1];
                } else if b[0] <= end && end == b[1] {
                    if idx + 1 < bounds.len() {
                        let b1 = bounds[idx + 1];
                        break 'loop_val b1[1];
                    } else {
                        break 'loop_val end;
                    }
                }
            }
            panic!();
        };

        (start_0, end_0)
    }

    #[test]
    fn test_complete_fragment() {
        fn run(txt: &str, occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            let txt = LocatedSpan::new(txt);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let cb = check_bounds_complete_fragment(*txt, i, j, &bounds);
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let next = SpanLines::complete_fragment(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{} -> {}:{} <> {}:{}",
                    //     frag.location_offset(),
                    //     frag.location_offset() + frag.len(),
                    //     frag.fragment().escape_debug(),
                    //     next.location_offset(),
                    //     next.fragment().escape_debug(),
                    //     cmp.location_offset(),
                    //     cmp.fragment().escape_debug()
                    // );

                    assert_eq!(next, cmp);
                }
            }
        }

        run("", &[]);
        run("a", &[]);
        run("aaaa", &[]);
        run("\naaaa", &[0]);
        run("aaaa\n", &[4]);
        run("\naaaa\n", &[0, 5]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    fn check_bounds_next_fragment(pos: usize, bounds: &Vec<[usize; 2]>) -> (usize, usize) {
        for (idx, b) in bounds.iter().enumerate() {
            if b[0] <= pos && pos < b[1] {
                return (pos, b[1]);
            } else if b[0] <= pos && pos == b[1] {
                if idx + 1 < bounds.len() {
                    let b1 = bounds[idx + 1];
                    return (b1[0], b1[1]);
                } else {
                    return (pos, pos);
                }
            }
        }
        panic!();
    }

    #[test]
    fn test_next_fragment() {
        fn run(txt: &str, occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            let txt = LocatedSpan::new(txt);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let cb = check_bounds_next_fragment(j, &bounds);
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let (next, _rnext) = SpanLines::next_fragment(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{} -> {}:{} <> {}:{}",
                    //     frag.location_offset(),
                    //     frag.location_offset() + frag.len(),
                    //     frag.fragment().escape_debug(),
                    //     next.location_offset(),
                    //     next.fragment().escape_debug(),
                    //     cmp.location_offset(),
                    //     cmp.fragment().escape_debug()
                    // );

                    assert_eq!(next, cmp);
                }
            }
        }

        run("", &[]);
        run("a", &[]);
        run("aaaa", &[]);
        run("\naaaa", &[0]);
        run("aaaa\n", &[4]);
        run("\naaaa\n", &[0, 5]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    fn check_bounds_prev_fragment(
        txt: &str,
        pos: usize,
        bounds: &Vec<[usize; 2]>,
    ) -> (usize, usize) {
        let btxt = txt.as_bytes();

        for b in bounds {
            if b[0] <= pos && pos < b[1] {
                return (b[0], pos);
            } else if b[0] <= pos && pos == b[1] && b[1] > 0 && btxt[b[1] - 1] == SEP {
                return (b[0], b[1]);
            } else if b[0] <= pos && pos == b[1] {
                if pos == txt.len() {
                    return (b[0], b[1]);
                }
            }
        }
        panic!();
    }

    #[test]
    fn test_prev_fragment() {
        fn run(txt: &str, occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            let txt = LocatedSpan::new(txt);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let cb = check_bounds_prev_fragment(*txt, i, &bounds);
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let (prev, _rprev) = SpanLines::prev_fragment(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{} -> {}:{} <> {}:{}",
                    //     frag.location_offset(),
                    //     frag.fragment().escape_debug(),
                    //     prev.location_offset(),
                    //     prev.fragment().escape_debug(),
                    //     cmp.location_offset(),
                    //     cmp.fragment().escape_debug()
                    // );

                    assert_eq!(prev, cmp);
                }
            }
        }

        run("", &[]);
        run("a", &[]);
        run("aaaa", &[]);
        run("\naaaa", &[0]);
        run("aaaa\n", &[4]);
        run("\naaaa\n", &[0, 5]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run("aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run("\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    #[test]
    fn test_count() {
        fn run(txt: &str) {
            // println!("--{:?}--", txt);
            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let buf = &txt[i..j];
                    let n = count(buf.as_bytes(), SEP);

                    let mut cnt = 0;
                    for c in buf.as_bytes() {
                        if *c == b'\n' {
                            cnt += 1;
                        }
                    }

                    assert_eq!(n, cnt);
                }
            }
        }

        run("");
        run("a");
        run("aaaa");
        run("aaaa\n");
        run("\naaaa");
        run("\naaaa\n");
        run("\n");
        run("\n\n");
        run("\n\n\n");
        run("\n\n\n\n");
        run("\n\n\n\n\n");
    }
}

#[cfg(test)]
mod tests_spanbytes {
    use crate::spans::SpanBytes;
    use bytecount::count;

    const SEP: u8 = b'\n';

    fn mk_fragment(span: &[u8], start: usize, end: usize) -> &[u8] {
        &span[start..end]
    }

    // take the list with the sep positions and turn into line bounds.
    fn test_bounds(txt: &[u8], occ: &[usize]) -> Vec<[usize; 2]> {
        let mut bounds = Vec::new();

        let mut st = 0usize;
        for b in occ {
            bounds.push([st, *b + 1]);
            st = *b + 1;
        }
        bounds.push([st, txt.len()]);

        // for bb in &bounds {
        //     println!("{:?} {:?}", bb, &txt[bb[0]..bb[1]]);
        // }
        bounds
    }

    #[test]
    fn test_frame_prefix() {
        fn run(txt: &[u8], occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let mut cb = check_bounds_complete_fragment(txt, i, i, &bounds);
                    // always override end.
                    cb.1 = i;
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(txt, cb.0, cb.1);

                    let frag = mk_fragment(txt, i, j);
                    let prefix = SpanBytes::frame_prefix(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{:?} -> {}:{:?} <> {}:{:?}",
                    //     SpanBytes::offset_from(txt, frag),
                    //     SpanBytes::offset_from(txt, frag) + frag.len(),
                    //     frag,
                    //     SpanBytes::offset_from(txt, prefix),
                    //     prefix,
                    //     SpanBytes::offset_from(txt, cmp),
                    //     cmp
                    // );

                    assert_eq!(prefix, cmp);
                }
            }
        }

        run(b"", &[]);
        run(b"a", &[]);
        run(b"aaaa", &[]);
        run(b"\naaaa", &[0]);
        run(b"aaaa\n", &[4]);
        run(b"\naaaa\n", &[0, 5]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    #[test]
    fn test_start_frame() {
        fn run(txt: &[u8], occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let cb = check_bounds_complete_fragment(txt, i, i, &bounds);
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let next = SpanBytes::start_frame(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{:?} -> {}:{:?} <> {}:{:?}",
                    //     SpanBytes::offset_from(txt, frag),
                    //     SpanBytes::offset_from(txt, frag) + frag.len(),
                    //     frag,
                    //     SpanBytes::offset_from(txt, next),
                    //     next,
                    //     SpanBytes::offset_from(txt, cmp),
                    //     cmp
                    // );

                    assert_eq!(next, cmp);
                }
            }
        }

        run(b"", &[]);
        run(b"a", &[]);
        run(b"aaaa", &[]);
        run(b"\naaaa", &[0]);
        run(b"aaaa\n", &[4]);
        run(b"\naaaa\n", &[0, 5]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    #[test]
    fn test_end_frame() {
        fn run(txt: &[u8], occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let cb = check_bounds_complete_fragment(txt, j, j, &bounds);
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let next = SpanBytes::end_frame(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{:?} -> {}:{:?} <> {}:{:?}",
                    //     SpanBytes::offset_from(txt, frag),
                    //     SpanBytes::offset_from(txt, frag) + frag.len(),
                    //     frag,
                    //     SpanBytes::offset_from(txt, next),
                    //     next,
                    //     SpanBytes::offset_from(txt, cmp),
                    //     cmp
                    // );

                    assert_eq!(next, cmp);
                }
            }
        }

        run(b"", &[]);
        run(b"a", &[]);
        run(b"aaaa", &[]);
        run(b"\naaaa", &[0]);
        run(b"aaaa\n", &[4]);
        run(b"\naaaa\n", &[0, 5]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    fn check_bounds_complete_fragment(
        txt: &[u8],
        start: usize,
        end: usize,
        bounds: &Vec<[usize; 2]>,
    ) -> (usize, usize) {
        let btxt = txt;

        let start_0 = 'loop_val: {
            for (_idx, b) in bounds.iter().enumerate() {
                if b[0] <= start && start < b[1] {
                    break 'loop_val b[0];
                } else if b[0] <= start && start == b[1] {
                    if start > 0 && btxt[start - 1] == SEP {
                        break 'loop_val start;
                    } else {
                        break 'loop_val b[0];
                    }
                }
            }
            panic!();
        };
        let end_0 = 'loop_val: {
            for (idx, b) in bounds.iter().enumerate() {
                if b[0] <= end && end < b[1] {
                    break 'loop_val b[1];
                } else if b[0] <= end && end == b[1] {
                    if idx + 1 < bounds.len() {
                        let b1 = bounds[idx + 1];
                        break 'loop_val b1[1];
                    } else {
                        break 'loop_val end;
                    }
                }
            }
            panic!();
        };

        (start_0, end_0)
    }

    #[test]
    fn test_complete_fragment() {
        fn run(txt: &[u8], occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let cb = check_bounds_complete_fragment(txt, i, j, &bounds);
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let next = SpanBytes::complete_fragment(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{:?} -> {}:{:?} <> {}:{:?}",
                    //     SpanBytes::offset_from(txt, frag),
                    //     SpanBytes::offset_from(txt, frag) + frag.len(),
                    //     frag,
                    //     SpanBytes::offset_from(txt, next),
                    //     next,
                    //     SpanBytes::offset_from(txt, cmp),
                    //     cmp
                    // );

                    assert_eq!(next, cmp);
                }
            }
        }

        run(b"", &[]);
        run(b"a", &[]);
        run(b"aaaa", &[]);
        run(b"\naaaa", &[0]);
        run(b"aaaa\n", &[4]);
        run(b"\naaaa\n", &[0, 5]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    fn check_bounds_next_fragment(pos: usize, bounds: &Vec<[usize; 2]>) -> (usize, usize) {
        for (idx, b) in bounds.iter().enumerate() {
            if b[0] <= pos && pos < b[1] {
                return (pos, b[1]);
            } else if b[0] <= pos && pos == b[1] {
                if idx + 1 < bounds.len() {
                    let b1 = bounds[idx + 1];
                    return (b1[0], b1[1]);
                } else {
                    return (pos, pos);
                }
            }
        }
        panic!();
    }

    #[test]
    fn test_next_fragment() {
        fn run(txt: &[u8], occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let cb = check_bounds_next_fragment(j, &bounds);
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let (next, _rnext) = SpanBytes::next_fragment(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{:?} -> {}:{:?} <> {}:{:?}",
                    //     SpanBytes::offset_from(txt, frag),
                    //     SpanBytes::offset_from(txt, frag) + frag.len(),
                    //     frag,
                    //     SpanBytes::offset_from(txt, next),
                    //     next,
                    //     SpanBytes::offset_from(txt, cmp),
                    //     cmp
                    // );

                    assert_eq!(next, cmp);
                }
            }
        }

        run(b"", &[]);
        run(b"a", &[]);
        run(b"aaaa", &[]);
        run(b"\naaaa", &[0]);
        run(b"aaaa\n", &[4]);
        run(b"\naaaa\n", &[0, 5]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    fn check_bounds_prev_fragment(
        txt: &[u8],
        pos: usize,
        bounds: &Vec<[usize; 2]>,
    ) -> (usize, usize) {
        let btxt = txt;

        for b in bounds {
            if b[0] <= pos && pos < b[1] {
                return (b[0], pos);
            } else if b[0] <= pos && pos == b[1] && b[1] > 0 && btxt[b[1] - 1] == SEP {
                return (b[0], b[1]);
            } else if b[0] <= pos && pos == b[1] {
                if pos == txt.len() {
                    return (b[0], b[1]);
                }
            }
        }
        panic!();
    }

    #[test]
    fn test_prev_fragment() {
        fn run(txt: &[u8], occ: &[usize]) {
            // println!("--{:?}--", txt);
            let bounds = test_bounds(txt, occ);

            let txt = txt;

            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let cb = check_bounds_prev_fragment(txt, i, &bounds);
                    // println!("    <{}:{}> -> <{}:{}>", i, j, cb.0, cb.1);
                    let cmp = mk_fragment(&txt, cb.0, cb.1);

                    let frag = mk_fragment(&txt, i, j);
                    let (prev, _rprev) = SpanBytes::prev_fragment(&txt, &frag, SEP);

                    // println!(
                    //     "    {}:{}:{:?} -> {}:{:?} <> {}:{:?}",
                    //     SpanBytes::offset_from(txt, frag),
                    //     SpanBytes::offset_from(txt, frag) + frag.len(),
                    //     frag,
                    //     SpanBytes::offset_from(txt, prev),
                    //     prev,
                    //     SpanBytes::offset_from(txt, cmp),
                    //     cmp
                    // );

                    assert_eq!(prev, cmp);
                }
            }
        }

        run(b"", &[]);
        run(b"a", &[]);
        run(b"aaaa", &[]);
        run(b"\naaaa", &[0]);
        run(b"aaaa\n", &[4]);
        run(b"\naaaa\n", &[0, 5]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee", &[4, 9, 14, 19]);
        run(b"aaaa\nbbbb\ncccc\ndddd\neeee\n", &[4, 9, 14, 19, 24]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee", &[0, 5, 10, 15, 20]);
        run(b"\naaaa\nbbbb\ncccc\ndddd\neeee\n", &[0, 5, 10, 15, 20, 25]);
    }

    #[test]
    fn test_count() {
        fn run(txt: &[u8]) {
            // println!("--{:?}--", txt);
            for i in 0..=txt.len() {
                for j in i..=txt.len() {
                    let buf = &txt[i..j];
                    let n = count(buf, SEP);

                    let mut cnt = 0;
                    for c in buf {
                        if *c == b'\n' {
                            cnt += 1;
                        }
                    }

                    assert_eq!(n, cnt);
                }
            }
        }

        run(b"");
        run(b"a");
        run(b"aaaa");
        run(b"aaaa\n");
        run(b"\naaaa");
        run(b"\naaaa\n");
        run(b"\n");
        run(b"\n\n");
        run(b"\n\n\n");
        run(b"\n\n\n\n");
        run(b"\n\n\n\n\n");
    }
}
