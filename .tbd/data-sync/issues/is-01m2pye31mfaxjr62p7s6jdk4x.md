---
type: is
id: is-01m2pye31mfaxjr62p7s6jdk4x
title: "P1.2.5: Per-tier write rules for snapshot and sidecar"
kind: task
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye3bwkv6049gkcyh3rrcp
  - type: blocks
    target: is-01m2pye80k4gebgn994ewchs53
  - type: blocks
    target: is-01m2pyec35d01hhcj2n349he3v
parent_id: is-01m2pmram44dgp78vm6xq4w7k7
created_at: 2026-09-17T05:46:35.443Z
updated_at: 2026-09-17T05:46:58.653Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 2: Store Identity, Equality Serve, and Per-Tier Writes", commit 5. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `snapshot.rs`: the save guard becomes `entries_writable`.
- `content/content_cache.rs`: the save filter (`:71-81`) becomes `content_record_writable`.
- `lib.rs`: `SaveTargets` (`:636-653`), `cold_scan_save_targets_with` (`:699-717`), `spawn_save` (`:730-777`): drop the joint completeness gate (`:736-737`) for per-tier rules; write the sidecar after a partial scan only when a snapshot of the same entry identity is already stored, so a partial run under another identity never evicts the sidecar that pairs with the stored snapshot.
- Call site: `save_content_cache` at `lib.rs:768`.

**Tests**

- `lib.rs`: add `a_partial_scan_writes_verified_content_but_no_snapshot`.
- `content/content_cache.rs`: add `records_under_an_unverified_subtree_are_not_written`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
