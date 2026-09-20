---
type: is
id: is-01m2pyebrt6j2ghh4c4vcdvyxg
title: "P2.2.8: Python models from the wire schema; migrate smoke.py; delete the native dict; writer-equality test; docs"
kind: task
status: in_progress
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies: []
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
hold: null
hold_until: null
created_at: 2026-09-17T05:46:44.377Z
updated_at: 2026-09-20T05:13:36.256Z
started_at: 2026-09-20T05:13:36.256Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 2: The Answer Model and Writers", commit 8. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `crates/fdu-py/src/lib.rs`: delete `PyIndex::report` (`:191-249`) and `report_dict` through `tree_dict` (`:718-975`) after migrating their test; `cache_status_dict` (`:1484-1523`) and `Index.since` change dicts (`:500-530`) read wire keys (`invalidate`, a labelled reason), folding change sets into the change-record model.
- `crates/fdu-py/python/fdu/_models.py`, `_api.py`: `MetricValues`, `MetricRow`, `Detection`, `_metric_row`, `_tree`, `report_from_dict` (`:573-1155`); `_cache_status` (`:151-172`), `_change` (`:344-356`): optional metric fields, `Pages`, per-unit coverage, paths preferring `path_raw`, an iterative `_tree`; read only wire keys.
- Call sites: `crates/fdu-py/src/opened_binding.rs:935-941`, `opened.py:1089-1098`; `crates/fdu-py/src/lib.rs:981-992`, `:1102`, `:1260`, `:1418-1449`, `:1467-1481`; `_api.py:198-207`, `:258-272`, `:464-473`; `_models.py:778-800`.
- Documentation for `fdu.report/7` and `fdu.stream/2`: `docs/project/guides/cache-design.md`, `docs/project/architecture/fdu-surface-architecture.md`, `docs/project/architecture/fdu-engine-architecture.md`, `docs/project/release-notes/0.1.0.md`, `docs/project/guides/release-process.md`, `crates/fdu/src/skills/SKILL.md` (`:226-229`, `:272`), and `README.md` (`:266`, `:550`, `:561`, `:604`).

**Tests**

- Move `crates/fdu-py/tests/smoke.py:324-412` to parsed `render("json")`.
- Add a writer-equality test in `test_models.py`: every document kind equals the Python model. YAML parser parity stays in Node unless a reviewed Python YAML dependency is added.

**Done when**

- Record JSON, JSON Lines, and YAML render modes in `perf_probe` with `make perf-record`, then `make perf-ledger` and `make perf-report`.
- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risk: `render` still materializes a string for Python, and YAML indentation still grows with tree depth.
