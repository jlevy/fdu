---
type: is
id: is-01m32f649ymyw0htpyrs3b1rhw
title: Measure the opened allocation slope on macOS and Windows, then assert the tightness rule
kind: bug
status: open
priority: 1
version: 4
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:10:59.134Z
updated_at: 2026-09-21T17:59:22.417Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---
Verified by execution, 2026-09-21.

`crates/fdu/tests/detached_performance_invariants.rs:104-110`. Measured on this Linux host: opened slope 24.33 against `OPENED_ALLOCATIONS_PER_ADDED_ENTRY = 26`.

Mutation: `std::hint::black_box(Box::new(0u8))` in `prepare_walk_entry` (`scan.rs:2941`), one extra heap allocation per admitted entry. Slope becomes 25.33 and the test PASSES. The ceiling needs at least +1.67 per entry to trip, but the file's own arithmetic self-checks (lines 70-77) are calibrated to a one-per-entry delta, so the test believes it is tighter than it is.

The detached counterpart is correctly calibrated: slope 6.15 against a ceiling of 7; the same mutation in `record_detached_entry` (`scan.rs:2896`) gives 7.15 and fails as intended.

Fix: derive the ceiling from the measured baseline plus a stated margin rather than a hand-set constant, and assert the margin is smaller than one allocation per entry so the self-check and the live bound agree. Same recipe as the H138 guard fix on PR #104: compare against a measured quantity, not a round number.

## Notes

The Linux half is fixed (PR #108): the ceiling is 25 against a measured 24.331, and the one-allocation-per-entry mutation now fails at 52,676 over 2,080 entries against a limit of 52,000. Reopened because the rule this implies is still only prose.

A senior review corrected an over-claim in the original fix. The remaining scope is narrower than "the other platforms are unknown":

- macOS 24.24 was MEASURED, at f9722505, and against its ceiling of 25 it is already tight at 0.76. Nothing is owed there beyond confirmation.
- Windows is the derived one: 34 comes from a predicted 33.43. Linux is the precedent for a derived ceiling drifting loose once the route got faster than the prediction, so the same hole may well be open on Windows.
- The measurement is available: CI runs this exact test on macos-latest and windows-latest (ci.yml:68, :94), so both slopes are an eprintln plus --nocapture away in a CI log.

Do: take both slopes from CI, tighten the Windows ceiling if it has drifted, then assert the rule in `assert_allocation_slope` — a ceiling must sit within one allocation per entry of a slope measured on its own platform, so `growth > limit - added_entries`. Six instrumented Linux runs give 24.314-24.333, a range of about 40 allocations against 1,391 of headroom, so the assertion will not flake there.

Not in scope: the `not(any(...))` fallback stays at 26. No CI platform reaches it, and tightening a ceiling on a platform nobody has measured would fail a port for the wrong reason.
