---
type: is
id: is-01m2pyeg0zegnzm857bvxrbpa6
title: "P2.4.5: Python projection check, cli-cache golden, cli-watch-initial harness route, and docs"
kind: task
status: open
priority: 0
version: 1
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies: []
parent_id: is-01m2pmrcb8he4a8a54zt957vcs
created_at: 2026-09-17T05:46:48.734Z
updated_at: 2026-09-17T05:46:48.734Z
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
