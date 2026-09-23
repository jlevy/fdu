---
type: is
id: is-01m2pye6pak55wktt2d0ge1je2
title: "P1.4.4: One meaning for scan_started_at, read from the snapshot's verified_started_at_ns"
kind: task
status: in_progress
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye7bt83a2erfcav6hhkfc
parent_id: is-01m2pmra8yqrcxg27kc6ezg9vd
hold: null
hold_until: null
created_at: 2026-09-17T05:46:39.178Z
updated_at: 2026-09-23T01:19:29.580Z
started_at: 2026-09-20T04:40:17.353Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 4: Provenance and Tree Status", commit 4. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `index.rs`: stamp `verified_started_at_ns`.
- `scan_started_at` means the start of the oldest verification pass whose facts the answer serves; for a stale answer, the pass that wrote the snapshot, read from the format-5 header's `verified_started_at_ns` (P1.2.2).

**Tests**

- Cache-only `scan_started_at` is the writing pass's start.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
