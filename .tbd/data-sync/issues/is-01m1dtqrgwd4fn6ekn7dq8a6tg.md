---
type: is
id: is-01m1dtqrgwd4fn6ekn7dq8a6tg
title: Make detached application skip exact commit consequences
kind: feature
status: closed
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - design
dependencies:
  - type: blocks
    target: is-01m1dtqxh815zb3zz6m3g11cx6
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:33:06.331Z
updated_at: 2026-09-01T09:40:19.474Z
closed_at: 2026-09-01T09:40:19.473Z
close_reason: One shared reducer now selects a zero-cost detached consequence sink once per batch; the unreleased eager compatibility projection is removed, Rust/Python consumers use exact Commit data, exp-077/078 record the allocation and timing evidence, and the complete make check handoff gate passes at 2d7a566.
resolution: null
duplicate_of: null
---
Introduce one private batch-level consequence boundary shared by the reducer. Detached baseline construction retains facts, roll-ups, scope, issues, provenance, and ApplyStats but allocates no effective-change paths, Impact, Commit, journal entry, or AppliedDelta. Remove the unreleased eager compatibility projection and prove detached and exact fact digests agree.

## Notes

Implemented in da5b8bc and db18e5e. A batch-selected zero-sized NoConsequences sink shares one generic reducer with ExactConsequences; detached baseline construction now retains facts, roll-ups, scope, issues, provenance, and ApplyStats but produces no effective paths, Impact, Commit, or journal work. exp-077 records default-tree wall -6.57%, component -7.24%, scoped allocations -33.7%, and allocated bytes -24.5%, with exact streaming digests unchanged. Removed the unreleased AppliedDelta projection and migrated Rust/Python consumers to Commit; exp-078 records 100,002 fewer scoped allocations for the 100,001-op exact batch, wall -1.55% CI [-2.46%, -1.30%], component -2.38%, and RSS -7.09%. Minimal/all-feature core suites, independent model, performance harness tests, Python lint/types/tests, wheel smoke, and exact job oracles pass. Awaiting repository-wide handoff gate before closure.
