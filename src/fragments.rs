use bytecount::{naive_num_chars, num_chars};
use memchr::{memchr, memrchr};
use nom::AsBytes;
use std::ops::Range;
use std::slice;

/// Unsafe trait to undo a previous slicing operation.
pub trait Fragment {
    /// Undo taking a slice.
    ///
    /// # Safety
    ///
    /// The offset must be less than isize::MAX and should denote the offset
    /// of this span in the original.
    unsafe fn undo_subslice(self, offset: usize) -> Self;
}

/// This trait provides functions to work with subslices of a buffer.
pub trait BufferFragments<'s, A, T>
where
    Self: Sized + 's,
{
    /// Finds the part from the last separator up to the given fragment
    /// excluding the separator. Starting at the beginning if there is
    /// no separator.
    ///
    /// # Panics
    /// if the fragment is not part of the buffer.
    fn current_prefix(self, fragment: A, sep: T) -> Self;

    /// 0-based column index of the fragment with respect to the previous fragment boundary.
    /// This is assuming all is ascii text.
    ///
    /// # Panics
    /// if the fragment is not part of the buffer.
    fn ascii_column(self, fragment: A, sep: T) -> usize;

    /// 0-based column index of the fragment starting at the preceding separator.
    /// This is counting UTF-8 encoded Unicode codepoints.
    ///
    /// Uses byte_count::num_chars().
    ///
    /// # Panics
    /// if the fragment is not part of the buffer.
    fn utf8_column(self, fragment: A, sep: T) -> usize;

    /// 0-based column index of the fragment with respect to the previous fragment boundary.
    /// This is counting UTF-8 encoded Unicode codepoints.
    /// Naive implementation, might do better in some situations.
    ///
    /// # Panics
    /// if the fragment is not part of the buffer.
    fn naive_utf8_column(self, fragment: A, sep: T) -> usize;

    /// Returns the number of occurrences of the sep.
    fn count(self, sep: T) -> usize;

    /// Completes the fragment on both ends up to but excluding the separator.
    /// It also works if the fragment itself contains a separator.
    ///
    /// Returns the offset of the result slice and the slice itself
    ///
    /// # Panics
    /// if the fragment is not part of the buffer.
    fn complete_fragment(self, fragment: A, sep: T) -> (usize, Self);

    /// Finds the next fragment.
    ///
    /// Returns the offset, a span and a flag that this span is just a empty
    /// placeholder and this is really the end of the buffer.
    ///
    /// This assumes that the fragment ends at a boundary.
    /// If not it skips one character assuming it is the separator and
    /// returns the rest up to the next separator.
    /// To start with a clean fragment call current_frame first.
    ///
    /// Does not include the separator in the returned span.
    ///
    /// # Panics
    /// if the fragment is not part of the buffer.
    fn next_fragment(self, fragment: A, sep: T) -> (usize, Self, bool);

    /// Finds the previous fragment.
    ///
    /// Returns the offset, a span and a flag that this span is just a empty
    /// placeholder and this is really the end of the buffer.
    ///
    /// This assumes that the fragment starts at a boundary.
    /// If not it returns the part from the boundary up to the start of the fragment.
    /// To start with a clean fragment call current_frame first.
    ///
    /// Does not include the separator in the returned span.
    ///
    /// # Panics
    /// if the fragment is not part of the buffer.
    fn prev_fragment(self, fragment: A, sep: T) -> (usize, Self, bool);

    /// Returns everything from the start of the first to the end of the second fragment.
    ///
    /// # Panics
    /// if the fragment is not part of the buffer or if the order of the
    /// fragments is reversed.
    fn union_of(self, first: A, second: A) -> Self;

    /// Returns the byte offset of an fragment relative to an enclosing outer slice.
    ///
    /// # Panics
    /// if the fragment is not part of the buffer.
    fn subslice_offset(self, fragment: A) -> usize;
}

