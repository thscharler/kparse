use nom::AsBytes;
use nom_locate::LocatedSpan;

/// Location within the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    /// Offset of the fragment.
    pub offset: usize,
    /// Line of the fragment.
    pub line: usize,
    /// Column of the fragment.
    pub column: usize,
}

/// Source span.
#[allow(clippy::needless_lifetimes)]
pub trait Source<I> {
    type Result;

    /// Changes the separator to use from '\n'.
    ///
    /// # Panics
    /// The separator must be an ASCII value (<128).
    fn with_separator(self, sep: u8) -> Self;
    /// Assume the content is plain ASCII and use a simplified
    /// column calculation.
    fn with_ascii(self, ascii: bool) -> Self;

    /// Returns the offset of the fragment.
    fn offset(&self, fragment: I) -> usize;
    /// Returns the line of the fragment.
    fn line(&self, fragment: I) -> usize;
    /// Returns the column of the fragment.
    fn column(&self, fragment: I) -> usize;
    /// Returns offset/line/column of the fragment.
    fn location(&self, fragment: I) -> SourceLocation;

    /// Return n lines before and after the fragment, and place the lines of the fragment
    /// between them.
    fn get_lines_around(&self, fragment: I, n: usize) -> Vec<Self::Result>;

    /// First full line for the fragment.
    fn start(&self, fragment: I) -> Self::Result;
    /// Last full line for the fragment.
    fn end(&self, fragment: I) -> Self::Result;

    /// Forward iterator.
    type SpanIter<'it>: Iterator<Item = Self::Result>
    where
        Self: 'it;

    /// Backward iterator.
    type RSpanIter<'it>: Iterator<Item = Self::Result>
    where
        Self: 'it;

    /// Expand the fragment to cover full lines and return an Iterator for the lines.
    fn current<'a>(&'a self, fragment: I) -> Self::SpanIter<'a>;
    /// Iterator for all lines of the buffer.
    fn iter<'a>(&'a self) -> Self::SpanIter<'a>;
    /// Iterator over the lines following the last line of the fragment.
    fn forward_from<'a>(&'a self, fragment: I) -> Self::SpanIter<'a>;
    /// Iterator over the lines preceding the first line of the fragment.
    /// In descending order.
    fn backward_from<'a>(&'a self, fragment: I) -> Self::RSpanIter<'a>;
}

#[derive(Debug)]
pub struct SourceBytes<'s> {
    sep: u8,
    ascii: bool,
    buf: &'s [u8],
    idx: Vec<usize>,
}

