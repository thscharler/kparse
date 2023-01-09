//!
//! Provides StrLines to split a buffer into lines.
//!
//! BUT it can start from any sub-slice of the buffer, going forward and backward.
//!
//! The main purpose is as a companion for HSpan to get the surroundings of a span.
//!

pub use byte_frames::*;
pub use str_lines::*;

use core::str::from_utf8_unchecked;

///
/// Splits a big blob of data into frames.
///
/// Safety:
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
    /// Safety:
    /// The fragment must be part of the original allocation.
    unsafe fn start(&self, fragment: &'_ T) -> Self::Frame;

    /// Extracts the last completed frame for the given fragment.
    ///
    /// Safety:
    /// The fragment must be part of the original allocation.
    unsafe fn end(&self, fragment: &'_ T) -> Self::Frame;

    /// Extracts all the frames for the given fragment.
    ///
    /// Safety:
    /// The fragment must be part of the original allocation.
    unsafe fn current(&self, fragment: &'_ T) -> Self::Iter;

    /// Iterates all frames of the buffer.
    fn iter(&self) -> Self::Iter;

    /// Returns an iterator over the frames following the given fragment.
    ///
    /// Safety:
    /// The fragment must be part of the original allocation.
    unsafe fn forward_from(&self, fragment: &'_ T) -> Self::FwdIter;

    /// Returns an iterator over the frames preceding the given fragment.
    /// In descending order.
    ///
    /// Safety:
    /// The fragment must be part of the original allocation.
    unsafe fn backward_from(&self, fragment: &'_ T) -> Self::RevIter;

    /// Returns the start-offset of the given fragment.
    ///
    /// Safety:
    /// The fragment must be part of the original allocation.
    unsafe fn offset(&self, fragment: &'_ T) -> usize;
}

/// Returns the union of the two &str slices.
///
/// Safety:
/// There are assertions that the offsets for the result are within the
/// bounds of buf.
///
/// But it can't be assured that first and second are derived from buf,
/// so UB cannot be ruled out.
///
/// So the prerequisite is that both first and second are derived from buf.
pub unsafe fn str_union<'a>(buf: &'a str, first: &'_ str, second: &'_ str) -> &'a str {
    let union = slice_union(buf.as_bytes(), first.as_bytes(), second.as_bytes());
    // both parts where &str, so offset_1, offset_2 and second.len()
    // must obey the utf8 boundaries.
    from_utf8_unchecked(union)
}

/// Returns the union of the two slices.
///
/// Safety:
/// There are assertions that the offsets for the result are within the
/// bounds of buf.
///
/// But it can't be assured that first and second are derived from buf,
/// so UB cannot be ruled out.
///
/// So the prerequisite is that both first and second are derived from buf.
pub unsafe fn slice_union<'a>(buf: &'a [u8], first: &'_ [u8], second: &'_ [u8]) -> &'a [u8] {
    unsafe {
        // fragment_offset checks for a negative offset.
        let offset_1 = finder::fragment_offset(buf, first);
        assert!(offset_1 <= buf.len());

        // fragment_offset checks for a negative offset.
        let offset_2 = finder::fragment_offset(buf, second);
        assert!(offset_2 <= buf.len());

        // correct ordering
        assert!(offset_1 <= offset_2);

        &buf[offset_1..offset_2 + second.len()]
    }
}

mod byte_frames {
    use crate::data_frame::finder;
    use crate::data_frame::DataFrames;

    /// Implements DataFrames for a &[u8] buffer.
    ///
    /// The default DELIMiter is \n for lines of text, but can be overridden with
    /// any other byte-value.
    ///
    /// Safety:
    /// utf8_column and naive_utf8_column can be used regardless of the DELIMiter,
    /// but the results might not meet your expectations.
    #[derive(Debug, Clone)]
    pub struct ByteFrames<'s, const DELIM: u8 = b'\n'> {
        buf: &'s [u8],
    }

    impl<'s, const DELIM: u8> ByteFrames<'s, DELIM> {
        /// New with a buffer.
        pub fn new(buf: &'s [u8]) -> Self {
            Self { buf }
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is assuming all is ascii text.
        ///
        /// Safety:
        /// The fragment really has to be a fragment of buf.
        pub unsafe fn ascii_column(&self, fragment: &[u8]) -> usize {
            finder::ascii_column(self.buf, fragment, DELIM)
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is counting UTF-8 encoded Unicode codepoints.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        //
        /// If the delimiter is one of the utf8 special bytes this doesn't break,
        /// but the result might not meet your expectation.
        pub unsafe fn utf8_column(&self, fragment: &[u8]) -> usize {
            finder::utf8_column(self.buf, fragment, DELIM)
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is counting UTF-8 encoded Unicode codepoints.
        /// Naive implementation, might do better in some situations.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        //
        /// If the delimiter is one of the utf8 special bytes this doesn't break,
        /// but the result might not meet your expectation.
        pub unsafe fn naive_utf8_column(&self, fragment: &[u8]) -> usize {
            finder::naive_utf8_column(self.buf, fragment, DELIM)
        }
    }

    impl<'s, const DELIM: u8> DataFrames<'s, [u8]> for ByteFrames<'s, DELIM> {
        type Frame = &'s [u8];
        type Iter = ByteSliceIter<'s, DELIM>;
        type FwdIter = FByteSliceIter<'s, DELIM>;
        type RevIter = RByteSliceIter<'s, DELIM>;

        /// First full line for the fragment.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn start(&self, fragment: &'_ [u8]) -> Self::Frame {
            let buf = finder::current_frame(self.buf, fragment, DELIM);
            finder::current_frame(buf, &buf[0..0], DELIM)
        }

        /// Last full line for the fragment.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn end(&self, fragment: &'_ [u8]) -> Self::Frame {
            let buf = finder::current_frame(self.buf, fragment, DELIM);
            let len = buf.len();
            finder::current_frame(buf, &buf[len..len], DELIM)
        }

        /// Expand the fragment to cover full lines and return an Iterator for the lines.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn current(&self, fragment: &[u8]) -> Self::Iter {
            ByteSliceIter {
                buf: finder::current_frame(self.buf, fragment, DELIM),
                fragment: None,
            }
        }

        /// Iterator for all lines of the buffer.
        fn iter(&self) -> Self::Iter {
            ByteSliceIter {
                buf: self.buf,
                fragment: None,
            }
        }

        /// Iterator over the lines following the last line of the fragment.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn forward_from(&self, fragment: &[u8]) -> Self::FwdIter {
            let current = self.end(fragment);
            FByteSliceIter {
                buf: self.buf,
                fragment: current,
            }
        }

        /// Iterator over the lines preceeding the first line of the fragment.
        /// In descending order.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn backward_from(&self, fragment: &[u8]) -> Self::RevIter {
            let current = self.start(fragment);
            RByteSliceIter {
                buf: self.buf,
                fragment: current,
            }
        }

        /// Offset in the buffer.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn offset(&self, fragment: &[u8]) -> usize {
            finder::fragment_offset(self.buf, fragment)
        }
    }

    /// Iterates all lines.
    pub struct ByteSliceIter<'s, const DELIM: u8> {
        buf: &'s [u8],
        fragment: Option<&'s [u8]>,
    }

    impl<'s, const DELIM: u8> Iterator for ByteSliceIter<'s, DELIM> {
        type Item = &'s [u8];

        fn next(&mut self) -> Option<Self::Item> {
            // if the constraints where upheld at creation we can be sure
            // nothing bad additionally happens.
            unsafe {
                if let Some(fragment) = self.fragment {
                    let (fragment, result) = finder::next_frame(self.buf, fragment, DELIM);
                    self.fragment = Some(fragment);
                    result
                } else {
                    let fragment = finder::current_frame(self.buf, &self.buf[0..0], DELIM);
                    self.fragment = Some(fragment);
                    Some(fragment)
                }
            }
        }
    }

    /// Forward iterator..
    pub struct FByteSliceIter<'s, const DELIM: u8> {
        buf: &'s [u8],
        fragment: &'s [u8],
    }

    impl<'s, const DELIM: u8> Iterator for FByteSliceIter<'s, DELIM> {
        type Item = &'s [u8];

        fn next(&mut self) -> Option<Self::Item> {
            // if the constraints where upheld at creation we can be sure
            // nothing bad additionally happens.
            unsafe {
                let (fragment, result) = finder::next_frame(self.buf, self.fragment, DELIM);
                self.fragment = fragment;
                return result;
            }
        }
    }

    /// Backward iterator.
    pub struct RByteSliceIter<'s, const DELIM: u8> {
        buf: &'s [u8],
        fragment: &'s [u8],
    }

    impl<'s, const DELIM: u8> Iterator for RByteSliceIter<'s, DELIM> {
        type Item = &'s [u8];

        fn next(&mut self) -> Option<Self::Item> {
            // if the constraints where upheld at creation we can be sure
            // nothing bad additionally happens.
            unsafe {
                let (fragment, result) = finder::prev_frame(self.buf, self.fragment, DELIM);

                self.fragment = fragment;
                return result;
            }
        }
    }
}

