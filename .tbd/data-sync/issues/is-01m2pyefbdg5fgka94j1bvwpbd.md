---
type: is
id: is-01m2pyefbdg5fgka94j1bvwpbd
title: "P2.4.3: Delete the retag and forget_ignore_classification"
kind: task
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyefnttqs5rzqcwbgf2qb5
parent_id: is-01m2pmrcb8he4a8a54zt957vcs
created_at: 2026-09-17T05:46:48.044Z
updated_at: 2026-09-17T05:47:01.961Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 4: The `.gitignore` Observation Projection on Every Route", commit 3. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `execution.rs`: delete the retag (`:324-334`), with a debug assertion that the answer's scope equals the request's.
- `query/query_report.rs`: delete `forget_ignore_classification` (`:1014-1046`) and its export at `query.rs:21`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