impl<'s> SourceBytes<'s> {
    /// Create a new SpanLines buffer.
    pub fn new(buf: &'s [u8]) -> Self {
        Self {
            sep: b'\n',
            ascii: false,
            buf,
            idx: raw::index_lines(buf, b'\n'),
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'s, 'i, Y> Source<LocatedSpan<&'i [u8], Y>> for SourceBytes<'s>
where
    Y: Clone + 'i,
{
    type Result = LocatedSpan<&'s [u8], ()>;

    fn with_separator(mut self, sep: u8) -> Self {
        assert!(sep < 128);
        self.sep = sep;
        self.idx = raw::index_lines(self.buf, sep);
        self
    }

    fn with_ascii(mut self, ascii: bool) -> Self {
        self.ascii = ascii;
        self
    }

    fn offset(&self, fragment: LocatedSpan<&'i [u8], Y>) -> usize {
        raw::offset_from(self.buf, fragment.as_bytes())
    }

    fn line(&self, fragment: LocatedSpan<&'i [u8], Y>) -> usize {
        raw::line_index(&self.idx, raw::offset_from(self.buf, fragment.as_bytes()))
    }

    fn column(&self, fragment: LocatedSpan<&'i [u8], Y>) -> usize {
        if self.ascii {
            raw::ascii_column(self.buf, fragment.as_bytes(), self.sep)
        } else {
            raw::utf8_column(self.buf, fragment.as_bytes(), self.sep)
        }
    }

    fn location(&self, fragment: LocatedSpan<&'i [u8], Y>) -> SourceLocation {
        SourceLocation {
            offset: raw::offset_from(self.buf, fragment.as_bytes()),
            line: raw::line_index(&self.idx, raw::offset_from(self.buf, fragment.as_bytes())),
            column: if self.ascii {
                raw::ascii_column(self.buf, fragment.as_bytes(), self.sep)
            } else {
                raw::utf8_column(self.buf, fragment.as_bytes(), self.sep)
            },
        }
    }

    fn get_lines_around(&self, fragment: LocatedSpan<&'i [u8], Y>, n: usize) -> Vec<Self::Result> {
        let mut buf: Vec<_> = self.backward_from(fragment.clone()).take(n).collect();
        buf.reverse();
        buf.extend(self.current(fragment.clone()));
        buf.extend(self.forward_from(fragment).take(n));

        buf
    }

    fn start(&self, fragment: LocatedSpan<&'i [u8], Y>) -> Self::Result {
        raw::start_frame(self.buf, fragment.as_bytes(), self.sep).as_span_bytes(&self.idx)
    }

    fn end(&self, fragment: LocatedSpan<&'i [u8], Y>) -> Self::Result {
        raw::end_frame(self.buf, fragment.as_bytes(), self.sep).as_span_bytes(&self.idx)
    }

    type SpanIter<'it> = LocatedSpanBytesIter<'it, 's>
    where Self: 'it;
    type RSpanIter<'it> = RLocatedSpanBytesIter<'it, 's>
    where Self: 'it;

    fn current<'a>(&'a self, fragment: LocatedSpan<&'i [u8], Y>) -> Self::SpanIter<'a> {
        let frag = raw::complete_fragment(self.buf, fragment.as_bytes(), self.sep);

        LocatedSpanBytesIter {
            sep: self.sep,
            buf: frag.span,
            idx: &self.idx,
            fragment: raw::empty_frame(self.buf, frag.span).span,
        }
    }

    fn iter<'a>(&'a self) -> Self::SpanIter<'a> {
        LocatedSpanBytesIter {
            sep: self.sep,
            buf: self.buf,
            idx: &self.idx,
            fragment: raw::empty_frame(self.buf, self.buf).span,
        }
    }

    fn forward_from<'a>(&'a self, fragment: LocatedSpan<&'i [u8], Y>) -> Self::SpanIter<'a> {
        let frag = raw::end_frame(self.buf, fragment.as_bytes(), self.sep);
        LocatedSpanBytesIter {
            sep: self.sep,
            buf: self.buf,
            idx: &self.idx,
            fragment: frag.span,
        }
    }

    fn backward_from<'a>(&'a self, fragment: LocatedSpan<&'i [u8], Y>) -> Self::RSpanIter<'a> {
        let frag = raw::start_frame(self.buf, fragment.as_bytes(), self.sep);
        RLocatedSpanBytesIter {
            sep: self.sep,
            buf: self.buf,
            idx: &self.idx,
            fragment: frag.span,
        }
    }
}

/// Iterates all lines.
#[doc(hidden)]
pub struct LocatedSpanBytesIter<'i, 's> {
    sep: u8,
    buf: &'s [u8],
    fragment: &'s [u8],
    idx: &'i [usize],
}

impl<'i, 's> Iterator for LocatedSpanBytesIter<'i, 's> {
    type Item = LocatedSpan<&'s [u8], ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let frag = raw::next_fragment(self.buf, self.fragment, self.sep);
        self.fragment = frag.span;
        frag.as_iter_span_bytes(self.idx)
    }
}

/// Backward iterator.
#[doc(hidden)]
pub struct RLocatedSpanBytesIter<'i, 's> {
    sep: u8,
    buf: &'s [u8],
    fragment: &'s [u8],
    idx: &'i [usize],
}

