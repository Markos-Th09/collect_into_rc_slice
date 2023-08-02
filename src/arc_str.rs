#![cfg(target_has_atomic = "ptr")]
use crate::arc::{data_offset, padding_needed, ArcInner};
use std::{
    alloc::{self, Layout},
    mem, ptr, slice,
    sync::{atomic::AtomicUsize, Arc},
};

pub trait CollectIntoArcStr {
    /// Collects the iterator into an `Arc<str>`.
    ///
    /// ## Important Note
    /// Please *DO NOT* use this if you already have a `String` or `&str` that contains the exact block memory you are trying convert to `Arc<[T]>`.
    /// It wouldn't do anything better than the `std` implementation. It always better to use `.into()` in this case.
    ///
    /// # Examples
    /// ```rust
    /// use std::sync::Arc;
    /// use collect_into_rc_slice::*;
    ///
    /// let s: Arc<str> = "Hello, world!".chars().collect_into_arc_str();
    ///
    /// assert!(s.as_ref() == "Hello, world!");
    /// ```
    fn collect_into_arc_str(self) -> Arc<str>;
}

impl<T> CollectIntoArcStr for T
where
    T: Iterator<Item = char>,
{
    fn collect_into_arc_str(self) -> Arc<str> {
        let metadata = data_offset::<u8>();
        let align = mem::align_of::<ArcInner<()>>();

        // the size should be at least metadata
        // but if bounds are known, it should be at least largest_known_bound+metadata
        let (lower_bound, upper_bound) = self.size_hint();
        let mut len = metadata;
        let mut cap = upper_bound.unwrap_or(lower_bound) + metadata;

        // SAFETY:
        // - `len` is always greater than or equal to `metadata` which is non-zero.
        // - `align` is always a power of two and non-zero.
        // - The layout is padded to the alignment.
        let layout = Layout::from_size_align(cap, align).unwrap().pad_to_align();
        let mut alloc = unsafe { alloc::alloc(layout) };

        if alloc.is_null() {
            alloc::handle_alloc_error(layout);
        }

        // SAFETY: The metadata part is not meant to be valid UTF-8 data, so it's safe to
        // initialize it with arbitrary data.
        unsafe {
            let init: *const u8 = &ArcInner {
                strong: AtomicUsize::new(1),
                weak: AtomicUsize::new(1),
                data: (),
            } as *const _ as *const u8;

            ptr::copy_nonoverlapping(init, alloc, metadata);
        }

        for c in self {
            let new_len = len + c.len_utf8();
            if new_len > cap {
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
                let ptr = alloc.add(len);
                len = new_len;
                c.encode_utf8(slice::from_raw_parts_mut(ptr, c.len_utf8()));
            }
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

        let data =
            unsafe { ptr::slice_from_raw_parts(alloc.add(metadata), len - metadata) as *const str };

        // SAFETY:
        // - `data` is a valid pointer to a `str` located at the heap
        // - `data` is part of an RcBox with proper metadata.
        unsafe { Arc::from_raw(data) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_into_arc_str() {
        let s = "Hello, world!".chars().collect_into_arc_str();

        assert!(s.as_ref() == "Hello, world!");
        assert_eq!(s.len(), 13);
        assert_eq!(Arc::strong_count(&s), 1);
        assert_eq!(Arc::weak_count(&s), 0);
    }

    #[test]
    fn test_collect_into_arc_str_ref() {
        let s = ['a', 'b', 'c'].iter().copied().collect_into_arc_str();

        assert!(s.as_ref() == "abc");
        assert_eq!(s.len(), 3);
        assert_eq!(Arc::strong_count(&s), 1);
        assert_eq!(Arc::weak_count(&s), 0);
    }

    #[test]
    fn test_collect_into_arc_str_unknown_size() {
        let mut str = "Hello, world!".chars();
        let s = std::iter::from_fn(move || str.next()).collect_into_arc_str();

        assert!(s.as_ref() == "Hello, world!");
        assert_eq!(s.len(), 13);
        assert_eq!(Arc::strong_count(&s), 1);
        assert_eq!(Arc::weak_count(&s), 0);
    }
}
