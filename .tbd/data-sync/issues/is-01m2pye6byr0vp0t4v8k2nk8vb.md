---
type: is
id: is-01m2pye6byr0vp0t4v8k2nk8vb
title: "P1.4.3: Producers keep IndexState.source current; ReportProvenance::of; delete live_provenance and Python status fields"
kind: task
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye6pak55wktt2d0ge1je2
  - type: blocks
    target: is-01m2pyec35d01hhcj2n349he3v
  - type: blocks
    target: is-01m2pmrcrmjrm62bm1x3mxwgvm
parent_id: is-01m2pmra8yqrcxg27kc6ezg9vd
created_at: 2026-09-17T05:46:38.845Z
updated_at: 2026-09-17T05:47:43.889Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 4: Provenance and Tree Status", commit 3. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `index.rs`: producers keep `IndexState.source` (set at `snapshot.rs:650`, `:710`) current: `Scanned`, `Revalidated`, `Cached`.
- `ReportProvenance::of(index, generated_at)` and `of_walk(started, generated_at, complete)`; `report(index, request, generated_at)` computes status and provenance, so no caller builds provenance; `report_in` (`:965-1012`) and `report_summary` (`:1053`) stop reading `index.freshness()` directly (`:998`).
- `watch_session.rs`: delete `live_provenance` (`:396-404`); `report(generated_at)` (`:153`).
- `opened/read.rs`: `:254-264` replaced by `TreeStatus::of` and `ReportProvenance::of`.
- `execution.rs`: delete `:279-285` and `:310-322`.
- `crates/fdu/src/cli.rs`: delete construction at `:777`, `:801-811`, `:962`; `run` (`:729-742`) reads `report.status`.
- `crates/fdu-py/src/lib.rs`: delete `PyIndex` fields `errors`, `operation_complete`, `scan_started_at`, `source` (`:138-146`), set in `open` (`:1640-1653`), `scan` (`:1696-1727`), `refresh` (`:457-472`); `status_dict` (`:646`), getters (`:165`, `:177`), and `build_report` (`:591-597`) compute from the index.
- `crates/fdu-py/python/fdu/_api.py`: `Index.report` (`:260-270`) drops the errors override.
- Call sites: `examples/perf_probe.rs:555-561`; `live_provenance` callers at `crates/fdu/src/cli.rs:962`, `crates/fdu-py/src/lib.rs:1105`, `watch_session_integration.rs:246` and `:264`.

**Tests**

- A watch repaint over an unreadable subtree reports `complete: false`.
- Regenerate the opened-root golden, reading the diff.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