impl<'i, 's> Iterator for RLocatedSpanBytesIter<'i, 's> {
    type Item = LocatedSpan<&'s [u8], ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let frag = raw::prev_fragment(self.buf, self.fragment, self.sep);
        self.fragment = frag.span;
        frag.as_iter_span_bytes(self.idx)
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'i, 's> Source<&'i [u8]> for SourceBytes<'s> {
    type Result = &'s [u8];

    fn with_separator(mut self, sep: u8) -> Self {
        self.sep = sep;
        self.idx = raw::index_lines(self.buf, sep);
        self
    }

    fn with_ascii(mut self, ascii: bool) -> Self {
        self.ascii = ascii;
        self
    }

    fn offset(&self, fragment: &'i [u8]) -> usize {
        raw::offset_from(self.buf, fragment.as_bytes())
    }

    fn line(&self, fragment: &'i [u8]) -> usize {
        raw::line_index(&self.idx, raw::offset_from(self.buf, fragment.as_bytes()))
    }

    fn column(&self, fragment: &'i [u8]) -> usize {
        if self.ascii {
            raw::ascii_column(self.buf, fragment, self.sep)
        } else {
            raw::utf8_column(self.buf, fragment, self.sep)
        }
    }

    fn location(&self, fragment: &'i [u8]) -> SourceLocation {
        SourceLocation {
            offset: raw::offset_from(self.buf, fragment),
            line: raw::line_index(&self.idx, raw::offset_from(self.buf, fragment.as_bytes())),
            column: if self.ascii {
                raw::ascii_column(self.buf, fragment, self.sep)
            } else {
                raw::utf8_column(self.buf, fragment, self.sep)
            },
        }
    }

    fn get_lines_around(&self, fragment: &'i [u8], n: usize) -> Vec<&'s [u8]> {
        let mut buf: Vec<_> = self.backward_from(fragment).take(n).collect();
        buf.reverse();
        buf.extend(self.current(fragment));
        buf.extend(self.forward_from(fragment).take(n));

        buf
    }

    fn start(&self, fragment: &'i [u8]) -> &'s [u8] {
        raw::start_frame(self.buf, fragment, self.sep).as_bytes()
    }

    fn end(&self, fragment: &'i [u8]) -> &'s [u8] {
        raw::end_frame(self.buf, fragment, self.sep).as_bytes()
    }

    type SpanIter<'it> = BytesIter<'s>
    where Self: 'it;
    type RSpanIter<'it> = RBytesIter<'s>
    where Self: 'it;

    fn current<'a>(&'a self, fragment: &'i [u8]) -> Self::SpanIter<'a> {
        let frag = raw::complete_fragment(self.buf, fragment, self.sep);

        BytesIter {
            sep: self.sep,
            buf: frag.as_bytes(),
            fragment: raw::empty_frame(self.buf, frag.as_bytes()).as_bytes(),
        }
    }

    fn iter<'a>(&'a self) -> Self::SpanIter<'a> {
        BytesIter {
            sep: self.sep,
            buf: self.buf,
            fragment: raw::empty_frame(self.buf, self.buf).as_bytes(),
        }
    }

    fn forward_from<'a>(&'a self, fragment: &'i [u8]) -> Self::SpanIter<'a> {
        let frag = raw::end_frame(self.buf, fragment, self.sep);
        BytesIter {
            sep: self.sep,
            buf: self.buf,
            fragment: frag.as_bytes(),
        }
    }

    fn backward_from<'a>(&'a self, fragment: &'i [u8]) -> Self::RSpanIter<'a> {
        let frag = raw::start_frame(self.buf, fragment, self.sep);
        RBytesIter {
            sep: self.sep,
            buf: self.buf,
            fragment: frag.as_bytes(),
        }
    }
}

/// Iterates all lines.
#[doc(hidden)]
pub struct BytesIter<'s> {
    sep: u8,
    buf: &'s [u8],
    fragment: &'s [u8],
}

impl<'s> Iterator for BytesIter<'s> {
    type Item = &'s [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let frag = raw::next_fragment(self.buf, self.fragment, self.sep);
        self.fragment = frag.as_bytes();
        frag.as_iter_bytes()
    }
}

/// Backward iterator.
#[doc(hidden)]
pub struct RBytesIter<'s> {
    sep: u8,
    buf: &'s [u8],
    fragment: &'s [u8],
}

