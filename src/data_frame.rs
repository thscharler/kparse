//!
//! Provides StrLines to split a buffer into lines.
//!
//! BUT it can start from any sub-slice of the buffer, going forward and backward.
//!
//! The main purpose is as a companion for HSpan to get the surroundings of a span.
//!

pub use byte_frames::*;
pub use span_lines::*;
pub use str_lines::*;

///
/// Splits a big blob of data into frames.
///
/// # Safety
/// The fragment must be part of the original allocation.
///
pub trait DataFrames<'a, T: ?Sized> {
    /// Type of the extracted frames.
    type Frame;

    /// Iterator for all frames.
    type Iter: Iterator<Item = Self::Frame>;

    /// Forward iterator over the frames.
    type FwdIter: Iterator<Item = Self::Frame>;

    /// Reverse iterator over the frames.
    type RevIter: Iterator<Item = Self::Frame>;

    /// Extracts the completed first frame for the given fragment.
    ///
    /// # Safety
    /// The fragment must be part of the original allocation.
    fn start(&self, fragment: &'_ T) -> Self::Frame;

    /// Extracts the last completed frame for the given fragment.
    ///
    /// # Safety
    /// The fragment must be part of the original allocation.
    fn end(&self, fragment: &'_ T) -> Self::Frame;

    /// Extracts all the frames for the given fragment.
    ///
    /// # Safety
    /// The fragment must be part of the original allocation.
    fn current(&self, fragment: &'_ T) -> Self::Iter;

    /// Iterates all frames of the buffer.
    fn iter(&self) -> Self::Iter;

    /// Returns an iterator over the frames following the given fragment.
    ///
    /// # Safety
    /// The fragment must be part of the original allocation.
    fn forward_from(&self, fragment: &'_ T) -> Self::FwdIter;

    /// Returns an iterator over the frames preceding the given fragment.
    /// In descending order.
    ///
    /// # Safety
    /// The fragment must be part of the original allocation.
    fn backward_from(&self, fragment: &'_ T) -> Self::RevIter;

    /// Returns the start-offset of the given fragment.
    ///
    /// # Safety
    /// The fragment must be part of the original allocation.
    fn offset(&self, fragment: &'_ T) -> usize;
}

mod byte_frames {
    use crate::data_frame::DataFrames;
    use crate::fragments::BufferFragments;

    /// Implements DataFrames for a &[u8] buffer.
    ///
    /// The default DELIMiter is \n for lines of text, but can be overridden with
    /// any other byte-value.
    ///
    /// # Safety
    /// utf8_column and naive_utf8_column can be used regardless of the DELIMiter,
    /// but the results might not meet your expectations.
    #[derive(Debug, Clone)]
    pub struct ByteFrames<'s> {
        delim: u8,
        buf: &'s [u8],
    }

    impl<'s> ByteFrames<'s> {
        /// New with a buffer.
        pub fn new(buf: &'s [u8]) -> Self {
            Self { delim: b'\n', buf }
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is assuming all is ascii text.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        pub fn ascii_column(&self, fragment: &[u8]) -> usize {
            self.buf.ascii_column(fragment, self.delim)
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is counting UTF-8 encoded Unicode codepoints.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        ///
        /// If the delimiter is one of the utf8 special bytes this doesn't break,
        /// but the result might not meet your expectation.
        pub fn utf8_column(&self, fragment: &[u8]) -> usize {
            self.buf.utf8_column(fragment, self.delim)
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is counting UTF-8 encoded Unicode codepoints.
        /// Naive implementation, might do better in some situations.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        /// If the delimiter is one of the utf8 special bytes this doesn't break,
        /// but the result might not meet your expectation.
        pub fn naive_utf8_column(&self, fragment: &[u8]) -> usize {
            self.buf.naive_utf8_column(fragment, self.delim)
        }
    }

    impl<'s> DataFrames<'s, [u8]> for ByteFrames<'s> {
        type Frame = &'s [u8];
        type Iter = ByteSliceIter<'s>;
        type FwdIter = FByteSliceIter<'s>;
        type RevIter = RByteSliceIter<'s>;

        /// First full line for the fragment.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn start(&self, fragment: &'_ [u8]) -> Self::Frame {
            let frag = self.buf.complete_fragment(fragment, self.delim).1;
            frag.complete_fragment(&frag[0..0], self.delim).1
        }

        /// Last full line for the fragment.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn end(&self, fragment: &'_ [u8]) -> Self::Frame {
            let frag = self.buf.complete_fragment(fragment, self.delim).1;
            let len = frag.len();
            frag.complete_fragment(&frag[len..len], self.delim).1
        }

        /// Expand the fragment to cover full lines and return an Iterator for the lines.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn current(&self, fragment: &[u8]) -> Self::Iter {
            ByteSliceIter {
                delim: self.delim,
                buf: self.buf.complete_fragment(fragment, self.delim).1,
                fragment: None,
            }
        }

        /// Iterator for all lines of the buffer.
        fn iter(&self) -> Self::Iter {
            ByteSliceIter {
                delim: self.delim,
                buf: self.buf,
                fragment: None,
            }
        }

        /// Iterator over the lines following the last line of the fragment.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn forward_from(&self, fragment: &[u8]) -> Self::FwdIter {
            let current = self.end(fragment);
            FByteSliceIter {
                delim: self.delim,
                buf: self.buf,
                fragment: current,
            }
        }

        /// Iterator over the lines preceeding the first line of the fragment.
        /// In descending order.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn backward_from(&self, fragment: &[u8]) -> Self::RevIter {
            let current = self.start(fragment);
            RByteSliceIter {
                delim: self.delim,
                buf: self.buf,
                fragment: current,
            }
        }

        /// Offset in the buffer.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn offset(&self, fragment: &[u8]) -> usize {
            self.buf.subslice_offset(fragment)
        }
    }

    /// Iterates all lines.
    pub struct ByteSliceIter<'s> {
        delim: u8,
        buf: &'s [u8],
        fragment: Option<&'s [u8]>,
    }

    impl<'s> Iterator for ByteSliceIter<'s> {
        type Item = &'s [u8];

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(fragment) = self.fragment {
                let (_offset, fragment, result) = self.buf.next_fragment(fragment, self.delim);
                self.fragment = Some(fragment);
                if result {
                    self.fragment
                } else {
                    None
                }
            } else {
                let (_offset, fragment) = self.buf.complete_fragment(&self.buf[0..0], self.delim);
                self.fragment = Some(fragment);
                Some(fragment)
            }
        }
    }

    /// Forward iterator..
    pub struct FByteSliceIter<'s> {
        delim: u8,
        buf: &'s [u8],
        fragment: &'s [u8],
    }

    impl<'s> Iterator for FByteSliceIter<'s> {
        type Item = &'s [u8];

        fn next(&mut self) -> Option<Self::Item> {
            let (_offset, fragment, result) = self.buf.next_fragment(self.fragment, self.delim);
            self.fragment = fragment;
            if result {
                Some(self.fragment)
            } else {
                None
            }
        }
    }

    /// Backward iterator.
    pub struct RByteSliceIter<'s> {
        delim: u8,
        buf: &'s [u8],
        fragment: &'s [u8],
    }

    impl<'s> Iterator for RByteSliceIter<'s> {
        type Item = &'s [u8];

        fn next(&mut self) -> Option<Self::Item> {
            let (_offset, fragment, result) = self.buf.prev_fragment(self.fragment, self.delim);

            self.fragment = fragment;
            if result {
                Some(self.fragment)
            } else {
                None
            }
        }
    }
}

