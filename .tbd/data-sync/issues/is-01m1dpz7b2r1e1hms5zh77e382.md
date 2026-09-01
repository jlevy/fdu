---
type: is
id: is-01m1dpz7b2r1e1hms5zh77e382
title: "Review PR #51 streaming performance regression and design"
kind: task
status: closed
priority: 1
version: 4
labels: []
dependencies: []
created_at: 2026-09-01T05:27:16.577Z
updated_at: 2026-09-01T06:17:58.559Z
closed_at: 2026-09-01T06:17:58.558Z
close_reason: "Completed design, correctness, performance-attribution, and validation review of PR #51."
resolution: null
duplicate_of: null
---

## Notes

Reviewed PR #51 at e8f1bed against origin/main b75bf85. Verdict: do not merge yet. Exploratory exact-tree interleaved measurements on ~/.rustup/toolchains (119,368 entries, uncontrolled busy host) put head at 847.7 ms wall / 628.0 ms component versus main 349.9 / 132.4; head halves PR-base cost but remains 2.4x wall and 4.7x component. Public 100,001-op delta apply was 432.5 ms versus main 56.0 ms. Allocation ladder: main 1,459,192; head 2,716,389; skip eager AppliedDelta 2,604,110; skip publish bookkeeping 2,127,817; skip ancestry overlay 1,830,432; owned observation 1,710,339; skip effect-path copy 1,590,362. Timing attribution shows StructuralOverlay ancestry preflight is the dominant CPU cost; impact publication is second; prepare/effect/delta copies dominate residual allocations. Found correctness issues: controls-on snapshot is accepted for controls-off public open but Auto later errors at exact reconcile and Only returns controls-on state; canonical_relative_path fast lane preserves encoded a/./b and a//b because Components hides redundant syntax. Full make check and make cross-lint pass in pristine exact-head worktree; 19 GitHub checks pass. Recommended lifecycle-aware effect sinks, trusted prepared scanner path with moved ownership and no transactional ancestry overlay, lazy/no AppliedDelta for baseline/internal paths, and clone-after-capacity journal retention. Add allocation guards and focused scope/canonicalization tests. No PR experiments were recorded in the required ledger.
