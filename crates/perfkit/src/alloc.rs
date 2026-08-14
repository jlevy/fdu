//! A global allocator wrapper that counts what it is asked to do.
//!
//! Allocation is routinely the largest single line in a systems profile and the one
//! least visible without a profiler — in the walker this crate was extracted from, it
//! was about 35% of a cold scan's engine work, and it took callgrind to see that. A
//! counting wrapper answers the same question on an ordinary build.
//!
//! # Why counting an allocation is affordable
//!
//! An allocation costs tens of nanoseconds even when it hits a hot free list; a
//! thread-local increment costs one or two. The counter is a small fraction of the
//! operation it counts. That ratio is the whole argument, and it would not hold for
//! counting something already cheap.
//!
//! # Runtime, not build-time
//!
//! The wrapper is always installed and checks [`crate::is_enabled`] on each call, so one
//! binary can be asked for allocation numbers instead of two that differ in whether they
//! have any. If a build ever needs the branch gone as well, gate the
//! `#[global_allocator]` statement itself — but measure first, because the branch is
//! predictable and the allocation is not.
//!
//! # What it deliberately does not do
//!
//! No stack sampling, no call-site attribution, no live-byte tracking. Each needs a
//! profiler or a shadow map, and both cost enough to change what they measure. Counts
//! and totals are what a paired A/B needs: they answer whether a change moved allocation
//! *volume*, which is the question most allocator hypotheses turn on.
//!
//! ```no_run
//! perfkit::counters! {
//!     pub mod mem { allocs: "allocations", "memory"; frees: "frees", "memory";
//!                   reallocs: "reallocations", "memory"; bytes: "bytes allocated", "memory"; }
//! }
//!
//! #[global_allocator]
//! static ALLOC: perfkit::alloc::CountingAlloc<std::alloc::System> =
//!     perfkit::alloc::CountingAlloc::system(perfkit::alloc::Sinks {
//!         alloc: |size| mem::bump(|c| { c.allocs += 1; c.bytes += size }),
//!         realloc: |growth| mem::bump(|c| { c.reallocs += 1; c.bytes += growth }),
//!         dealloc: || mem::bump(|c| c.frees += 1),
//!     });
//! ```

use std::alloc::{GlobalAlloc, Layout, System};

/// Where a [`CountingAlloc`] reports what it did.
///
/// Function pointers rather than a trait so the whole thing stays usable in a `const`
/// initializer, which a `#[global_allocator]` static requires. They also keep this crate
/// from owning the counter names: the application declares those with
/// [`counters!`](crate::counters) and wires them up here.
#[derive(Clone, Copy)]
pub struct Sinks {
    /// Called with the requested size on every allocation.
    pub alloc: fn(u64),
    /// Called with the *growth* on every reallocation. A shrink reports zero, because
    /// folding growth and shrink into one total makes the number mean nothing.
    pub realloc: fn(u64),
    /// Called on every free.
    pub dealloc: fn(),
}

impl Sinks {
    /// Sinks that discard everything, for a binary that wants the wrapper installed but
    /// has no counters wired up yet.
    #[must_use]
    pub const fn noop() -> Self {
        Self { alloc: |_| {}, realloc: |_| {}, dealloc: || {} }
    }
}

/// An allocator that forwards every request and counts it on the way through.
pub struct CountingAlloc<A> {
    inner: A,
    sinks: Sinks,
}

impl CountingAlloc<System> {
    /// Count the system allocator.
    #[must_use]
    pub const fn system(sinks: Sinks) -> Self {
        Self { inner: System, sinks }
    }
}

impl<A> CountingAlloc<A> {
    /// Count some other allocator — a bump arena, mimalloc, whatever is being trialled.
    #[must_use]
    pub const fn wrapping(inner: A, sinks: Sinks) -> Self {
        Self { inner, sinks }
    }
}