mod span_lines {
    use crate::data_frame::DataFrames;
    use crate::fragments::BufferFragments;
    use nom_locate::LocatedSpan;

    /// Split a LocatedSpan into lines.
    ///
    /// Helps locating a parsed span inside the original parse text and iterating over
    /// adjacent lines.
    #[derive(Debug)]
    pub struct SpanLines<'s, X> {
        delim: u8,
        buf: LocatedSpan<&'s str, X>,
    }

    impl<'s, X: Copy + 's> SpanLines<'s, X> {
        /// Create a new SpanLines buffer.
        pub fn new(buf: LocatedSpan<&'s str, X>) -> Self {
            Self { delim: b'\n', buf }
        }

        /// Return n lines before and after the fragment, and place the lines of the fragment
        /// between them.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        pub fn get_lines_around(
            &self,
            fragment: &LocatedSpan<&'s str, X>,
            n: usize,
        ) -> Vec<LocatedSpan<&'s str, X>> {
            let mut buf: Vec<_> = self.backward_from(fragment).take(n).collect();
            buf.reverse();
            buf.extend(self.current(fragment));
            buf.extend(self.forward_from(fragment).take(n));

            buf
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is assuming all is ascii text.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        pub fn ascii_column(&self, fragment: &LocatedSpan<&'s str, X>) -> usize {
            self.buf
                .fragment()
                .ascii_column(fragment.fragment(), self.delim)
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is counting UTF-8 encoded Unicode codepoints.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        pub fn utf8_column(&self, fragment: &LocatedSpan<&'s str, X>) -> usize {
            self.buf
                .fragment()
                .utf8_column(fragment.fragment(), self.delim)
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is counting UTF-8 encoded Unicode codepoints.
        /// Naive implementation, might do better in some situations.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        pub fn naive_utf8_column(&self, fragment: &LocatedSpan<&'s str, X>) -> usize {
            self.buf
                .fragment()
                .naive_utf8_column(fragment.fragment(), self.delim)
        }
    }

    impl<'s, X: Copy + 's> DataFrames<'s, LocatedSpan<&'s str, X>> for SpanLines<'s, X> {
        type Frame = LocatedSpan<&'s str, X>;
        type Iter = SpanIter<'s, X>;
        type FwdIter = FSpanIter<'s, X>;
        type RevIter = RSpanIter<'s, X>;

        /// First full line for the fragment.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn start(&self, fragment: &LocatedSpan<&'s str, X>) -> Self::Frame {
            let (_offset, current) = self
                .buf
                .fragment()
                .complete_fragment(fragment.fragment(), self.delim);

            let (offset, current) = current.complete_fragment(&current[0..0], self.delim);

            unsafe {
                LocatedSpan::new_from_raw_offset(
                    offset,
                    fragment.location_line(),
                    current,
                    fragment.extra,
                )
            }
        }

        /// Last full line for the fragment.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn end(&self, fragment: &LocatedSpan<&'s str, X>) -> Self::Frame {
            let (_offset, current) = self
                .buf
                .fragment()
                .complete_fragment(fragment.fragment(), self.delim);
            let len = current.len();
            let lines = current.count(self.delim) as u32;
            let (offset, current) = current.complete_fragment(&current[len..len], self.delim);

            unsafe {
                LocatedSpan::new_from_raw_offset(
                    offset,
                    fragment.location_line() + lines,
                    current,
                    fragment.extra,
                )
            }
        }

        /// Expand the fragment to cover full lines and return an Iterator for the lines.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn current(&self, fragment: &LocatedSpan<&'s str, X>) -> Self::Iter {
            let (offset, current) = self
                .buf
                .fragment()
                .complete_fragment(fragment.fragment(), self.delim);

            SpanIter {
                // \n is none of the utf8 specials, so as all inputs are str
                // the result must be too.
                delim: self.delim,
                buf: unsafe {
                    LocatedSpan::new_from_raw_offset(
                        offset,
                        fragment.location_line(),
                        current,
                        fragment.extra,
                    )
                },
                fragment: None,
            }
        }

        /// Iterator for all lines of the buffer.
        fn iter(&self) -> Self::Iter {
            SpanIter {
                delim: self.delim,
                buf: self.buf,
                fragment: None,
            }
        }

        /// Iterator over the lines following the last line of the fragment.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn forward_from(&self, fragment: &LocatedSpan<&'s str, X>) -> Self::FwdIter {
            let current = self.end(fragment);
            FSpanIter {
                delim: self.delim,
                buf: self.buf,
                fragment: current,
            }
        }

        /// Iterator over the lines preceeding the first line of the fragment.
        /// In descending order.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn backward_from(&self, fragment: &LocatedSpan<&'s str, X>) -> Self::RevIter {
            let current = self.start(fragment);
            RSpanIter {
                delim: self.delim,
                buf: self.buf,
                fragment: current,
            }
        }

        /// Offset in the buffer.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn offset(&self, fragment: &LocatedSpan<&'s str, X>) -> usize {
            self.buf.fragment().subslice_offset(fragment.fragment())
        }
    }

    /// Iterates all lines.
    pub struct SpanIter<'s, X> {
        delim: u8,
        buf: LocatedSpan<&'s str, X>,
        fragment: Option<LocatedSpan<&'s str, X>>,
    }

    impl<'s, X: Copy> Iterator for SpanIter<'s, X> {
        type Item = LocatedSpan<&'s str, X>;

        fn next(&mut self) -> Option<Self::Item> {
            // \n is none of the utf8 specials, so as all inputs are str
            // the result must be too.
            if let Some(old_fragment) = self.fragment {
                let (offset, fragment, result) =
                    self.buf.next_fragment(old_fragment.fragment(), self.delim);

                self.fragment = Some(unsafe {
                    LocatedSpan::new_from_raw_offset(
                        offset,
                        old_fragment.location_line() + 1,
                        fragment,
                        self.buf.extra,
                    )
                });
                if result {
                    self.fragment
                } else {
                    None
                }
            } else {
                let (offset, fragment) = self.buf.complete_fragment(&self.buf[0..0], self.delim);

                self.fragment = Some(unsafe {
                    LocatedSpan::new_from_raw_offset(offset, 1, fragment, self.buf.extra)
                });

                self.fragment
            }
        }
    }

    /// Forward iterator..
    pub struct FSpanIter<'s, X> {
        delim: u8,
        buf: LocatedSpan<&'s str, X>,
        fragment: LocatedSpan<&'s str, X>,
    }

    impl<'s, X: Copy> Iterator for FSpanIter<'s, X> {
        type Item = LocatedSpan<&'s str, X>;

        fn next(&mut self) -> Option<Self::Item> {
            let (offset, fragment, result) = self
                .buf
                .fragment()
                .next_fragment(self.fragment.fragment(), self.delim);

            self.fragment = unsafe {
                LocatedSpan::new_from_raw_offset(
                    offset,
                    self.fragment.location_line() + 1,
                    fragment,
                    self.buf.extra,
                )
            };
            if result {
                Some(self.fragment)
            } else {
                None
            }
        }
    }

    /// Backward iterator.
    pub struct RSpanIter<'s, X> {
        delim: u8,
        buf: LocatedSpan<&'s str, X>,
        fragment: LocatedSpan<&'s str, X>,
    }

    impl<'s, X: Copy> Iterator for RSpanIter<'s, X> {
        type Item = LocatedSpan<&'s str, X>;

        fn next(&mut self) -> Option<Self::Item> {
            let (offset, fragment, result) = self
                .buf
                .fragment()
                .prev_fragment(self.fragment.fragment(), self.delim);

            self.fragment = unsafe {
                LocatedSpan::new_from_raw_offset(
                    offset,
                    self.fragment.location_line() - 1,
                    fragment,
                    self.buf.extra,
                )
            };
            if result {
                Some(self.fragment)
            } else {
                None
            }
        }
    }
}

