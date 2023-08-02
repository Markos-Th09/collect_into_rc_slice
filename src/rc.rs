use std::alloc::Layout;
use std::mem;

#[repr(C)]
pub(crate) struct RcBox<T: ?Sized> {
    pub(crate) strong_count: usize,
    pub(crate) weak_count: usize,
    pub(crate) data: T,
}

pub(crate) fn data_offset<T>() -> usize {
    let layout = Layout::new::<RcBox<()>>();
    layout.size() + padding_needed(layout.size(), mem::align_of::<T>())
}

#[inline]
pub(crate) fn padding_needed(len: usize, align: usize) -> usize {
    // Math for computing padding is taken from `Layout::padding_needed_for`.
    let padding = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
    padding.wrapping_sub(len)
}