impl<'a, 's> BufferFragments<'s, &'a [u8], u8> for &'s [u8] {
    /// Sees the buffer as
    ///
    /// # Panics
    /// The fragment has to be a fragment of self.    
    fn current_prefix(self, fragment: &'a [u8], sep: u8) -> &'s [u8] {
        let offset = self.subslice_offset(fragment);
        let start = match memrchr(sep, &self[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };
        &self[start..offset]
    }

    fn ascii_column(self, fragment: &'a [u8], sep: u8) -> usize {
        let prefix = self.current_prefix(fragment, sep);
        prefix.len()
    }

    fn utf8_column(self, fragment: &'a [u8], sep: u8) -> usize {
        let prefix = self.current_prefix(fragment, sep);
        num_chars(prefix)
    }

    fn naive_utf8_column(self, fragment: &'a [u8], sep: u8) -> usize {
        let prefix = self.current_prefix(fragment, sep);
        naive_num_chars(prefix)
    }

    fn count(self, sep: u8) -> usize {
        let mut count = 0;

        let mut start = 0;
        loop {
            match memchr(sep, &self[start..]) {
                None => break,
                Some(o) => {
                    count += 1;
                    start = o + 1;
                }
            }
        }

        count
    }

    fn complete_fragment(self, fragment: &'a [u8], sep: u8) -> (usize, &'s [u8]) {
        let offset = self.subslice_offset(fragment);
        let len = fragment.len();

        let start = match memrchr(sep, &self[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };
        let end = match memchr(sep, &self[offset + len..]) {
            None => self.len(),
            Some(o) => offset + len + o,
        };

        (start, &self[start..end])
    }

    fn next_fragment(self, fragment: &'a [u8], sep: u8) -> (usize, &'s [u8], bool) {
        let offset = self.subslice_offset(fragment);

        let (truncate_start, start) = match offset + fragment.len() + 1 {
            n if n > self.len() => (true, self.len()),
            n => (false, n),
        };
        let end = match memchr(sep, &self[start..]) {
            None => self.len(),
            Some(o) => start + o,
        };

        let next_fragment = &self[start..end];

        if truncate_start {
            (start, next_fragment, false)
        } else {
            (start, next_fragment, true)
        }
    }

    fn prev_fragment(self, fragment: &'a [u8], sep: u8) -> (usize, &'s [u8], bool) {
        let offset = self.subslice_offset(fragment);

        let (trunc_end, end) = match offset as isize - 1 {
            -1 => (true, 0),
            n => (false, n as usize),
        };
        let start = match memrchr(sep, &self[..end]) {
            None => 0,
            Some(n) => n + 1,
        };

        let prev_fragment = &self[start..end];

        if trunc_end {
            (start, prev_fragment, false)
        } else {
            (start, prev_fragment, true)
        }
    }

    /// Returns the union of the two slices.
    ///
    /// # Safety
    /// There are assertions that the offsets for the result are within the
    /// bounds of buf.
    ///
    /// But it can't be assured that first and second are derived from buf,
    /// so UB cannot be ruled out.
    ///
    /// So the prerequisite is that both first and second are derived from buf.
    fn union_of(self, first: &'a [u8], second: &'a [u8]) -> &'s [u8] {
        let offset_1 = self.subslice_offset(first);
        let offset_2 = self.subslice_offset(second);
        &self[offset_1..offset_2 + second.len()]
    }

    /// Copied from the crate with the same name.
    ///
    /// Returns the offset of the fragment inside of the buffer.
    fn subslice_offset(self, fragment: &'a [u8]) -> usize {
        if self.as_ptr_range().contains(&fragment.as_bytes().as_ptr()) {
            let outer_start = self.as_ptr() as usize;
            let fragment_start = fragment.as_ptr() as usize;
            fragment_start - outer_start
        } else {
            panic!("subspan");
        }
    }
}

impl<'a, 's> BufferFragments<'s, &'a str, u8> for &'s str {
    /// Finds the part from the last frame start up to the given fragment.
    ///
    /// # Panics
    /// The fragment has to be a fragment of self.
    fn current_prefix(self, fragment: &'a str, sep: u8) -> &'s str {
        let self_bytes = self.as_bytes();

        let offset = self.subslice_offset(fragment);
        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };
        &self[start..offset]
    }

    fn ascii_column(self, fragment: &'a str, sep: u8) -> usize {
        let prefix = self.current_prefix(fragment, sep);
        prefix.len()
    }

    fn utf8_column(self, fragment: &'a str, sep: u8) -> usize {
        let prefix = self.current_prefix(fragment, sep);
        num_chars(prefix.as_bytes())
    }

    fn naive_utf8_column(self, fragment: &'a str, sep: u8) -> usize {
        let prefix = self.current_prefix(fragment, sep);
        naive_num_chars(prefix.as_bytes())
    }

    fn count(self, sep: u8) -> usize {
        let self_bytes = self.as_bytes();
        let mut count = 0;

        let mut start = 0;
        loop {
            match memchr(sep, &self_bytes[start..]) {
                None => break,
                Some(o) => {
                    count += 1;
                    start = o + 1;
                }
            }
        }

        count
    }

    fn complete_fragment(self, fragment: &'a str, sep: u8) -> (usize, &'s str) {
        let offset = self.subslice_offset(fragment);
        let len = fragment.len();

        let self_bytes = self.as_bytes();
        let start = match memrchr(sep, &self_bytes[..offset]) {
            None => 0,
            Some(o) => o + 1,
        };
        let end = match memchr(sep, &self_bytes[offset + len..]) {
            None => self.len(),
            Some(o) => offset + len + o,
        };

        (start, &self[start..end])
    }

    fn next_fragment(self, fragment: &'a str, sep: u8) -> (usize, &'s str, bool) {
        let offset = self.subslice_offset(fragment);

        let (truncate_start, start) = match offset + fragment.len() + 1 {
            n if n > self.len() => (true, self.len()),
            n => (false, n),
        };
        let self_bytes = self.as_bytes();
        let end = match memchr(sep, &self_bytes[start..]) {
            None => self.len(),
            Some(o) => start + o,
        };

        let next_fragment = &self[start..end];

        if truncate_start {
            (start, next_fragment, false)
        } else {
            (start, next_fragment, true)
        }
    }

    fn prev_fragment(self, fragment: &'a str, sep: u8) -> (usize, &'s str, bool) {
        let offset = self.subslice_offset(fragment);

        let (trunc_end, end) = match offset as isize - 1 {
            -1 => (true, 0),
            n => (false, n as usize),
        };
        let self_bytes = self.as_bytes();
        let start = match memrchr(sep, &self_bytes[..end]) {
            None => 0,
            Some(n) => n + 1,
        };

        let prev_fragment = &self[start..end];

        if trunc_end {
            (start, prev_fragment, false)
        } else {
            (start, prev_fragment, true)
        }
    }

    /// Returns the union of the two slices.
    ///
    /// # Panics
    /// If any of first and second is not within the range of self.
    fn union_of(self, first: &'a str, second: &'a str) -> &'s str {
        let offset_1 = self.subslice_offset(first);
        let offset_2 = self.subslice_offset(second);
        &self[offset_1..offset_2 + second.len()]
    }

    /// Returns the offset of fragment in the buffer.
    ///
    /// # Panics
    /// If the fragment is not a part of the buffer.
    fn subslice_offset(self, fragment: &'a str) -> usize {
        // println!(
        //     "{:?} {:?}",
        //     self.as_bytes().as_ptr_range(),
        //     fragment.as_bytes().as_ptr()
        // );

        let Range {
            start: start_self,
            end: end_self,
        } = self.as_bytes().as_ptr_range();

        let frag = fragment.as_bytes().as_ptr();

        if frag >= start_self && frag <= end_self {
            let outer_start = self.as_bytes().as_ptr() as usize;
            let fragment_start = fragment.as_bytes().as_ptr() as usize;
            fragment_start - outer_start
        } else {
            panic!("subspan");
        }
    }
}

