---
type: is
id: is-01m2pye7bt83a2erfcav6hhkfc
title: "P1.4.6: Per-tier content provenance (TierProvenance.content)"
kind: task
status: open
priority: 0
version: 1
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies: []
parent_id: is-01m2pmra8yqrcxg27kc6ezg9vd
created_at: 2026-09-17T05:46:39.865Z
updated_at: 2026-09-17T05:46:39.865Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 4: Provenance and Tree Status", commit 6. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `TierProvenance { entries: TierState, content: Option<TierState> }` with `TierState { source, freshness, observed_at_ns }`, computed for the content tier from its store identity.

**Tests**

- Content freshness is stale under cache-only.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
