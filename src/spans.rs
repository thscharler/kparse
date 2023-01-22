use bytecount::{naive_num_chars, num_chars};
use memchr::{memchr, memrchr};
use nom::{AsBytes, InputLength};
use nom_locate::LocatedSpan;

pub trait SpanExt<A, T, X> {
    fn span_union(&self, first: &A, second: &A) -> Self;
}

impl<'s, 'a, X> SpanExt<LocatedSpan<&'a str, X>, &'s str, X> for LocatedSpan<&'s str, X>
where
    X: Copy,
{
    fn span_union(
        &self,
        first: &LocatedSpan<&'a str, X>,
        second: &LocatedSpan<&'a str, X>,
    ) -> LocatedSpan<&'s str, X> {
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

        unsafe {
            LocatedSpan::new_from_raw_offset(
                offset_0 + offset,
                line,
                &self.fragment()[offset..offset + len],
                extra,
            )
        }
    }
}

impl<'s, 'a, X> SpanExt<LocatedSpan<&'a [u8], X>, &'s [u8], X> for LocatedSpan<&'s [u8], X>
where
    X: Copy,
{
    fn span_union(
        &self,
        first: &LocatedSpan<&'a [u8], X>,
        second: &LocatedSpan<&'a [u8], X>,
    ) -> LocatedSpan<&'s [u8], X> {
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

        unsafe {
            LocatedSpan::new_from_raw_offset(
                offset_0 + offset,
                line,
                &self.fragment()[offset..offset + len],
                extra,
            )
        }
    }
}

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

    pub fn ascii_column<Y>(&self, fragment: &LocatedSpan<&str, Y>, sep: u8) -> usize {
        let prefix = Self::frame_prefix(&self.buf, fragment, sep);
        prefix.len()
    }

    pub fn utf8_column<Y>(&self, fragment: &LocatedSpan<&str, Y>, sep: u8) -> usize {
        let prefix = Self::frame_prefix(&self.buf, fragment, sep);
        num_chars(prefix.as_bytes())
    }

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
        // assert!(sep <= 127);
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

    /// First full line for the fragment.
    fn start_frame<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> LocatedSpan<&'s str, X> {
        let offset = fragment.location_offset();
        assert!(offset <= complete.len());

        let self_bytes = complete.as_bytes();

        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(v) => v + 1,
        };
        let end = match memchr(sep, &self_bytes[offset..]) {
            None => complete.len(),
            Some(v) => offset + v,
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

    /// Last full line for the fragment.
    ///
    /// # Safety The fragment really has to be a fragment of buf.
    fn end_frame<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> LocatedSpan<&'s str, X> {
        let offset = fragment.location_offset() + fragment.len();
        let lines = Self::count(fragment.fragment(), sep);
        assert!(offset <= complete.len());

        let self_bytes = complete.as_bytes();

        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(v) => v + 1,
        };
        let end = match memchr(sep, &self_bytes[offset..]) {
            None => 0,
            Some(v) => v + 1,
        };

        unsafe {
            LocatedSpan::new_from_raw_offset(
                start,
                fragment.location_line() + lines,
                &complete[start..end],
                complete.extra,
            )
        }
    }

    fn complete_fragment<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> LocatedSpan<&'s str, X> {
        let offset = fragment.location_offset();
        let len = fragment.len();
        assert!(offset + len <= complete.len());

        let self_bytes = complete.as_bytes();

        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };
        let end = match memchr(sep, &self_bytes[offset + len..]) {
            None => complete.len(),
            Some(o) => offset + len + o,
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

    fn next_fragment<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> (LocatedSpan<&'s str, X>, Option<LocatedSpan<&'s str, X>>) {
        let offset = fragment.location_offset();
        let len = fragment.len();
        assert!(offset + len <= complete.len());
        let lines = Self::count(fragment.fragment(), sep);

        let self_bytes = complete.as_bytes();

        let start_0 = offset + len;
        let (truncate_start, lines, start) = if start_0 == complete.len() {
            (true, lines, start_0)
        } else if self_bytes[start_0] == sep {
            // skip sep
            (false, lines + 1, start_0 + 1)
        } else {
            (false, lines, start_0)
        };

        let end = match memchr(sep, &self_bytes[start..]) {
            None => complete.len(),
            Some(o) => start + o,
        };

        let span = unsafe {
            LocatedSpan::new_from_raw_offset(
                start,
                fragment.location_line() + lines,
                &complete[start..end],
                complete.extra,
            )
        };

        (span, if truncate_start { None } else { Some(span) })
    }

    fn prev_fragment<'a, Y>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, Y>,
        sep: u8,
    ) -> (LocatedSpan<&'s str, X>, Option<LocatedSpan<&'s str, X>>) {
        let offset = fragment.location_offset();
        assert!(offset <= complete.len());

        let offset = if offset > complete.len() {
            complete.len()
        } else {
            offset
        };

        let self_bytes = complete.as_bytes();

        let end_0 = offset;
        let (trunc_end, lines, end) = if end_0 == 0 {
            (true, 0, end_0)
        } else if self_bytes[end_0 - 1] == sep {
            // skip sep
            (false, 1, end_0 - 1)
        } else {
            (false, 0, end_0)
        };

        let start = match memrchr(sep, &self_bytes[..end]) {
            None => 0,
            Some(n) => n + 1,
        };

        let span = unsafe {
            LocatedSpan::new_from_raw_offset(
                start,
                fragment.location_line() - lines,
                &complete[start..end],
                complete.extra,
            )
        };

        (span, if trunc_end { None } else { Some(span) })
    }

    fn count(fragment: &str, sep: u8) -> u32 {
        let mut count = 0;

        let mut start = 0;
        let bytes = fragment.as_bytes();
        loop {
            match memchr(sep, &bytes[start..]) {
                None => break,
                Some(o) => {
                    count += 1;
                    start = o + 1;
                }
            }
        }

        count
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
