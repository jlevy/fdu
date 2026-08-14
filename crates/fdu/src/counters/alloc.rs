//! A global allocator wrapper with certified, non-reentrant counter sinks.

#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};

/// Callbacks used by [`CountingAlloc`].
///
/// Fields are private so safe code cannot construct a sink that violates the global
/// allocator contract. Use [`Self::noop`] or the unsafe [`Self::new`] constructor.
#[derive(Clone, Copy)]
pub struct Sinks {
    alloc: fn(u64),
    realloc: fn(u64),
    dealloc: fn(),
}

impl Sinks {
    /// Construct callbacks that discard every event.
    #[must_use]
    pub const fn noop() -> Self {
        Self { alloc: |_| {}, realloc: |_| {}, dealloc: || {} }
    }

    /// Construct a certified sink set.
    ///
    /// # Safety
    ///
    /// Every callback must remain valid for the program's lifetime and must never
    /// unwind, allocate, deallocate, acquire a lock, or otherwise re-enter the global
    /// allocator. It must also remain callable during thread-local destruction. All
    /// arithmetic in a callback must be non-panicking in every build profile.
    ///
    /// ```
    /// fn allocation(_size: u64) {}
    /// fn reallocation(_growth: u64) {}
    /// fn deallocation() {}
    ///
    /// // SAFETY: these empty callbacks cannot unwind, allocate, lock, or re-enter.
    /// let sinks = unsafe {
    ///     fdu::counters::alloc::Sinks::new(allocation, reallocation, deallocation)
    /// };
    /// let _allocator =
    ///     fdu::counters::alloc::CountingAlloc::system(sinks);
    /// ```
    #[must_use]
    pub const unsafe fn new(alloc: fn(u64), realloc: fn(u64), dealloc: fn()) -> Self {
        Self { alloc, realloc, dealloc }
    }
}

/// An allocator that forwards every request unchanged and reports it to certified sinks.
pub struct CountingAlloc<A> {
    inner: A,
    sinks: Sinks,
}

impl CountingAlloc<System> {
    /// Wrap the system allocator with an already-certified sink set.
    #[must_use]
    pub const fn system(sinks: Sinks) -> Self {
        Self { inner: System, sinks }
    }
}

impl<A> CountingAlloc<A> {
    /// Wrap another allocator with an already-certified sink set.
    #[must_use]
    pub const fn wrapping(inner: A, sinks: Sinks) -> Self {
        Self { inner, sinks }
    }
}

/// fdu's sink set, whose callbacks use only guarded TLS cells and relaxed atomics.
pub(super) const fn fdu_sinks() -> Sinks {
    // SAFETY: these private functions perform only non-panicking saturating arithmetic,
    // guarded const TLS access, and relaxed atomic updates. They never format, allocate,
    // deallocate, lock, or call the allocator.
    unsafe { Sinks::new(super::record_alloc, super::record_realloc, super::record_dealloc) }
}

// SAFETY: every request is forwarded to the inner allocator with identical arguments
// and its pointer is returned unchanged. `Sinks` can contain non-noop callbacks only
// after an unsafe construction that certifies the allocator-specific requirements.
unsafe impl<A: GlobalAlloc> GlobalAlloc for CountingAlloc<A> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (self.sinks.alloc)(u64::try_from(layout.size()).unwrap_or(u64::MAX));
        // SAFETY: the caller supplied a valid layout; it is forwarded unchanged.
        unsafe { self.inner.alloc(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        (self.sinks.dealloc)();
        // SAFETY: the caller guarantees this pointer came from the allocator with this
        // layout; both are forwarded unchanged.
        unsafe { self.inner.dealloc(ptr, layout) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let growth = u64::try_from(new_size.saturating_sub(layout.size())).unwrap_or(u64::MAX);
        (self.sinks.realloc)(growth);
        // SAFETY: the caller's pointer, layout, and new size are forwarded unchanged.
        unsafe { self.inner.realloc(ptr, layout, new_size) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        (self.sinks.alloc)(u64::try_from(layout.size()).unwrap_or(u64::MAX));
        // SAFETY: the caller supplied a valid layout; it is forwarded unchanged.
        unsafe { self.inner.alloc_zeroed(layout) }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static ALLOCS: AtomicU64 = AtomicU64::new(0);
    static BYTES: AtomicU64 = AtomicU64::new(0);
    static FREES: AtomicU64 = AtomicU64::new(0);

    fn alloc(size: u64) {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(size, Ordering::Relaxed);
    }

    fn realloc(growth: u64) {
        BYTES.fetch_add(growth, Ordering::Relaxed);
    }

    fn dealloc() {
        FREES.fetch_add(1, Ordering::Relaxed);
    }

    fn test_sinks() -> Sinks {
        // SAFETY: the test callbacks above use relaxed atomics only and cannot unwind,
        // allocate, lock, or re-enter the allocator.
        unsafe { Sinks::new(alloc, realloc, dealloc) }
    }

    #[test]
    fn forwards_memory_intact_and_counts_it() {
        let _serial = crate::counters::test_serial();
        let allocator = CountingAlloc::system(test_sinks());
        let layout = Layout::from_size_align(64, 8).expect("valid layout");
        ALLOCS.store(0, Ordering::Relaxed);
        FREES.store(0, Ordering::Relaxed);
        BYTES.store(0, Ordering::Relaxed);

        // SAFETY: the valid allocation is checked, used within its bounds, and freed once
        // with the same allocator and layout.
        unsafe {
            let pointer = allocator.alloc(layout);
            assert!(!pointer.is_null());
            pointer.write_bytes(0xAB, 64);
            assert_eq!(pointer.read(), 0xAB);
            allocator.dealloc(pointer, layout);
        }

        assert_eq!(ALLOCS.load(Ordering::Relaxed), 1);
        assert_eq!(FREES.load(Ordering::Relaxed), 1);
        assert_eq!(BYTES.load(Ordering::Relaxed), 64);
    }

    #[test]
    fn noop_sinks_are_const_constructible() {
        const ALLOCATOR: CountingAlloc<System> = CountingAlloc::system(Sinks::noop());
        let layout = Layout::from_size_align(16, 8).expect("valid layout");
        // SAFETY: the valid allocation is freed once with the same allocator and layout.
        unsafe {
            let pointer = ALLOCATOR.alloc(layout);
            assert!(!pointer.is_null());
            ALLOCATOR.dealloc(pointer, layout);
        }
    }
}
