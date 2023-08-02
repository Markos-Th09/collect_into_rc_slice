#![cfg(target_has_atomic = "ptr")]
use crate::arc::{data_offset, padding_needed, ArcInner};
use std::{
    alloc::{self, Layout},
    mem, ptr,
    sync::{atomic::AtomicUsize, Arc},
};

pub trait CollectIntoArcStr<T> {
    fn collect_into_arc_slice(self) -> Arc<[T]>;
}

impl<I, T> CollectIntoArcStr<T> for I
where
    I: Iterator<Item = T>,
{
    fn collect_into_arc_slice(self) -> Arc<[T]> {
        let metadata = data_offset::<T>();
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

        for item in self {
            if len + mem::size_of::<T>() > cap {
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
            len += mem::size_of::<T>();
        }

        // Trim the allocation down to `len`.
        if cap > len {
            // SAFETY:
            // - `len` is always non-zero.
            // - `align` is always a power of two and non-zero.
            // - The layout is padded to the alignment.
            let layout = Layout::from_size_align(len, align).unwrap().pad_to_align();
            alloc = unsafe { alloc::realloc(alloc, layout, len + padding_needed(len, align)) };

            if alloc.is_null() {
                alloc::handle_alloc_error(layout);
            }
        }

        // SAFETY: The allocation is non-null and has the proper layout.
        let data = unsafe {
            ptr::slice_from_raw_parts(
                alloc.add(metadata) as *mut T,
                (len - metadata) / mem::size_of::<T>(),
            )
        };

        // SAFETY:
        // - `data` is a valid pointer to a `[T]` located at the heap
        // - `data` is part of an RcBox with proper metadata.
        unsafe { Arc::from_raw(data) }
    }
}
