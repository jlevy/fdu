---
type: is
id: is-01m2pyeeppbcbq3hfepwjcthyw
title: "P2.4.1: ProjectControlsOff in serves_snapshot, and snapshot::load_serving"
kind: task
status: in_progress
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyef13ts4hgtt92bfeftfb
parent_id: is-01m2pmrcb8he4a8a54zt957vcs
hold: null
hold_until: null
created_at: 2026-09-17T05:46:47.381Z
updated_at: 2026-09-20T05:15:46.598Z
started_at: 2026-09-20T05:15:46.596Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 4: The `.gitignore` Observation Projection on Every Route", commit 1. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `stored_state.rs`: extend `serves_snapshot` with `ProjectControlsOff`; it takes no policy and no consumer.
- `snapshot.rs`: add `load_serving(path, types, wanted) -> LoadOutcome::{Served(Index, Serves), Refused(SnapshotIdentity), Absent}`; in `parse_stream` (`:614-708`) a projection builds the index with the requested scope and skips installing the control section (`:699-700`).

**Tests**

- `snapshot_serving_is_equality_plus_observation_on_to_off`; `a_projected_load_equals_a_controls_off_scan`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
