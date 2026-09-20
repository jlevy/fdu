---
type: is
id: is-01m2pyeg0zegnzm857bvxrbpa6
title: "P2.4.5: Python projection check, cli-cache golden, cli-watch-initial harness route, and docs"
kind: task
status: in_progress
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies: []
parent_id: is-01m2pmrcb8he4a8a54zt957vcs
hold: null
hold_until: null
created_at: 2026-09-17T05:46:48.734Z
updated_at: 2026-09-20T07:00:58.899Z
started_at: 2026-09-20T05:15:46.659Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 4: The `.gitignore` Observation Projection on Every Route", commit 5. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- Python smoke check: a controls-off open answers from a default snapshot.
- A `cli-cache` golden answering `warm_revalidate`.
- The `cli-watch-initial` harness route; clear `projection-route` after a full Linux run.
- Documentation rewrites: `lib.rs:174-183`, `:331-336`; `execution.rs:198-206`; `crates/fdu-py/python/fdu/_api.py:367-374`; `cli-cache.tryscript.md:253-257`; the cache design's Known Gaps for projection.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

## Notes

Implemented core projection and persistence safety in commit 0244b3bf. Added ProjectControlsOff and load_serving; projected parsing consumes and validates stored controls while constructing the requested blind scope; all open/report routes use it; report retagging was removed; projected metadata and watch saves cannot overwrite the stronger snapshot. Focused core projection, cold-equivalence, checksum/control validation, and stronger-cache preservation tests pass. Watch-enabled fdu compiles and its unit suite is 45/46 with the sole failure being the pre-existing schema-doc mismatch owned by the machine-output integration. Python smoke, CLI goldens/harness, durable docs, full make check, and integration CI remain for the root integration pass.\n\nFollow-up on merged report/7: the public Python open and cache-only routes now assert projected warm service; the path-independence harness compares request, status, and all report content while excluding only nested provenance; it includes a real cli-watch-initial route that reconstructs every JSONL section; malformed schema/status answers fail classification. The cache and engine architecture docs now describe the all-route projection, stronger-snapshot save guard, capture-gap watch startup, nested status/provenance fields, and reverse-direction cold/miss semantics. Focused parser/harness and snapshot identity/control-state tests pass. The actual projected CLI watch save-guard test passes when run outside the sandbox, where macOS FSEvents delivery is available.
