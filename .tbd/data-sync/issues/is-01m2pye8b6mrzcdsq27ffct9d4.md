---
type: is
id: is-01m2pye8b6mrzcdsq27ffct9d4
title: "P2.1.3: Name-based grouping; probe results reported as detection counts"
kind: task
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye8ptnr6019rcb15g4w69
parent_id: is-01m2phzn814exmf4ty5vw6zha0
created_at: 2026-09-17T05:46:40.870Z
updated_at: 2026-09-17T05:46:55.927Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 1: Measured Values", commit 3. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `index.rs`: `pending_analysis_candidates` and `apply_analysis` drop `record.profile` and compare against the name-only classification.
- `query/query_report.rs`: `metric_summary` (`:1380-1565`) groups by `index.classify` only (delete `:1390-1392`); content detection is reported as counts, not regrouping.

**Tests**

- Update `deep_detection_drives_named_consumers_and_report_evidence` (`content/content_analysis.rs:576`).
- Goldens in `cli-content.tryscript.md` change where files regroup.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risk: regrouping is user-visible: extensionless scripts leave `languages`, and C++ headers named `.h` group as C.