impl<'s> Iterator for RBytesIter<'s> {
    type Item = &'s [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let frag = raw::prev_fragment(self.buf, self.fragment, self.sep);
        self.fragment = frag.as_bytes();
        frag.as_iter_bytes()
    }
}

#[derive(Debug)]
pub struct SourceStr<'s> {
    sep: u8,
    ascii: bool,
    buf: &'s [u8],
    idx: Vec<usize>,
}

impl<'s> SourceStr<'s> {
    /// Create a new SpanLines buffer.
    pub fn new(buf: &'s str) -> Self {
        Self {
            sep: b'\n',
            ascii: false,
            buf: buf.as_bytes(),
            idx: raw::index_lines(buf.as_bytes(), b'\n'),
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'s, 'i, Y> Source<LocatedSpan<&'i str, Y>> for SourceStr<'s>
where
    Y: Clone + 'i,
{
    type Result = LocatedSpan<&'s str, ()>;

    fn with_separator(mut self, sep: u8) -> Self {
        assert!(sep < 128);
        self.sep = sep;
        self.idx = raw::index_lines(self.buf, sep);
        self
    }

    fn with_ascii(mut self, ascii: bool) -> Self {
        self.ascii = ascii;
        self
    }

    fn offset(&self, fragment: LocatedSpan<&'i str, Y>) -> usize {
        raw::offset_from(self.buf, fragment.as_bytes())
    }

    fn line(&self, fragment: LocatedSpan<&'i str, Y>) -> usize {
        raw::line_index(&self.idx, raw::offset_from(self.buf, fragment.as_bytes()))
    }

    fn column(&self, fragment: LocatedSpan<&'i str, Y>) -> usize {
        if self.ascii {
            raw::ascii_column(self.buf, fragment.as_bytes(), self.sep)
        } else {
            raw::utf8_column(self.buf, fragment.as_bytes(), self.sep)
        }
    }

    fn location(&self, fragment: LocatedSpan<&'i str, Y>) -> SourceLocation {
        SourceLocation {
            offset: raw::offset_from(self.buf, fragment.as_bytes()),
            line: raw::line_index(&self.idx, raw::offset_from(self.buf, fragment.as_bytes())),
            column: if self.ascii {
                raw::ascii_column(self.buf, fragment.as_bytes(), self.sep)
            } else {
                raw::utf8_column(self.buf, fragment.as_bytes(), self.sep)
            },
        }
    }

    fn get_lines_around(
        &self,
        fragment: LocatedSpan<&'i str, Y>,
        n: usize,
    ) -> Vec<LocatedSpan<&'s str, ()>> {
        let mut buf: Vec<_> = self.backward_from(fragment.clone()).take(n).collect();
        buf.reverse();
        buf.extend(self.current(fragment.clone()));
        buf.extend(self.forward_from(fragment).take(n));

        buf
    }

    fn start(&self, fragment: LocatedSpan<&'i str, Y>) -> LocatedSpan<&'s str, ()> {
        raw::start_frame(self.buf, fragment.as_bytes(), self.sep).as_span_str(&self.idx)
    }

    fn end(&self, fragment: LocatedSpan<&'i str, Y>) -> LocatedSpan<&'s str, ()> {
        raw::end_frame(self.buf, fragment.as_bytes(), self.sep).as_span_str(&self.idx)
    }

    type SpanIter<'it> = LocatedSpanStrIter<'it, 's>
    where Self: 'it;
    type RSpanIter<'it> = RLocatedSpanStrIter<'it, 's>
    where Self: 'it;

    fn current<'a>(&'a self, fragment: LocatedSpan<&'i str, Y>) -> Self::SpanIter<'a> {
        let frag = raw::complete_fragment(self.buf, fragment.as_bytes(), self.sep);

        LocatedSpanStrIter {
            sep: self.sep,
            buf: frag.span,
            idx: &self.idx,
            fragment: raw::empty_frame(self.buf, frag.span).span,
        }
    }

    fn iter<'a>(&'a self) -> Self::SpanIter<'a> {
        LocatedSpanStrIter {
            sep: self.sep,
            buf: self.buf,
            idx: &self.idx,
            fragment: raw::empty_frame(self.buf, self.buf).span,
        }
    }

    fn forward_from<'a>(&'a self, fragment: LocatedSpan<&'i str, Y>) -> Self::SpanIter<'a> {
        let frag = raw::end_frame(self.buf, fragment.as_bytes(), self.sep);
        LocatedSpanStrIter {
            sep: self.sep,
            buf: self.buf,
            idx: &self.idx,
            fragment: frag.span,
        }
    }

    fn backward_from<'a>(&'a self, fragment: LocatedSpan<&'i str, Y>) -> Self::RSpanIter<'a> {
        let frag = raw::start_frame(self.buf, fragment.as_bytes(), self.sep);
        RLocatedSpanStrIter {
            sep: self.sep,
            buf: self.buf,
            idx: &self.idx,
            fragment: frag.span,
        }
    }
}

/// Iterates all lines.
#[doc(hidden)]
pub struct LocatedSpanStrIter<'i, 's> {
    sep: u8,
    buf: &'s [u8],
    fragment: &'s [u8],
    idx: &'i [usize],
}

impl<'i, 's> Iterator for LocatedSpanStrIter<'i, 's> {
    type Item = LocatedSpan<&'s str, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let frag = raw::next_fragment(self.buf, self.fragment, self.sep);
        self.fragment = frag.span;
        frag.as_iter_span_str(self.idx)
    }
}

/// Backward iterator.
#[doc(hidden)]
pub struct RLocatedSpanStrIter<'i, 's> {
    sep: u8,
    buf: &'s [u8],
    fragment: &'s [u8],
    idx: &'i [usize],
}

impl<'i, 's> Iterator for RLocatedSpanStrIter<'i, 's> {
    type Item = LocatedSpan<&'s str, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let frag = raw::prev_fragment(self.buf, self.fragment, self.sep);
        self.fragment = frag.span;
        frag.as_iter_span_str(self.idx)
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'i, 's> Source<&'i str> for SourceStr<'s> {
    type Result = &'s str;

    fn with_separator(mut self, sep: u8) -> Self {
        self.sep = sep;
        self.idx = raw::index_lines(self.buf, sep);
        self
    }

    fn with_ascii(mut self, ascii: bool) -> Self {
        self.ascii = ascii;
        self
    }

    fn offset(&self, fragment: &'i str) -> usize {
        raw::offset_from(self.buf, fragment.as_bytes())
    }

    fn line(&self, fragment: &'i str) -> usize {
        raw::line_index(&self.idx, raw::offset_from(self.buf, fragment.as_bytes()))
    }

    fn column(&self, fragment: &'i str) -> usize {
        if self.ascii {
            raw::ascii_column(self.buf.as_bytes(), fragment.as_bytes(), self.sep)
        } else {
            raw::utf8_column(self.buf.as_bytes(), fragment.as_bytes(), self.sep)
        }
    }

    fn location(&self, fragment: &'i str) -> SourceLocation {
        SourceLocation {
            offset: raw::offset_from(self.buf.as_bytes(), fragment.as_bytes()),
            line: raw::line_index(&self.idx, raw::offset_from(self.buf, fragment.as_bytes())),
            column: if self.ascii {
                raw::ascii_column(self.buf.as_bytes(), fragment.as_bytes(), self.sep)
            } else {
                raw::utf8_column(self.buf.as_bytes(), fragment.as_bytes(), self.sep)
            },
        }
    }

    fn get_lines_around(&self, fragment: &'i str, n: usize) -> Vec<&'s str> {
        let mut buf: Vec<_> = self.backward_from(fragment).take(n).collect();
        buf.reverse();
        buf.extend(self.current(fragment));
        buf.extend(self.forward_from(fragment).take(n));

        buf
    }

    fn start(&self, fragment: &'i str) -> &'s str {
        raw::start_frame(self.buf.as_bytes(), fragment.as_bytes(), self.sep).as_str()
    }

    fn end(&self, fragment: &'i str) -> &'s str {
        raw::end_frame(self.buf.as_bytes(), fragment.as_bytes(), self.sep).as_str()
    }

    type SpanIter<'it> = StrIter<'s>
    where Self: 'it;
    type RSpanIter<'it> = RStrIter<'s>
    where Self: 'it;

    fn current<'a>(&'a self, fragment: &'i str) -> Self::SpanIter<'a> {
        let frag = raw::complete_fragment(self.buf.as_bytes(), fragment.as_bytes(), self.sep);

        StrIter {
            sep: self.sep,
            buf: frag.span,
            fragment: raw::empty_frame(self.buf.as_bytes(), frag.span).span,
        }
    }

    fn iter<'a>(&'a self) -> Self::SpanIter<'a> {
        StrIter {
            sep: self.sep,
            buf: self.buf.as_bytes(),
            fragment: raw::empty_frame(self.buf.as_bytes(), self.buf.as_bytes()).span,
        }
    }

    fn forward_from<'a>(&'a self, fragment: &'i str) -> Self::SpanIter<'a> {
        let frag = raw::end_frame(self.buf.as_bytes(), fragment.as_bytes(), self.sep);
        StrIter {
            sep: self.sep,
            buf: self.buf.as_bytes(),
            fragment: frag.span,
        }
    }

    fn backward_from<'a>(&'a self, fragment: &'i str) -> Self::RSpanIter<'a> {
        let frag = raw::start_frame(self.buf.as_bytes(), fragment.as_bytes(), self.sep);
        RStrIter {
            sep: self.sep,
            buf: self.buf.as_bytes(),
            fragment: frag.span,
        }
    }
}

/// Iterates all lines.
#[doc(hidden)]
pub struct StrIter<'s> {
    sep: u8,
    buf: &'s [u8],
    fragment: &'s [u8],
}

impl<'s> Iterator for StrIter<'s> {
    type Item = &'s str;

