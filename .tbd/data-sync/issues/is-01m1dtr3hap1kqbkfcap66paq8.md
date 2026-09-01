---
type: is
id: is-01m1dtr3hap1kqbkfcap66paq8
title: Remove duplicate path ownership from exact impact and journal publication
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - design
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:33:17.609Z
updated_at: 2026-09-01T11:13:08.945Z
closed_at: 2026-09-01T11:13:08.944Z
close_reason: Accepted the journal capacity preflight in e2ac4f9 after a 3.46% large exact-update wall improvement and unchanged semantic digests; profiled, recorded, and removed two ownership spikes that reduced allocations without supported elapsed-time gains. Shared Commit ownership was not justified.
resolution: null
duplicate_of: null
---
Move scanner-owned paths once, accumulate impact with flags and bounded IDs rather than a full PathBuf set, stop at all_dirty, and compute journal retention cost before cloning. Introduce shared Commit ownership only if a post-cleanup profile still names retained cloning.

## Notes

Fresh post-proof counters and sampling separated exact publication mechanisms from allocation volume. Accepted exp-080 at e2ac4f9: journal capacity is checked before cloning; delta-apply-large wall time improved 3.46% (paired 95% CI -4.32% to -2.61%), 100,003 scoped allocations disappeared, exact digests were unchanged, and opened discovery remained noninferior. Rejected and removed exp-081: borrowing impact paths cut opened allocations 8.2% but produced no supported opened timing gain and regressed the large exact component. Rejected and removed exp-082: moving scanner commits into the journal removed about 10.3% of opened allocations and nearly all scanner journal clones but changed opened wall time -0.01% (CI -2.15% to +5.77%) while adding a second result form. No shared Commit ownership was introduced. The retained-clone mechanism is not a measured wall-time target; StructuralOverlay remains an exact public-mutation correctness boundary and is not paid by detached or scanner discovery.