// SAFETY: every method forwards its arguments to the inner allocator unchanged and
// returns its pointer unchanged, so this type's safety contract is exactly the inner
// allocator's.
//
// The counting is the part that needs justifying, because a `GlobalAlloc` method that
// unwinds or re-enters the allocator is undefined behaviour. Both are ruled out by
// construction: the sinks are plain function pointers that do integer arithmetic on a
// thread-local `Copy` struct behind a relaxed atomic check. Nothing formats, nothing
// locks, nothing allocates — which is exactly why this counts rather than logs.
unsafe impl<A: GlobalAlloc> GlobalAlloc for CountingAlloc<A> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (self.sinks.alloc)(layout.size() as u64);
        // SAFETY: forwarding an unmodified layout to the inner allocator.
        unsafe { self.inner.alloc(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        (self.sinks.dealloc)();
        // SAFETY: forwarding an unmodified pointer and layout, which the caller has
        // already guaranteed came from this allocator.
        unsafe { self.inner.dealloc(ptr, layout) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        (self.sinks.realloc)(new_size.saturating_sub(layout.size()) as u64);
        // SAFETY: forwarding unmodified arguments.
        unsafe { self.inner.realloc(ptr, layout, new_size) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        (self.sinks.alloc)(layout.size() as u64);
        // SAFETY: forwarding an unmodified layout.
        unsafe { self.inner.alloc_zeroed(layout) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static ALLOCS: AtomicU64 = AtomicU64::new(0);
    static BYTES: AtomicU64 = AtomicU64::new(0);
    static FREES: AtomicU64 = AtomicU64::new(0);

    fn test_sinks() -> Sinks {
        Sinks {
            alloc: |size| {
                ALLOCS.fetch_add(1, Ordering::Relaxed);
                BYTES.fetch_add(size, Ordering::Relaxed);
            },
            realloc: |growth| {
                BYTES.fetch_add(growth, Ordering::Relaxed);
            },
            dealloc: || {
                FREES.fetch_add(1, Ordering::Relaxed);
            },
        }
    }

    /// The wrapper must hand back exactly what the inner allocator returned, because
    /// every pointer in the process depends on it. Exercised against the real system
    /// allocator rather than a mock.
    #[test]
    fn forwards_memory_intact_and_counts_it() {
        let _serial = crate::test_serial();
        let alloc = CountingAlloc::system(test_sinks());
        let layout = Layout::from_size_align(64, 8).expect("valid layout");
        ALLOCS.store(0, Ordering::Relaxed);
        FREES.store(0, Ordering::Relaxed);
        BYTES.store(0, Ordering::Relaxed);

        // SAFETY: a 64-byte, 8-aligned layout is valid; the pointer is checked before
        // use, written within its length, and freed exactly once with the same layout.
        unsafe {
            let ptr = alloc.alloc(layout);
            assert!(!ptr.is_null(), "allocation succeeded");
            ptr.write_bytes(0xAB, 64);
            assert_eq!(ptr.read(), 0xAB, "memory is usable after counting");
            alloc.dealloc(ptr, layout);
        }

        assert_eq!(ALLOCS.load(Ordering::Relaxed), 1);
        assert_eq!(FREES.load(Ordering::Relaxed), 1);
        assert_eq!(BYTES.load(Ordering::Relaxed), 64);
    }

    #[test]
    fn a_shrinking_realloc_reports_no_growth() {
        let _serial = crate::test_serial();
        let alloc = CountingAlloc::system(test_sinks());
        let layout = Layout::from_size_align(128, 8).expect("valid layout");
        BYTES.store(0, Ordering::Relaxed);

        // SAFETY: the pointer comes from this allocator with this layout, is reallocated
        // once to a smaller size, and the result is freed once with the new layout.
        unsafe {
            let ptr = alloc.alloc(layout);
            assert!(!ptr.is_null());
            BYTES.store(0, Ordering::Relaxed);
            let smaller = alloc.realloc(ptr, layout, 32);
            assert!(!smaller.is_null());
            alloc.dealloc(smaller, Layout::from_size_align(32, 8).expect("valid layout"));
        }

        assert_eq!(BYTES.load(Ordering::Relaxed), 0, "a shrink adds no allocated bytes");
    }

    #[test]
    fn noop_sinks_are_usable_in_a_const() {
        const ALLOC: CountingAlloc<System> = CountingAlloc::system(Sinks::noop());
        let layout = Layout::from_size_align(16, 8).expect("valid layout");
        // SAFETY: valid layout; allocated once, freed once with the same layout.
        unsafe {
            let ptr = ALLOC.alloc(layout);
            assert!(!ptr.is_null());
            ALLOC.dealloc(ptr, layout);
        }
    }
}
