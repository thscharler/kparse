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

    pub fn ascii_column(self, fragment: LocatedSpan<&str, X>, sep: u8) -> usize {
        let prefix = self.frame_prefix(fragment, sep);
        prefix.len()
    }

    pub fn utf8_column(self, fragment: LocatedSpan<&str, X>, sep: u8) -> usize {
        let prefix = self.frame_prefix(fragment, sep);
        num_chars(prefix.as_bytes())
    }

    pub fn naive_utf8_column(self, fragment: LocatedSpan<&str, X>, sep: u8) -> usize {
        let prefix = self.frame_prefix(fragment, sep);
        naive_num_chars(prefix.as_bytes())
    }

    /// First full line for the fragment.
    pub fn start<'a>(&self, fragment: &LocatedSpan<&'a str, X>) -> LocatedSpan<&'s str, X> {
        Self::start_frame(self, fragment, self.sep)
    }

    /// Last full line for the fragment.
    pub fn end<'a>(&self, fragment: &LocatedSpan<&'a str, X>) -> LocatedSpan<&'s str, X> {
        Self::end_frame(self, fragment, self.sep)
    }

    /// Expand the fragment to cover full lines and return an Iterator for the lines.
    pub fn current(&self, fragment: &LocatedSpan<&'s str, X>) -> Self::Iter {
        let current = self.complete_fragment(fragment, self.sep);

        SpanIter {
            delim: self.sep,
            buf: current,
            fragment: None,
        }
    }

    /// Iterator for all lines of the buffer.
    pub fn iter(&self) -> Self::Iter {
        SpanIter {
            delim: self.sep,
            buf: self.buf,
            fragment: None,
        }
    }

    /// Iterator over the lines following the last line of the fragment.
    pub fn forward_from(&self, fragment: &LocatedSpan<&'s str, X>) -> Self::FwdIter {
        let current = Self::end_frame(self, fragment, self.sep);
        FSpanIter {
            delim: self.sep,
            buf: self.buf,
            fragment: current,
        }
    }

    /// Iterator over the lines preceeding the first line of the fragment.
    /// In descending order.
    pub fn backward_from(&self, fragment: &LocatedSpan<&'s str, X>) -> Self::RevIter {
        let current = Self::start_frame(self, fragment, self.sep);
        RSpanIter {
            delim: self.sep,
            buf: self.buf,
            fragment: current,
        }
    }

    /// Returns the part of the frame from the last separator up to the start of the
    /// fragment.
    fn frame_prefix<'a>(
        complete: LocatedSpan<&'s str, X>,
        fragment: LocatedSpan<&'a str, X>,
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

    /// First full line for the fragment.
    fn start_frame<'a>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, X>,
        sep: u8,
    ) -> LocatedSpan<&'s str, X> {
        // assert!(sep <= 127);
        let offset = fragment.location_offset();
        assert!(offset <= complete.buf.len());

        let self_bytes = complete.buf.as_bytes();

        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(v) => v + 1,
        };
        let end = match memchr(sep, &self_bytes[offset..]) {
            None => complete.buf.len(),
            Some(v) => offset + v,
        };

        unsafe {
            LocatedSpan::new_from_raw_offset(
                start,
                fragment.location_line(),
                &complete.buf[start..end],
                complete.buf.extra,
            )
        }
    }

    /// Last full line for the fragment.
    ///
    /// # Safety The fragment really has to be a fragment of buf.
    fn end_frame<'a>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, X>,
        sep: u8,
    ) -> Self::Frame {
        // assert!(sep <= 127);

        let offset = fragment.location_offset() + fragment.len();
        let lines = Self::count(fragment.fragment(), sep);
        assert!(offset <= complete.buf.len());

        let self_bytes = complete.buf.as_bytes();

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
                &complete.buf[start..end],
                complete.buf.extra,
            )
        }
    }

    fn complete_fragment<'a>(
        complete: &LocatedSpan<&'s str, X>,
        fragment: &LocatedSpan<&'a str, X>,
        sep: u8,
    ) -> LocatedSpan<&'s str, X> {
        // assert!(sep <= 127);

        let offset = fragment.location_offset();
        let len = fragment.len();
        assert!(offset + len <= complete.buf.len());

        let self_bytes = complete.buf.as_bytes();

        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };
        let end = match memchr(sep, &self_bytes[offset + len..]) {
            None => complete.buf.len(),
            Some(o) => offset + len + o,
        };

        unsafe {
            LocatedSpan::new_from_raw_offset(
                start,
                fragment.location_line(),
                &complete.buf[start..end],
                complete.buf.extra,
            )
        }
    }

    fn next_fragment<'a>(
        complete: LocatedSpan<&'s str, X>,
        fragment: LocatedSpan<&'a str, X>,
        sep: u8,
    ) -> (LocatedSpan<&'s str, X>, bool) {
        // assert!(sep <= 127);
        let offset = fragment.location_offset();
        let len = fragment.len();
        let lines = Self::count(fragment.fragment(), sep);
        assert!(offset + len <= complete.buf.len());

        let self_bytes = complete.buf.as_bytes();

        let start_0 = offset + len;
        let (truncate_start, lines, start) = if start_0 == complete.buf.len() {
            (true, lines, start_0)
        } else if self_bytes[start_0] == sep {
            // skip sep
            (false, lines + 1, start_0 + 1)
        } else {
            (false, lines, start_0)
        };

        let end = match memchr(sep, &self_bytes[start..]) {
            None => complete.buf.len(),
            Some(o) => start + o,
        };

        unsafe {
            (
                LocatedSpan::new_from_raw_offset(
                    start,
                    fragment.location_line() + lines,
                    &complete.buf[start..end],
                    complete.buf.extra,
                ),
                !truncate_start,
            )
        }
    }

    fn prev_fragment<'a>(
        complete: LocatedSpan<&'s str, X>,
        fragment: LocatedSpan<&'a str, X>,
        sep: u8,
    ) -> (LocatedSpan<&'s str, X>, bool) {
        // assert!(sep <= 127);

        let offset = fragment.location_offset();

        let self_bytes = complete.buf.as_bytes();

        let end_0 = offset;
        let (trunc_end, lines, end) = if end_0 == 0 {
            (true, 0, end_0)
        } else if self_bytes[end_0 - 1] == sep {
            // skip sep
            (false, -1, end_0 - 1)
        } else {
            (false, 0, end_0)
        };

        let start = match memrchr(sep, &self_bytes[..end]) {
            None => 0,
            Some(n) => n + 1,
        };

        unsafe {
            (
                LocatedSpan::new_from_raw_offset(
                    start,
                    fragment.location_line() + lines,
                    &complete.buf[start..end],
                    complete.buf.extra,
                ),
                !trunc_end,
            )
        }
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