    fn next(&mut self) -> Option<Self::Item> {
        let next = raw::next_fragment(self.buf, self.fragment, self.sep);
        self.fragment = next.span;
        next.as_iter_str()
    }
}

/// Backward iterator.
#[doc(hidden)]
pub struct RStrIter<'s> {
    sep: u8,
    buf: &'s [u8],
    fragment: &'s [u8],
}

impl<'s> Iterator for RStrIter<'s> {
    type Item = &'s str;

    fn next(&mut self) -> Option<Self::Item> {
        let next = raw::prev_fragment(self.buf, self.fragment, self.sep);
        self.fragment = next.span;
        next.as_iter_str()
    }
}

mod raw {
    use bytecount::num_chars;
    use memchr::{memchr, memchr_iter, memrchr};
    use nom_locate::LocatedSpan;

    #[derive(Debug)]
    #[allow(dead_code)]
    pub(crate) struct MemFragment<'a> {
        pub(crate) start: usize,
        pub(crate) end: usize,
        pub(crate) span: &'a [u8],
        pub(crate) iter_span: Option<&'a [u8]>,
    }

    impl<'a> MemFragment<'a> {
        pub(crate) fn as_str(&self) -> &'a str {
            unsafe { std::str::from_utf8_unchecked(self.span) }
        }