mod str_lines {
    use crate::data_frame::DataFrames;
    use crate::fragments::BufferFragments;

    /// Implements DataFrames for a &str buffer.
    ///
    /// This uses '\n' as frame separator, as in 'give me lines of text'.
    ///
    #[derive(Debug, Clone)]
    pub struct StrLines<'s> {
        delim: u8,
        buf: &'s str,
    }

    impl<'s> StrLines<'s> {
        /// New with a buffer.
        pub fn new(buf: &'s str) -> Self {
            Self { delim: b'\n', buf }
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is assuming all is ascii text.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        pub fn ascii_column(&self, fragment: &str) -> usize {
            self.buf.ascii_column(fragment, self.delim)
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is counting UTF-8 encoded Unicode codepoints.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        pub fn utf8_column(&self, fragment: &str) -> usize {
            self.buf.utf8_column(fragment, self.delim)
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is counting UTF-8 encoded Unicode codepoints.
        /// Naive implementation, might do better in some situations.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        pub fn naive_utf8_column(&self, fragment: &str) -> usize {
            self.buf.naive_utf8_column(fragment, self.delim)
        }
    }

    impl<'s> DataFrames<'s, str> for StrLines<'s> {
        type Frame = &'s str;
        type Iter = StrIter<'s>;
        type FwdIter = FStrIter<'s>;
        type RevIter = RStrIter<'s>;

        /// First full line for the fragment.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn start(&self, fragment: &'_ str) -> Self::Frame {
            let frag = self.buf.complete_fragment(fragment, self.delim).1;
            frag.complete_fragment(&frag[0..0], self.delim).1
        }

        /// Last full line for the fragment.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn end(&self, fragment: &'_ str) -> Self::Frame {
            let frag = self.buf.complete_fragment(fragment, self.delim).1;
            let len = frag.len();
            frag.complete_fragment(&frag[len..len], self.delim).1
        }

        /// Expand the fragment to cover full lines and return an Iterator for the lines.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn current(&self, fragment: &str) -> Self::Iter {
            StrIter {
                delim: self.delim,
                buf: self.buf.complete_fragment(fragment, self.delim).1,
                fragment: None,
            }
        }

        /// Iterator for all lines of the buffer.
        fn iter(&self) -> Self::Iter {
            StrIter {
                delim: self.delim,
                buf: self.buf,
                fragment: None,
            }
        }

        /// Iterator over the lines following the last line of the fragment.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn forward_from(&self, fragment: &str) -> Self::FwdIter {
            let current = self.end(fragment);
            FStrIter {
                delim: self.delim,
                buf: self.buf,
                fragment: current,
            }
        }

        /// Iterator over the lines preceeding the first line of the fragment.
        /// In descending order.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn backward_from(&self, fragment: &str) -> Self::RevIter {
            let current = self.start(fragment);
            RStrIter {
                delim: self.delim,
                buf: self.buf,
                fragment: current,
            }
        }

        /// Offset in the buffer.
        ///
        /// # Safety The fragment really has to be a fragment of buf.
        fn offset(&self, fragment: &str) -> usize {
            self.buf.subslice_offset(fragment)
        }
    }

    /// Iterates all lines.
    pub struct StrIter<'s> {
        delim: u8,
        buf: &'s str,
        fragment: Option<&'s str>,
    }

    impl<'s> Iterator for StrIter<'s> {
        type Item = &'s str;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(fragment) = self.fragment {
                let (_offset, fragment, result) = self.buf.next_fragment(fragment, self.delim);

                self.fragment = Some(fragment);
                if result {
                    self.fragment
                } else {
                    None
                }
            } else {
                let (_offset, fragment) = self.buf.complete_fragment(&self.buf[0..0], self.delim);

                self.fragment = Some(fragment);
                self.fragment
            }
        }
    }

    /// Forward iterator..
    pub struct FStrIter<'s> {
        delim: u8,
        buf: &'s str,
        fragment: &'s str,
    }

    impl<'s> Iterator for FStrIter<'s> {
        type Item = &'s str;

        fn next(&mut self) -> Option<Self::Item> {
            let (_offset, fragment, result) = self.buf.next_fragment(self.fragment, self.delim);

            self.fragment = fragment;
            if result {
                Some(self.fragment)
            } else {
                None
            }
        }
    }

    /// Backward iterator.
    pub struct RStrIter<'s> {
        delim: u8,
        buf: &'s str,
        fragment: &'s str,
    }

    impl<'s> Iterator for RStrIter<'s> {
        type Item = &'s str;

        fn next(&mut self) -> Option<Self::Item> {
            let (_offset, fragment, result) = self.buf.prev_fragment(self.fragment, self.delim);

            self.fragment = fragment;
            if result {
                Some(self.fragment)
            } else {
                None
            }
        }
    }
}
