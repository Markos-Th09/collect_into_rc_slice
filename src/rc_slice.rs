#[cfg(feature = "std")]
use std::alloc;
#[cfg(feature = "std")]
use std::rc::Rc;
#[cfg(not(feature = "std"))]
extern crate alloc;
use crate::{data_offset, padding_needed, RcBox};
#[cfg(not(feature = "std"))]
use alloc::rc::Rc;
use core::alloc::Layout;
use core::mem;
use core::ptr;

pub trait CollectIntoRcSlice<T> {
    /// Collects the iterator into an `Rc<[T]>`.
    ///
    /// ## Important Note
    /// Please *DO NOT* use this if you already have a `Vec<T>`, `String` or `&[T]` that contains the exact block memory you are trying convert to `Rc<[T]>`.
    /// It wouldn't do anything better than the `std` implementation. It always better to use `.into()` in this case.
    ///
    /// # Examples
    /// ```rust
    /// use std::rc::Rc;
    /// use collect_into_rc_slice::*;
    ///
    /// let a = [1, 2, 3, 4, 5];
    /// let rc: Rc<[i32]> = a.into_iter().collect_into_rc_slice();
    ///
    /// assert_eq!(&*rc, &[1, 2, 3, 4, 5]);
    /// ```
    fn collect_into_rc_slice(self) -> Rc<[T]>;
}

impl<T, I> CollectIntoRcSlice<T> for I
where
    I: Iterator<Item = T>,
{
    fn collect_into_rc_slice(self) -> Rc<[T]> {
        let metadata = data_offset::<T>();
        let align = mem::align_of::<RcBox<()>>();

        // the size should be at least metadata
        // but if bounds are known, it should be at least largest_known_bound + metadata
        let (lower_bound, upper_bound) = self.size_hint();
        let mut len = metadata;
        let mut cap = upper_bound.unwrap_or(lower_bound) * mem::size_of::<T>() + metadata;
        let size = mem::size_of::<T>();

        // SAFETY:
        // - `len` is always greater than or equal to `metadata` which is non-zero.
        // - `align` is always a power of two and non-zero.
        // - The layout is padded to the alignment.
        let layout = Layout::from_size_align(cap, align).unwrap().pad_to_align();
        let mut alloc = unsafe { alloc::alloc(layout) };

        if alloc.is_null() {
            alloc::handle_alloc_error(layout);
        }

        // SAFETY: The metadata part is not meant to be valid data, so it's safe to
        // initialize it with arbitrary data.
        unsafe {
            let init: *const u8 = &RcBox {
                strong_count: 1,
                weak_count: 1,
                data: (),
            } as *const _ as *const u8;

            ptr::copy_nonoverlapping(init, alloc, metadata);
        }

        for item in self {
            if len + size > cap {
                // SAFETY:
                // - `size` is always non-zero.
                // - `align` is always a power of two and non-zero.
                // - The layout is padded to the alignment.
                let layout = Layout::from_size_align(cap, align).unwrap().pad_to_align();
                cap *= 2;
                alloc = unsafe { alloc::realloc(alloc, layout, cap) };

                if alloc.is_null() {
                    alloc::handle_alloc_error(layout);
                }
            }

            unsafe {
                ptr::write(alloc.add(len) as *mut T, item);
            }
            len += size;
        }

        // Trim the allocation down to `len`.
        if cap > len {
            // SAFETY:
            // - `cap` is always non-zero.
            // - `align` is always a power of two and non-zero.
            // - The layout is padded to the alignment.
            let layout = Layout::from_size_align(cap, align).unwrap().pad_to_align();
            alloc = unsafe { alloc::realloc(alloc, layout, len + padding_needed(len, align)) };

            if alloc.is_null() {
                alloc::handle_alloc_error(layout);
            }
        }

        let data = unsafe {
            ptr::slice_from_raw_parts(alloc.add(metadata) as *mut T, (len - metadata) / size)
        };

        // SAFETY:
        // - `data` is a valid pointer to a `[T]` located at the heap
        // - `data` is part of an RcBox with proper metadata.
        unsafe { Rc::from_raw(data) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc_slice() {
        let v = vec![1, 2, 3, 4, 5];
        let rc = v.into_iter().collect_into_rc_slice();
        assert_eq!(&*rc, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_rc_slice2() {
        let v = vec![[0u8; 7]];
        let rc = v.into_iter().collect_into_rc_slice();
        assert_eq!(&*rc, &[[0; 7]]);
    }
}