impl<'s, T> Fragment for &'s [T] {
    /// Undo taking a slice.
    ///
    /// # Safety
    ///
    /// The offset must be less than isize::MAX and should denote the offset
    /// of this span in the original.
    unsafe fn undo_subslice(self, offset: usize) -> Self {
        assert!(offset < isize::MAX as usize);

        let ptr = self.as_ptr();
        let new_ptr = ptr.offset(-(offset as isize));

        slice::from_raw_parts(new_ptr, offset + self.len())
    }
}

impl<'s> Fragment for &'s str {
    /// Undo taking a slice.
    ///
    /// # Safety
    ///
    /// The offset must be less than isize::MAX and should denote the offset
    /// of this span in the original.
    unsafe fn undo_subslice(self, offset: usize) -> Self {
        assert!(offset < isize::MAX as usize);

        let ptr = self.as_ptr();
        let new_ptr = ptr.offset(-(offset as isize));

        let bytes = slice::from_raw_parts(new_ptr, self.len() + offset);
        std::str::from_utf8_unchecked(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::fragments::Fragment;
    use std::str::Utf8Error;

    #[test]
    fn test_raw1() -> Result<(), Utf8Error> {
        unsafe {
            //              01234567890
            let buf = "1234567890";
            let slice = &buf[4..5];

            let orig = slice.undo_subslice(4);

            assert_eq!(orig, &buf[..5]);
        }

        Ok(())
    }
}
