---
type: is
id: is-01m2pyeeppbcbq3hfepwjcthyw
title: "P2.4.1: ProjectControlsOff in serves_snapshot, and snapshot::load_serving"
kind: task
status: in_progress
priority: 0
version: 5
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
updated_at: 2026-09-20T06:58:47.335Z
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

## Notes

Implemented core projection and persistence safety in commit 0244b3bf. Added ProjectControlsOff and load_serving; projected parsing consumes and validates stored controls while constructing the requested blind scope; all open/report routes use it; report retagging was removed; projected metadata and watch saves cannot overwrite the stronger snapshot. Focused core projection, cold-equivalence, checksum/control validation, and stronger-cache preservation tests pass. Watch-enabled fdu compiles and its unit suite is 45/46 with the sole failure being the pre-existing schema-doc mismatch owned by the machine-output integration. Python smoke, CLI goldens/harness, durable docs, full make check, and integration CI remain for the root integration pass.\n\nFollow-up on merged report/7: the public Python open and cache-only routes now assert projected warm service; the path-independence harness compares request, status, and all report content while excluding only nested provenance; it includes a real cli-watch-initial route that reconstructs every JSONL section; malformed schema/status answers fail classification. The cache and engine architecture docs now describe the all-route projection, stronger-snapshot save guard, capture-gap watch startup, and nested status/provenance fields. Focused parser/harness and snapshot identity/control-state tests pass. The actual projected CLI watch save-guard test is added; on this macOS host it reaches the initial report but the native observer does not emit the warm-up event within 30 seconds, and an existing watch_controls test independently has the same timeout, so integration must verify it on the full platform gate.