mod str_lines {
    use crate::data_frame::finder;
    use crate::data_frame::DataFrames;
    use std::str::from_utf8_unchecked;

    const DELIM: u8 = b'\n';

    /// Implements DataFrames for a &str buffer.
    ///
    /// This uses '\n' as frame separator, as in 'give me lines of text'.
    ///
    #[derive(Debug, Clone)]
    pub struct StrLines<'s> {
        buf: &'s str,
    }

    impl<'s> StrLines<'s> {
        /// New with a buffer.
        pub fn new(buf: &'s str) -> Self {
            Self { buf }
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is assuming all is ascii text.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        pub unsafe fn ascii_column(&self, fragment: &str) -> usize {
            finder::ascii_column(self.buf.as_bytes(), fragment.as_bytes(), DELIM)
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is counting UTF-8 encoded Unicode codepoints.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        pub unsafe fn utf8_column(&self, fragment: &str) -> usize {
            // \n is none of the utf8 specials so at least we can be sure of the count.
            finder::utf8_column(self.buf.as_bytes(), fragment.as_bytes(), DELIM)
        }

        /// 0-based column index of the fragment with respect to the previous fragment boundary.
        /// This is counting UTF-8 encoded Unicode codepoints.
        /// Naive implementation, might do better in some situations.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        pub unsafe fn naive_utf8_column(&self, fragment: &str) -> usize {
            // \n is none of the utf8 specials so at least we can be sure of the count.
            finder::naive_utf8_column(self.buf.as_bytes(), fragment.as_bytes(), DELIM)
        }
    }

    impl<'s> DataFrames<'s, str> for StrLines<'s> {
        type Frame = &'s str;
        type Iter = StrIter<'s>;
        type FwdIter = FStrIter<'s>;
        type RevIter = RStrIter<'s>;

        /// First full line for the fragment.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn start(&self, fragment: &'_ str) -> Self::Frame {
            let buf = finder::current_frame(self.buf.as_bytes(), fragment.as_bytes(), DELIM);
            // \n is none of the utf8 specials, so as all inputs are str
            // the result must be too.
            from_utf8_unchecked(finder::current_frame(buf, &buf[0..0], DELIM))
        }

        /// Last full line for the fragment.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn end(&self, fragment: &'_ str) -> Self::Frame {
            let buf = finder::current_frame(self.buf.as_bytes(), fragment.as_bytes(), DELIM);
            let len = buf.len();
            // \n is none of the utf8 specials, so as all inputs are str
            // the result must be too.
            from_utf8_unchecked(finder::current_frame(buf, &buf[len..len], DELIM))
        }

        /// Expand the fragment to cover full lines and return an Iterator for the lines.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn current(&self, fragment: &str) -> Self::Iter {
            StrIter {
                // \n is none of the utf8 specials, so as all inputs are str
                // the result must be too.
                buf: from_utf8_unchecked(finder::current_frame(
                    self.buf.as_bytes(),
                    fragment.as_bytes(),
                    DELIM,
                )),
                fragment: None,
            }
        }

        /// Iterator for all lines of the buffer.
        fn iter(&self) -> Self::Iter {
            StrIter {
                buf: self.buf,
                fragment: None,
            }
        }

        /// Iterator over the lines following the last line of the fragment.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn forward_from(&self, fragment: &str) -> Self::FwdIter {
            let current = self.end(fragment);
            FStrIter {
                buf: self.buf,
                fragment: current,
            }
        }

        /// Iterator over the lines preceeding the first line of the fragment.
        /// In descending order.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn backward_from(&self, fragment: &str) -> Self::RevIter {
            let current = self.start(fragment);
            RStrIter {
                buf: self.buf,
                fragment: current,
            }
        }

        /// Offset in the buffer.
        ///
        /// Safety: The fragment really has to be a fragment of buf.
        unsafe fn offset(&self, fragment: &str) -> usize {
            finder::fragment_offset(self.buf.as_bytes(), fragment.as_bytes())
        }
    }

    /// Iterates all lines.
    pub struct StrIter<'s> {
        buf: &'s str,
        fragment: Option<&'s str>,
    }

    impl<'s> Iterator for StrIter<'s> {
        type Item = &'s str;

        fn next(&mut self) -> Option<Self::Item> {
            // \n is none of the utf8 specials, so as all inputs are str
            // the result must be too.
            unsafe {
                if let Some(fragment) = self.fragment {
                    let (fragment, result) =
                        finder::next_frame(self.buf.as_bytes(), fragment.as_bytes(), DELIM);

                    self.fragment = Some(from_utf8_unchecked(fragment));
                    result.map(|v| from_utf8_unchecked(v))
                } else {
                    let fragment = finder::current_frame(
                        self.buf.as_bytes(),
                        &self.buf[0..0].as_bytes(),
                        DELIM,
                    );

                    self.fragment = Some(from_utf8_unchecked(fragment));
                    Some(from_utf8_unchecked(fragment))
                }
            }
        }
    }

    /// Forward iterator..
    pub struct FStrIter<'s> {
        buf: &'s str,
        fragment: &'s str,
    }

    impl<'s> Iterator for FStrIter<'s> {
        type Item = &'s str;

        fn next(&mut self) -> Option<Self::Item> {
            // \n is none of the utf8 specials, so as all inputs are str
            // the result must be too.
            unsafe {
                let (fragment, result) =
                    finder::next_frame(self.buf.as_bytes(), self.fragment.as_bytes(), DELIM);

                self.fragment = from_utf8_unchecked(fragment);
                return result.map(|v| from_utf8_unchecked(v));
            }
        }
    }

    /// Backward iterator.
    pub struct RStrIter<'s> {
        buf: &'s str,
        fragment: &'s str,
    }

    impl<'s> Iterator for RStrIter<'s> {
        type Item = &'s str;

        fn next(&mut self) -> Option<Self::Item> {
            // \n is none of the utf8 specials, so as all inputs are str
            // the result must be too.
            unsafe {
                let (fragment, result) =
                    finder::prev_frame(self.buf.as_bytes(), self.fragment.as_bytes(), DELIM);

                self.fragment = from_utf8_unchecked(fragment);
                return result.map(|v| from_utf8_unchecked(v));
            }
        }
    }
}