        pub(crate) fn as_iter_str(&self) -> Option<&'a str> {
            self.iter_span
                .map(|v| unsafe { std::str::from_utf8_unchecked(v) })
        }

        pub(crate) fn as_span_str(&self, line_idx: &[usize]) -> LocatedSpan<&'a str, ()> {
            unsafe {
                LocatedSpan::new_from_raw_offset(
                    self.start,
                    line_index(line_idx, self.start) as u32,
                    std::str::from_utf8_unchecked(self.span),
                    (),
                )
            }
        }

        pub(crate) fn as_iter_span_str(
            &self,
            line_idx: &[usize],
        ) -> Option<LocatedSpan<&'a str, ()>> {
            self.iter_span.map(|v| unsafe {
                LocatedSpan::new_from_raw_offset(
                    self.start,
                    line_index(line_idx, self.start) as u32,
                    std::str::from_utf8_unchecked(v),
                    (),
                )
            })
        }

        pub(crate) fn as_bytes(&self) -> &'a [u8] {
            self.span
        }

        pub(crate) fn as_iter_bytes(&self) -> Option<&'a [u8]> {
            self.iter_span
        }

        pub(crate) fn as_span_bytes(&self, line_idx: &[usize]) -> LocatedSpan<&'a [u8], ()> {
            unsafe {
                LocatedSpan::new_from_raw_offset(
                    self.start,
                    line_index(line_idx, self.start) as u32,
                    self.span,
                    (),
                )
            }
        }

        pub(crate) fn as_iter_span_bytes(
            &self,
            line_idx: &[usize],
        ) -> Option<LocatedSpan<&'a [u8], ()>> {
            self.iter_span.map(|v| unsafe {
                LocatedSpan::new_from_raw_offset(
                    self.start,
                    line_index(line_idx, self.start) as u32,
                    v,
                    (),
                )
            })
        }
    }

    pub(crate) fn index_lines(complete: &[u8], sep: u8) -> Vec<usize> {
        memchr_iter(sep, complete).collect()
    }

    pub(crate) fn line_index(line_idx: &[usize], offset: usize) -> usize {
        match line_idx.binary_search(&offset) {
            Ok(v) => v + 1,
            Err(v) => v + 1,
        }
    }

    // pub(crate) fn line(complete: &[u8], fragment: &[u8], sep: u8) -> usize {
    //     let offset = offset_from(complete, fragment);
    //     assert!(offset <= complete.len());
    //     memchr_iter(sep, &complete[..offset]).count() + 1
    // }

    /// Assumes ASCII text and gives a column.
    pub(crate) fn ascii_column(complete: &[u8], fragment: &[u8], sep: u8) -> usize {
        let frag = frame_prefix(complete, fragment, sep);
        frag.span.len()
    }

    /// Gives a column for UTF8 text.
    pub(crate) fn utf8_column(complete: &[u8], fragment: &[u8], sep: u8) -> usize {
        let frag = frame_prefix(complete, fragment, sep);
        num_chars(frag.span)
    }

    /// Returns the part of the frame from the last separator up to the start of the
    /// fragment.
    #[allow(clippy::needless_lifetimes)]
    pub(crate) fn frame_prefix<'s, 'a>(
        complete: &'s [u8],
        fragment: &'a [u8],
        sep: u8,
    ) -> MemFragment<'s> {
        let offset = offset_from(complete, fragment);
        assert!(offset <= complete.len());

        let self_bytes = complete;

        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };

        MemFragment {
            start,
            end: offset,
            span: &complete[start..offset],
            iter_span: None,
        }
    }

    /// Empty span at the beginning of the fragment.
    #[allow(clippy::needless_lifetimes)]
    pub(crate) fn empty_frame<'s, 'a>(complete: &'s [u8], fragment: &'a [u8]) -> MemFragment<'s> {
        let offset = offset_from(complete, fragment);
        assert!(offset <= complete.len());

        MemFragment {
            start: offset,
            end: offset,
            span: &complete[offset..offset],
            iter_span: None,
        }
    }

    /// Return the first full line for the fragment.
    #[allow(clippy::needless_lifetimes)]
    pub(crate) fn start_frame<'s, 'a>(
        complete: &'s [u8],
        fragment: &'a [u8],
        sep: u8,
    ) -> MemFragment<'s> {
        let offset = offset_from(complete, fragment);

        // trim the offset to our bounds.
        assert!(offset <= complete.len());

        // no skip_lines, already correct.

        let self_bytes = complete;
        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(v) => v + 1,
        };
        let end = match memchr(sep, &self_bytes[offset..]) {
            None => complete.len(),
            Some(v) => offset + v + 1,
        };

        MemFragment {
            start,
            end,
            span: &complete[start..end],
            iter_span: None,
        }
    }

    /// Returns the last full frame of the fragment.
    #[allow(clippy::needless_lifetimes)]
    pub(crate) fn end_frame<'s, 'a>(
        complete: &'s [u8],
        fragment: &'a [u8],
        sep: u8,
    ) -> MemFragment<'s> {
        let offset = offset_from(complete, fragment) + fragment.len();

        // trim the offset to our bounds.
        assert!(offset <= complete.len());

        let self_bytes = complete;
        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(v) => v + 1,
        };
        let end = match memchr(sep, &self_bytes[offset..]) {
            None => complete.len(),
            Some(v) => offset + v + 1,
        };

        MemFragment {
            start,
            end,
            span: &complete[start..end],
            iter_span: None,
        }
    }

    /// Completes the fragment to a full frame.
    #[allow(clippy::needless_lifetimes)]
    pub(crate) fn complete_fragment<'s, 'a>(
        complete: &'s [u8],
        fragment: &'a [u8],
        sep: u8,
    ) -> MemFragment<'s> {
        let offset = offset_from(complete, fragment);
        let len = fragment.len();

        // trim start and end to our bounds.
        assert!(offset <= complete.len());
        assert!(offset + len <= complete.len());
        let (start, end) = (offset, offset + len);

        // fill up front and back
        let self_bytes = complete;
        let start = match memrchr(sep, &self_bytes[..start]) {
            None => 0,
            Some(o) => o + 1,
        };
        let end = match memchr(sep, &self_bytes[end..]) {
            None => complete.len(),
            Some(o) => end + o + 1,
        };

        MemFragment {
            start,
            end,
            span: &complete[start..end],
            iter_span: None,
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
    #[allow(clippy::needless_lifetimes)]
    pub(crate) fn next_fragment<'s, 'a>(
        complete: &'s [u8],
        fragment: &'a [u8],
        sep: u8,
    ) -> MemFragment<'s> {
        let offset = offset_from(complete, fragment);
        let len = fragment.len();

        // trim start to our bounds.
        assert!(offset + len <= complete.len());
        let start = offset + len;

        let is_terminal = start == complete.len();

        let self_bytes = complete;
        let end = match memchr(sep, &self_bytes[start..]) {
            None => complete.len(),
            Some(o) => start + o + 1,
        };

        let span = &complete[start..end];

        MemFragment {
            start,
            end,
            span,
            iter_span: if is_terminal { None } else { Some(span) },
        }
    }

    /// Return the preceding frame.
    ///
    /// If the byte immediately preceding the start of the fragment is not the separator,
    /// just a truncated fragment is returned.
    ///
    /// The separator is included at the end of a frame.
    #[allow(clippy::needless_lifetimes)]
    pub(crate) fn prev_fragment<'s, 'a>(
        complete: &'s [u8],
        fragment: &'a [u8],
        sep: u8,
    ) -> MemFragment<'s> {
        let offset = offset_from(complete, fragment);

        // assert our bounds.
        assert!(offset <= complete.len());
        let end = offset;

        // At the beginning?
        let is_terminal = end == 0;

        // immediately preceeding separator.
        let self_bytes = complete;
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

        MemFragment {
            start,
            end,
            span,
            iter_span: if is_terminal { None } else { Some(span) },
        }
    }

    pub(crate) fn offset_from(complete: &[u8], fragment: &[u8]) -> usize {
        let offset = unsafe { fragment.as_ptr().offset_from(complete.as_ptr()) };
        assert!(offset >= 0);
        offset as usize
    }
}

#[cfg(test)]
mod tests_spanbytes {
    use crate::source::raw;
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
                    let prefix = raw::frame_prefix(&txt, &frag, SEP);

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

                    assert_eq!(prefix.span, cmp);
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
                    let next = raw::start_frame(&txt, &frag, SEP);

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

                    assert_eq!(next.span, cmp);
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
                    let next = raw::end_frame(&txt, &frag, SEP);

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

                    assert_eq!(next.span, cmp);
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
                    let next = raw::complete_fragment(&txt, &frag, SEP);

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

                    assert_eq!(next.span, cmp);
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
                    let next = raw::next_fragment(&txt, &frag, SEP);

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

                    assert_eq!(next.span, cmp);
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
                    let prev = raw::prev_fragment(&txt, &frag, SEP);

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

                    assert_eq!(prev.span, cmp);
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
