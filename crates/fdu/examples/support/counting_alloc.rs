//! A global allocator that tallies what it is asked to do.
//!
//! The allocator measured about 35% of the engine work in a cold scan's profile, which
//! made it the largest single line — and it took callgrind to see that, because nothing
//! in the engine could say how many allocations a run performed. This wrapper answers
//! that question directly: it forwards every request to the system allocator and counts
//! it on the way through.
//!
//! # Why this is affordable
//!
//! An allocation costs tens of nanoseconds even when it hits a hot free list. A
//! thread-local increment costs one or two. The counter is therefore a small fraction of
//! the operation it counts, which is the property that makes always-on defensible here
//! and would not hold for, say, counting every path component.
//!
//! # What it deliberately does not do
//!
//! It does not sample stacks, attribute allocations to call sites, or track live bytes.
//! Those need either a profiler or a shadow map, and both cost enough to change the
//! thing being measured. Counts and total bytes are what a paired A/B comparison needs:
//! they say whether a change moved allocation *volume*, which is exactly the question
//! H51, H62, H74 and H85 have all turned on.
//!
//! # Why this lives in the probe and not the library
//!
//! Counting allocations needs `unsafe impl GlobalAlloc`, and the workspace denies
//! unsafe code with exactly one exception — the platform FFI boundary in
//! `scan::macos_bulk`, which is there because the syscall cannot be reached otherwise.
//! This wrapper has no such excuse: it is measurement scaffolding, so it belongs in the
//! probe that measures rather than in the engine that ships. The library keeps its
//! unsafe-free guarantee, the shipped CLI carries no allocator wrapper, and the
//! visibility lands where the experiments run.
//!
//! Installing a global allocator is a binary's decision in any case; a library that made
//! it would take the choice away from every consumer.

use std::alloc::{GlobalAlloc, Layout, System};

use fdu::counters;

/// Wraps an allocator and counts every request.
///
/// Generic over the inner allocator because the `GlobalAlloc` impl below is, which
/// costs nothing and leaves the door open for counting mimalloc's traffic the day
/// `exp-047`'s −23% needs explaining as fewer operations or cheaper ones. There is
/// deliberately no constructor for that case until something calls it.
pub struct CountingAlloc<A> {
    inner: A,
}

impl CountingAlloc<System> {
    /// Count the system allocator, which is what every current build uses.
    pub const fn system() -> Self {
        Self { inner: System }
    }
}

// SAFETY: every method forwards to the inner allocator with its arguments unchanged and
// returns its pointer unchanged, so the safety contract is exactly the inner
// allocator's. The counting is a thread-local integer add that touches no allocator
// state and cannot unwind: `counters::bump` takes a closure that only performs
// arithmetic on a `Copy` struct. That last point matters more than it looks — a panic
// or a re-entrant allocation inside a `GlobalAlloc` method would be undefined behaviour,
// and the reason this wrapper counts rather than logs is that formatting would allocate.
unsafe impl<A: GlobalAlloc> GlobalAlloc for CountingAlloc<A> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        counters::bump(|c| {
            c.allocs += 1;
            c.bytes_allocated += layout.size() as u64;
        });
        // SAFETY: forwarding an unmodified layout to the inner allocator.
        unsafe { self.inner.alloc(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        counters::bump(|c| c.frees += 1);
        // SAFETY: forwarding an unmodified pointer and layout, which the caller has
        // already guaranteed came from this allocator.
        unsafe { self.inner.dealloc(ptr, layout) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        counters::bump(|c| {
            c.reallocs += 1;
            // Growth only: a shrink returns bytes rather than taking them, and folding
            // both into one total would make the number mean nothing.
            c.bytes_allocated += new_size.saturating_sub(layout.size()) as u64;
        });
        // SAFETY: forwarding unmodified arguments.
        unsafe { self.inner.realloc(ptr, layout, new_size) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        counters::bump(|c| {
            c.allocs += 1;
            c.bytes_allocated += layout.size() as u64;
        });
        // SAFETY: forwarding an unmodified layout.
        unsafe { self.inner.alloc_zeroed(layout) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wrapper must return exactly what the inner allocator returned, because every
    /// pointer in the process depends on it. Exercising it through a real allocation
    /// rather than a mock keeps that honest.
    #[test]
    fn forwards_allocations_intact() {
        let alloc = CountingAlloc::system();
        let layout = Layout::from_size_align(64, 8).expect("layout");

        // SAFETY: a 64-byte layout with 8-byte alignment is valid, the pointer is
        // checked before use, and it is freed exactly once with the same layout.
        unsafe {
            let ptr = alloc.alloc(layout);
            assert!(!ptr.is_null(), "allocation succeeded");
            ptr.write_bytes(0xAB, 64);
            assert_eq!(ptr.read(), 0xAB, "memory is usable after counting");
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn counts_what_it_forwards() {
        counters::reset();
        let alloc = CountingAlloc::system();
        let layout = Layout::from_size_align(128, 8).expect("layout");

        // SAFETY: as above; each pointer is freed once with its own layout.
        unsafe {
            let ptr = alloc.alloc(layout);
            assert!(!ptr.is_null());
            alloc.dealloc(ptr, layout);
        }

        let counts = counters::snapshot();
        if counters::enabled() {
            assert!(counts.allocs >= 1, "the allocation was counted");
            assert!(counts.frees >= 1, "the free was counted");
            assert!(counts.bytes_allocated >= 128, "sizes are counted: {counts:?}");
        }
        counters::reset();
    }
}
