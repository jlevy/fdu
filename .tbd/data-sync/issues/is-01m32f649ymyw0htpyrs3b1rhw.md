---
type: is
id: is-01m32f649ymyw0htpyrs3b1rhw
title: Opened-route allocation ceiling tolerates the per-entry regression it exists to catch
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:10:59.134Z
updated_at: 2026-09-21T17:22:34.680Z
closed_at: 2026-09-21T17:22:34.680Z
close_reason: "Fixed in 784638be on claude/gate-integrity. Measured the slopes directly: opened 24.331/entry against a ceiling of 26 (1.669 headroom), detached 6.146 against 7 (0.854, correctly tight). Linux opened ceiling tightened to 25, leaving 0.669. The one-allocation-per-entry mutation in prepare_walk_entry now fails at 52,693 over 2,080 entries against a limit of 52,000. The implied rule -- a ceiling stays within one allocation per entry of a slope measured on its own platform -- is documented but deliberately not asserted: the macOS and Windows figures in that comment are demonstrably stale (Linux's implied 25.29 against an actual 24.331 is the proof), so asserting from unverifiable numbers would trade a silent hole for a red build on two platforms."
resolution: null
duplicate_of: null
---
Verified by execution, 2026-09-21.

`crates/fdu/tests/detached_performance_invariants.rs:104-110`. Measured on this Linux host: opened slope 24.33 against `OPENED_ALLOCATIONS_PER_ADDED_ENTRY = 26`.

Mutation: `std::hint::black_box(Box::new(0u8))` in `prepare_walk_entry` (`scan.rs:2941`), one extra heap allocation per admitted entry. Slope becomes 25.33 and the test PASSES. The ceiling needs at least +1.67 per entry to trip, but the file's own arithmetic self-checks (lines 70-77) are calibrated to a one-per-entry delta, so the test believes it is tighter than it is.

The detached counterpart is correctly calibrated: slope 6.15 against a ceiling of 7; the same mutation in `record_detached_entry` (`scan.rs:2896`) gives 7.15 and fails as intended.

Fix: derive the ceiling from the measured baseline plus a stated margin rather than a hand-set constant, and assert the margin is smaller than one allocation per entry so the self-check and the live bound agree. Same recipe as the H138 guard fix on PR #104: compare against a measured quantity, not a round number.
