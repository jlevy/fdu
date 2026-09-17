---
type: is
id: is-01m2pye4pgvg11q2txaafk5y0y
title: "P1.3.4: The command line and the Python binding build requests through RequestSpec"
kind: task
status: open
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye522p05a7rd9aj4390tk
  - type: blocks
    target: is-01m2pmrcrmjrm62bm1x3mxwgvm
parent_id: is-01m2pmr9ytx0ye8d701mr5vp9s
created_at: 2026-09-17T05:46:37.135Z
updated_at: 2026-09-17T05:47:43.870Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 3: The Request Model", commit 4. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `crates/fdu/src/cli.rs`: `run` (`:612-651`), `scan_config` (`:1109`), `parse_query` (`:1186`), `parse_analysis` (`:1245`), `resolved_query` (`:1180`), `resolve_views` (`:1488`) become `Cli::spec()` and `Request::build(.., SystemTime::now(), AxisNames::FLAGS)`; delete `:627-634`. Typed values (for example `--scan-depth`) reach `RequestSpec` through their `Display`, so a mismatch shows up as a golden difference.
- `crates/fdu-py/src/lib.rs`: `PyIndex` (`:134-147`) holds `basis`; `build_query_at` (`:1172-1246`) becomes `build_request(now, basis, spec)`; `build_report` (`:551`), `report_once` (`:1319`), `open` (`:1610`), `scan` (`:1685`) build through it; delete validation at `:375`, `:590`, `:1380`, `:1244`; `to_py_err` (`:38`) maps `InvalidRequest` to `ValueError`.

**Tests**

- Repoint the grammar tests in `crates/fdu/src/cli.rs` at the model.
- Goldens and the parity label class check that `RequestError` reproduces each surface's current wording.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
