---
type: is
id: is-01m2pye5q72fsbb1n3y19mz0ma
title: "P1.4.1: Split Report into TreeStatus and ReportProvenance; writers emit the same bytes"
kind: task
status: closed
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
delegate: codex@spud10
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye61m0r7gny0f5nm361e3
parent_id: is-01m2pmra8yqrcxg27kc6ezg9vd
hold: null
hold_until: null
created_at: 2026-09-17T05:46:38.183Z
updated_at: 2026-09-23T08:14:06.514Z
started_at: 2026-09-20T04:40:17.230Z
closed_at: 2026-09-23T08:14:06.513Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 1, Item 4: Provenance and Tree Status", commit 1. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `query/query_status.rs` (new): `TreeStatus { complete, coverage, errors, errors_omitted }`, `ReportProvenance { source, freshness, scan_started_at, generated_at, tiers }`, `TierProvenance`, `TierState`. The report type is renamed from `Provenance`, which also names the per-entry type at the crate root (`lib.rs:120`).
- `query/query_report.rs`: `Provenance` (`:463-481`) and `Report` (`:779-829`) split into `status` and `provenance`.
- `report_format.rs`: the JSON envelope (`:575-600`) and YAML (`:976-996`) read the split fields with the same output.
- Test constructions in `query/query_report.rs`, `report_format.rs`, `content/content_analysis.rs`, and `execution.rs`.

**Tests**

- Machine goldens do not change.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
