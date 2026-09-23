---
type: is
id: is-01m2pyecqwam3e928fc2jn2xjm
title: "P2.3.3: Plan::writes as the only write decision"
kind: task
status: in_progress
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyed27kmqd4z20e9kqddh1
parent_id: is-01m2pmrbxvnerxyhnjhrswy1ye
created_at: 2026-09-17T05:46:45.371Z
updated_at: 2026-09-23T01:47:40.802Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 3: The Execution Plan Model", commit 3. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `lib.rs`: `SaveTargets` and warm targets (`:579-586`, `:636-653`), `SNAPSHOT_MIN_ENTRIES` (`:687`), and `cold_scan_save_targets` (`:693-717`) are replaced by `Plan::writes(&RunFacts) -> SaveTargets`; `load_content` (`:719`) and `spawn_save` (`:730-777`) become pure executors.

**Tests**

- For every `Delivery::enumerate()` value and route, `writes` is identical for the same run facts.
- Content is written after a partial scan when verified and a snapshot of its entry identity exists; entries never.
- Turn `save_tests` (`lib.rs:1623`) and `cold_scan_persistence_tests` (`:1730`) into `Plan::writes` tests.

**Done when**

- Lands only with the path-independence subset passing.
- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
