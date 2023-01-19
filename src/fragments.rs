pub use bytes::*;
pub use strs::*;

/// Unsafe trait to undo a previous slicing operation.
pub trait UndoSlicing<T> {
    /// Undo taking a slice.
    ///
    /// # Safety
    ///
    /// The offset must be less than isize::MAX and should denote the offset
    /// of this span in the original.
    unsafe fn undo_slice(&self, offset: usize) -> T;

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
    unsafe fn union_slice(&self, first: T, second: T) -> T;

    /// Gets the offset of the fragment.
    ///
    /// # Safety
    /// The fragment really has to be a fragment of buf.
    unsafe fn slice_offset(&self, fragment: T) -> usize;
}

mod bytes {
    use crate::UndoSlicing;
    use std::slice;

    impl<'s> UndoSlicing<&'s [u8]> for &'s [u8] {
        /// Undo taking a slice.
        ///
        /// # Safety
        ///
        /// The offset must be less than isize::MAX and should denote the offset
        /// of this span in the original.
        unsafe fn undo_slice(&self, offset: usize) -> &'s [u8] {
            assert!(offset < isize::MAX as usize);

            let ptr = self.as_ptr();
            let new_ptr = ptr.offset(-(offset as isize));

            slice::from_raw_parts(new_ptr, self.len() + offset)
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
        unsafe fn union_slice(&self, first: &'s [u8], second: &'s [u8]) -> &'s [u8] {
            unsafe {
                // fragment_offset checks for a negative offset.
                let offset_1 = self.slice_offset(first);
                assert!(offset_1 <= self.len());

                // fragment_offset checks for a negative offset.
                let offset_2 = self.slice_offset(second);
                assert!(offset_2 <= self.len());

                // correct ordering
                assert!(offset_1 <= offset_2);

                &self[offset_1..offset_2 + second.len()]
            }
        }

        /// Gets the offset of the fragment.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        unsafe fn slice_offset(&self, fragment: &'s [u8]) -> usize {
            let o = self.as_ptr();
            let f = fragment.as_ptr();

            let offset = f.offset_from(o);
            assert!(offset >= 0);

            offset as usize
        }
    }
}

mod strs {
    use crate::UndoSlicing;
    use std::slice;
    use std::str::from_utf8_unchecked;

    impl<'s> UndoSlicing<&'s str> for &'s str {
        /// Undo taking a slice.
        ///
        /// # Safety
        ///
        /// The offset must be less than isize::MAX and should denote the offset
        /// of this span in the original.
        unsafe fn undo_slice(&self, offset: usize) -> &'s str {
            assert!(offset < isize::MAX as usize);

            let ptr = self.as_ptr();
            let new_ptr = ptr.offset(-(offset as isize));

            let bytes = slice::from_raw_parts(new_ptr, self.len() + offset);
            from_utf8_unchecked(bytes)
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
        unsafe fn union_slice(&self, first: &'s str, second: &'s str) -> &'s str {
            unsafe {
                // fragment_offset checks for a negative offset.
                let offset_1 = self.slice_offset(first);
                assert!(offset_1 <= self.len());

                // fragment_offset checks for a negative offset.
                let offset_2 = self.slice_offset(second);
                assert!(offset_2 <= self.len());

                // correct ordering
                assert!(offset_1 <= offset_2);

                &self[offset_1..offset_2 + second.len()]
            }
        }

        /// Gets the offset of the fragment.
        ///
        /// # Safety
        /// The fragment really has to be a fragment of buf.
        unsafe fn slice_offset(&self, fragment: &'s str) -> usize {
            let o = self.as_ptr();
            let f = fragment.as_ptr();

            let offset = f.offset_from(o);
            assert!(offset >= 0);

            offset as usize
        }
    }
}