mod strings {
    use crate::data_frame::finder::undo_take_slice;
    use std::str::{from_utf8, from_utf8_unchecked, Utf8Error};

    /// Undo taking a slice and correct for the given offset into the original &str.
    ///
    /// Safety
    /// offset must be within the original bounds.
    #[allow(dead_code)]
    pub unsafe fn undo_take_str_slice(s: &str, offset: usize) -> Result<&str, Utf8Error> {
        let bytes = undo_take_slice(s.as_bytes(), offset);
        from_utf8(bytes)
    }

    /// Undo taking a slice and correct for the given offset into the original &str.
    ///
    /// Safety
    /// offset must be within the original bounds.
    /// offset must not hit between an utf8 boundary.
    #[allow(dead_code)]
    pub unsafe fn undo_take_str_slice_unchecked(s: &str, offset: usize) -> &str {
        let bytes = undo_take_slice(s.as_bytes(), offset);
        from_utf8_unchecked(bytes)
    }
}

mod finder {
    use bytecount::{naive_num_chars, num_chars};
    use memchr::{memchr, memrchr};
    use std::slice;

    /// 0-based column index of the fragment with respect to the previous fragment boundary.
    /// This is assuming all is ascii text.
    ///
    /// Safety: The fragment really has to be a fragment of buf.
    pub unsafe fn ascii_column<'s>(buf: &'s [u8], fragment: &'_ [u8], sep: u8) -> usize {
        let prefix = current_prefix(buf, fragment, sep);
        prefix.len()
    }

    /// 0-based column index of the fragment with respect to the previous fragment boundary.
    /// This is counting UTF-8 encoded Unicode codepoints.
    ///
    /// Safety:
    /// The fragment really has to be a fragment of buf.
    ///
    /// The fragment must not be inside a utf8 sequence and the separator must not
    /// be a utf8 special, otherwise the result will be off. But not UB.
    pub unsafe fn utf8_column<'s>(buf: &'s [u8], fragment: &'_ [u8], sep: u8) -> usize {
        let prefix = current_prefix(buf, fragment, sep);
        num_chars(prefix)
    }

    /// 0-based column index of the fragment with respect to the previous fragment boundary.
    /// This is counting UTF-8 encoded Unicode codepoints.
    /// Naive implementation, might do better in some situations.
    ///
    /// Safety:
    /// The fragment really has to be a fragment of buf.
    ///
    /// The fragment must not be inside a utf8 sequence and the separator must not
    /// be a utf8 special, otherwise the result will be off. But not UB.
    pub unsafe fn naive_utf8_column<'s>(buf: &'s [u8], fragment: &'_ [u8], sep: u8) -> usize {
        let prefix = current_prefix(buf, fragment, sep);
        naive_num_chars(prefix)
    }

    /// Finds the part from the last frame start up to the given fragment.
    ///
    /// Safety:
    /// The fragment really has to be a fragment of buf.
    pub unsafe fn current_prefix<'s>(buf: &'s [u8], fragment: &'_ [u8], sep: u8) -> &'s [u8] {
        let offset = fragment_offset(buf, fragment);

        let start = match memrchr(sep, &buf[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };

        &buf[start..offset]
    }

    /// Finds the frame that contains the given fragment.
    ///
    /// It works if the fragment itself contains the separator.
    /// The result is the fragment expanded at both ends.
    ///
    /// Safety: The fragment really has to be a fragment of buf.
    pub unsafe fn current_frame<'s>(buf: &'s [u8], fragment: &'_ [u8], sep: u8) -> &'s [u8] {
        let offset = fragment_offset(buf, fragment);
        let len = fragment.len();

        let start = match memrchr(sep, &buf[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };
        let end = match memchr(sep, &buf[offset + len..]) {
            None => buf.len(),
            Some(o) => offset + len + o,
        };

        &buf[start..end]
    }

    /// Finds the next frame.
    /// Returns the next frame even if it's at the end of the buffer,
    /// and a second copy wrapped in an Option to indicate that it's indeed the end and
    /// not an intermediary empty frame.
    ///
    /// Safety: The fragment really has to be a fragment of buf.
    pub unsafe fn next_frame<'s>(
        buf: &'s [u8],
        fragment: &'_ [u8],
        sep: u8,
    ) -> (&'s [u8], Option<&'s [u8]>) {
        let offset = unsafe { fragment_offset(buf, fragment) };

        let (trunc_start, start) = match offset + fragment.len() + 1 {
            n if n > buf.len() => (true, buf.len()),
            n => (false, n),
        };
        let end = match memchr(sep, &buf[start..]) {
            None => buf.len(),
            Some(o) => start + o,
        };

        let next_fragment = &buf[start..end];

        if trunc_start {
            (next_fragment, None)
        } else {
            (next_fragment, Some(next_fragment))
        }
    }

    /// Finds the previous frame.
    ///
    /// Returns the previous frame even if it's at the beginning of the buffer,
    /// and a second copy wrapped in an Option to indicate that it's indeed the beginning and
    /// not an intermediary empty frame.
    ///
    /// This assumes that the fragment starts at a boundary.
    /// If not it returns the part from the previous boundary up to the start.
    /// To start with a clean fragment call current_frame first.
    ///
    /// Safety: The fragment really has to be a fragment of buf.
    pub unsafe fn prev_frame<'s>(
        buf: &'s [u8],
        fragment: &'_ [u8],
        sep: u8,
    ) -> (&'s [u8], Option<&'s [u8]>) {
        let offset = unsafe { fragment_offset(buf, fragment) };

        let (trunc_end, end) = match offset as isize - 1 {
            -1 => (true, 0),
            n => (false, n as usize),
        };
        let start = match memrchr(sep, &buf[..end]) {
            None => 0,
            Some(n) => n + 1,
        };

        let prev_fragment = &buf[start..end];

        if trunc_end {
            (prev_fragment, None)
        } else {
            (prev_fragment, Some(prev_fragment))
        }
    }

    /// Gets the offset of the fragment.
    ///
    /// Safety:
    /// The fragment really has to be a fragment of buf.
    pub unsafe fn fragment_offset<'s>(buf: &'s [u8], fragment: &'_ [u8]) -> usize {
        let o = buf.as_ptr();
        let f = fragment.as_ptr();

        let offset = f.offset_from(o);
        assert!(offset >= 0);

        offset as usize
    }

    /// Undo taking a slice and correct for the given offset into the original &[u8].
    ///
    /// Safety
    /// offset must be within the original bounds.
    pub unsafe fn undo_take_slice(s: &[u8], offset: usize) -> &[u8] {
        assert!(offset < isize::MAX as usize);

        let ptr = s.as_ptr();
        let new_ptr = ptr.offset(-(offset as isize));

        slice::from_raw_parts(new_ptr, s.len() + offset)
    }

    #[cfg(test)]
    mod tests {
        use crate::data_frame::finder::undo_take_slice;
        use std::str::{from_utf8, Utf8Error};

        #[test]
        fn test_raw1() -> Result<(), Utf8Error> {
            unsafe {
                //              01234567890
                let buf = "1234567890";
                let slice = &buf[4..5];

                let orig = from_utf8(undo_take_slice(slice.as_bytes(), 4))?;

                assert_eq!(orig, &buf[..5]);
            }

            Ok(())
        }
    }
}
