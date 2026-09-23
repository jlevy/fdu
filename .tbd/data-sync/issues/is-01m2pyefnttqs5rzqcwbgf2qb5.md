---
type: is
id: is-01m2pyefnttqs5rzqcwbgf2qb5
title: "P2.4.4: A projected load never overwrites the stronger snapshot; save_live guard"
kind: task
status: closed
priority: 0
version: 8
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyeg0zegnzm857bvxrbpa6
parent_id: is-01m2pmrcb8he4a8a54zt957vcs
hold: null
hold_until: null
created_at: 2026-09-17T05:46:48.377Z
updated_at: 2026-09-23T08:14:06.724Z
started_at: 2026-09-20T05:15:46.645Z
closed_at: 2026-09-23T08:14:06.724Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 4: The `.gitignore` Observation Projection on Every Route", commit 4. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `lib.rs`: a projected load does not overwrite the stronger snapshot.
- `crates/fdu/src/cli.rs`: the watch's initial report (`run_watch`, `:760-798`) warm-starts through `open_with_pending_save`; `save_live` (`:931-948`) skips writing while projected. If P2.3.6 has landed, the guard lives in `Session::persist_due`.

**Tests**

- `a_projected_open_leaves_the_stronger_snapshot_in_place`.
- `a_session_over_a_projected_index_refuses_an_ignored_selection`.

**Done when**

- Confirmed: a projected index's empty control table cannot later be saved claiming limits the request never had.
- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risk: a controls-off watch never persists over the stronger snapshot, so each run revalidates from an older one, a cost rather than a different answer.

## Notes

Implemented core projection and persistence safety in commit 0244b3bf. Added ProjectControlsOff and load_serving; projected parsing consumes and validates stored controls while constructing the requested blind scope; all open/report routes use it; report retagging was removed; projected metadata and watch saves cannot overwrite the stronger snapshot. Focused core projection, cold-equivalence, checksum/control validation, and stronger-cache preservation tests pass. Watch-enabled fdu compiles and its unit suite is 45/46 with the sole failure being the pre-existing schema-doc mismatch owned by the machine-output integration. Python smoke, CLI goldens/harness, durable docs, full make check, and integration CI remain for the root integration pass.\n\nFollow-up on merged report/7: the public Python open and cache-only routes now assert projected warm service; the path-independence harness compares request, status, and all report content while excluding only nested provenance; it includes a real cli-watch-initial route that reconstructs every JSONL section; malformed schema/status answers fail classification. The cache and engine architecture docs now describe the all-route projection, stronger-snapshot save guard, capture-gap watch startup, nested status/provenance fields, and reverse-direction cold/miss semantics. Focused parser/harness and snapshot identity/control-state tests pass. The actual projected CLI watch save-guard test passes when run outside the sandbox, where macOS FSEvents delivery is available.
